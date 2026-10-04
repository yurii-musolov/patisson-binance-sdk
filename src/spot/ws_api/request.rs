use std::fmt;

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::{SensitiveString, Timestamp, crypto::sign_ws_params, spot::ws::MessageID};

/// A WebSocket API request: `{"id": ..., "method": ..., "params": {...}}`.
///
/// `Debug` redacts `apiKey` and `signature`, so requests can be logged.
#[derive(Serialize, PartialEq)]
pub struct Request {
    pub id: MessageID,
    pub method: &'static str,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub params: Map<String, Value>,
}

impl Request {
    /// Subscribe to the user data stream of the account owning `api_key`
    /// (`userDataStream.subscribe.signature`, works on any connection).
    ///
    /// `timestamp` is the current time in milliseconds (e.g.
    /// [`crate::timestamp`] or [`crate::TimeOffset::now`]); the request is
    /// rejected once `recv_window` (default 5000 ms, max 60000) has passed,
    /// so build it right before sending.
    pub fn user_data_stream_subscribe_signature(
        id: impl Into<MessageID>,
        api_key: &SensitiveString,
        api_secret: &SensitiveString,
        timestamp: Timestamp,
        recv_window: Option<u64>,
    ) -> Self {
        let mut signed = vec![
            ("apiKey", api_key.expose().to_owned()),
            ("timestamp", timestamp.to_string()),
        ];
        if let Some(recv_window) = recv_window {
            signed.push(("recvWindow", recv_window.to_string()));
        }
        let signature = sign_ws_params(api_secret, &signed);

        let mut params = Map::new();
        params.insert("apiKey".into(), json!(api_key.expose()));
        params.insert("timestamp".into(), json!(timestamp));
        if let Some(recv_window) = recv_window {
            params.insert("recvWindow".into(), json!(recv_window));
        }
        params.insert("signature".into(), json!(signature));
        Self {
            id: id.into(),
            method: "userDataStream.subscribe.signature",
            params,
        }
    }

    /// Stop one subscription, or every subscription of the connection when
    /// `subscription_id` is `None` (`userDataStream.unsubscribe`).
    pub fn user_data_stream_unsubscribe(
        id: impl Into<MessageID>,
        subscription_id: Option<u64>,
    ) -> Self {
        let mut params = Map::new();
        if let Some(subscription_id) = subscription_id {
            params.insert("subscriptionId".into(), json!(subscription_id));
        }
        Self {
            id: id.into(),
            method: "userDataStream.unsubscribe",
            params,
        }
    }

    /// List the active subscriptions of the connection
    /// (`session.subscriptions`).
    pub fn session_subscriptions(id: impl Into<MessageID>) -> Self {
        Self {
            id: id.into(),
            method: "session.subscriptions",
            params: Map::new(),
        }
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params: Map<String, Value> = self
            .params
            .iter()
            .map(|(name, value)| match name.as_str() {
                "apiKey" | "signature" => (name.clone(), json!("REDACTED")),
                _ => (name.clone(), value.clone()),
            })
            .collect();
        f.debug_struct("Request")
            .field("id", &self.id)
            .field("method", &self.method)
            .field("params", &params)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_signature_request_is_signed_and_redacted_in_debug() {
        let key = SensitiveString::from("my-api-key");
        let secret = SensitiveString::from("my-api-secret");
        let request =
            Request::user_data_stream_subscribe_signature(1, &key, &secret, 1747385641636, None);

        let expected_signature = sign_ws_params(
            &secret,
            &[
                ("apiKey", "my-api-key".to_string()),
                ("timestamp", "1747385641636".to_string()),
            ],
        );
        let json: Value = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "userDataStream.subscribe.signature");
        assert_eq!(json["id"], 1);
        assert_eq!(json["params"]["apiKey"], "my-api-key");
        assert_eq!(json["params"]["timestamp"], 1747385641636u64);
        assert_eq!(json["params"]["signature"], expected_signature);
        assert!(json["params"].get("recvWindow").is_none());

        let debug = format!("{request:?}");
        assert!(!debug.contains("my-api-key"), "{debug}");
        assert!(!debug.contains(&expected_signature), "{debug}");
    }

    #[test]
    fn recv_window_is_signed_too() {
        let key = SensitiveString::from("k");
        let secret = SensitiveString::from("s");
        let request =
            Request::user_data_stream_subscribe_signature(1, &key, &secret, 5, Some(3000));
        let expected = sign_ws_params(
            &secret,
            &[
                ("apiKey", "k".to_string()),
                ("recvWindow", "3000".to_string()),
                ("timestamp", "5".to_string()),
            ],
        );
        assert_eq!(request.params["signature"], expected);
        assert_eq!(request.params["recvWindow"], 3000);
    }

    #[test]
    fn unsubscribe_and_list_requests() {
        let all = serde_json::to_value(Request::user_data_stream_unsubscribe("a", None)).unwrap();
        assert_eq!(
            all,
            json!({"id": "a", "method": "userDataStream.unsubscribe"})
        );
        let one = serde_json::to_value(Request::user_data_stream_unsubscribe(2, Some(7))).unwrap();
        assert_eq!(one["params"]["subscriptionId"], 7);
        let list = serde_json::to_value(Request::session_subscriptions(3)).unwrap();
        assert_eq!(list, json!({"id": 3, "method": "session.subscriptions"}));
    }
}
