use serde::Deserialize;

use crate::{ErrorCode, spot::ws::MessageID, ws::ReceivedMessage};

use super::UserDataEvent;

/// A frame received on a WebSocket API connection: an event pushed for a
/// subscription, or the response to a request.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum IncomingMessage {
    Event(EventMessage),
    Response(ResponseMessage),
}

impl ReceivedMessage for IncomingMessage {
    fn server_shutdown_event_time(&self) -> Option<u64> {
        match self {
            IncomingMessage::Event(EventMessage {
                event: UserDataEvent::ServerShutdown(event),
                ..
            }) => Some(event.event_time),
            _ => None,
        }
    }
}

/// `{"subscriptionId": 0, "event": {...}}`. `serverShutdown` arrives without
/// a subscription id.
#[derive(Debug, Deserialize, PartialEq)]
pub struct EventMessage {
    #[serde(rename = "subscriptionId")]
    pub subscription_id: Option<u64>,
    pub event: UserDataEvent,
}

/// `{"id": ..., "status": 200, "result": ...}` or, on failure,
/// `{"id": ..., "status": 4xx, "error": {"code": ..., "msg": ...}}`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ResponseMessage {
    pub id: Option<MessageID>,
    pub status: u16,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<ResponseError>,
    #[serde(rename = "rateLimits", default)]
    pub rate_limits: Option<Vec<serde_json::Value>>,
}

impl ResponseMessage {
    /// `subscriptionId` of a successful `userDataStream.subscribe*` response.
    pub fn subscription_id(&self) -> Option<u64> {
        self.result.as_ref()?.get("subscriptionId")?.as_u64()
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ResponseError {
    pub code: ErrorCode,
    pub msg: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}
