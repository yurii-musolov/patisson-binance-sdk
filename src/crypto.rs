use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::{
    fmt::{self, Display, Formatter},
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
};

use crate::Timestamp;

/// A secret (API key, API secret) that never leaks through formatting.
///
/// Both `Display` and `Debug` print `REDACTED`; use [`SensitiveString::expose`]
/// to get the raw value. `Serialize` is intentionally not implemented so a
/// secret can't be written to JSON or logs by accident; `Deserialize` is kept
/// so secrets can still be loaded from a config file.
#[derive(Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct SensitiveString(String);

impl Display for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "REDACTED")
    }
}

impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SensitiveString").field(&"REDACTED").finish()
    }
}

impl std::ops::Deref for SensitiveString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for SensitiveString {
    fn from(s: String) -> Self {
        SensitiveString(s)
    }
}

impl From<&str> for SensitiveString {
    fn from(s: &str) -> Self {
        SensitiveString(s.to_string())
    }
}

impl AsRef<str> for SensitiveString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SensitiveString {
    /// The secret value. Avoid logging it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

pub fn hmac_sha256(key: impl AsRef<[u8]>, message: impl AsRef<[u8]>) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_ref()).unwrap();
    mac.update(message.as_ref());
    let mac = mac.finalize().into_bytes().to_vec();
    hex::encode(&mac)
}

pub fn sign_query(api_secret: &SensitiveString, timestamp: Timestamp, query: &str) -> String {
    let query = if query.is_empty() {
        format!("timestamp={timestamp}")
    } else {
        format!("{query}&timestamp={timestamp}")
    };
    let signature = hmac_sha256(api_secret.expose(), &query);
    format!("{query}&signature={signature}")
}

/// Signature of a WebSocket API request (HMAC-SHA256, hex): the `params`
/// (everything except `signature`) are sorted by name and joined as
/// `name=value&...` without URL-encoding.
pub fn sign_ws_params(api_secret: &SensitiveString, params: &[(&str, String)]) -> String {
    let mut sorted: Vec<&(&str, String)> = params.iter().collect();
    sorted.sort_by_key(|(name, _)| *name);
    let payload = sorted
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    hmac_sha256(api_secret.expose(), payload)
}

/// Local wall-clock time in milliseconds since the Unix epoch (0 if the
/// system clock is set before 1970).
pub fn timestamp() -> Timestamp {
    std::time::UNIX_EPOCH
        .elapsed()
        .map_or(0, |d| d.as_millis() as Timestamp)
}

/// Correction (in milliseconds) between the local clock and Binance's,
/// applied to the `timestamp` of every signed request.
///
/// Binance rejects signed requests whose timestamp is outside `recvWindow`
/// (error `-1021`), which happens when the local clock drifts. Measure the
/// offset with the server-time endpoint and share one `TimeOffset` (it is a
/// cheap, cloneable handle) with every private client:
///
/// ```no_run
/// # async fn run() -> Result<(), binance::spot::Error> {
/// use binance::{TimeOffset, timestamp};
/// use binance::spot::http::{PrivateClient, PrivateConfig, PublicClient, PublicConfig};
///
/// let offset = TimeOffset::new();
/// let public = PublicClient::new(PublicConfig::new("https://api.binance.com"))?;
/// let sent_at = timestamp();
/// let server = public.get_server_time().await?.result.server_time;
/// offset.update(server, sent_at, timestamp());
///
/// let cfg = PrivateConfig::new("https://api.binance.com", "key".into(), "secret".into())
///     .time_offset(offset.clone());
/// let private = PrivateClient::new(cfg)?;
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Default)]
pub struct TimeOffset(Arc<AtomicI64>);

impl TimeOffset {
    /// A zero offset: requests are signed with the local clock.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the offset directly: `server_time - local_time`, in milliseconds.
    pub fn set(&self, offset_ms: i64) {
        self.0.store(offset_ms, Ordering::Relaxed);
    }

    /// Current offset in milliseconds.
    pub fn get(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }

    /// Derive the offset from one server-time round trip: `server_time` was
    /// returned for a request sent at local time `sent_at` and answered at
    /// `received_at`. The server is assumed to have stamped the response in
    /// the middle of the round trip.
    pub fn update(&self, server_time: Timestamp, sent_at: Timestamp, received_at: Timestamp) {
        let midpoint = sent_at / 2 + received_at / 2 + (sent_at % 2 + received_at % 2) / 2;
        self.set(server_time as i64 - midpoint as i64);
    }

    /// Local time corrected by the offset, in milliseconds.
    pub fn now(&self) -> Timestamp {
        timestamp().saturating_add_signed(self.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_query() {
        let api_secret = SensitiveString(
            "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j".to_string(),
        );
        let query = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        let signature = "6497562735f592c5b199a6050647b2972e322c6daea096b47cf7b84694619206";
        let timestamp = 123456789;
        let expected = format!("{query}&timestamp={timestamp}&signature={signature}");

        let signed_query = sign_query(&api_secret, timestamp, query);

        assert_eq!(expected, signed_query);
    }

    #[test]
    fn sensitive_string_is_redacted_in_debug_and_display() {
        let secret = SensitiveString::from("top-secret-value");
        assert_eq!(format!("{secret}"), "REDACTED");
        let debug = format!("{secret:?}");
        assert!(!debug.contains("top-secret-value"), "got {debug}");
        assert_eq!(secret.expose(), "top-secret-value");
    }

    #[test]
    fn private_config_debug_does_not_leak_secrets() {
        let cfg = crate::spot::http::PrivateConfig::new(
            "https://example.com",
            SensitiveString::from("my-api-key"),
            SensitiveString::from("my-api-secret"),
        );
        let debug = format!("{cfg:?}");
        assert!(!debug.contains("my-api-key"), "got {debug}");
        assert!(!debug.contains("my-api-secret"), "got {debug}");
    }

    #[test]
    fn sensitive_string_deserializes_from_plain_string() {
        let s: SensitiveString = serde_json::from_str(r#""abc""#).unwrap();
        assert_eq!(s.expose(), "abc");
    }

    #[test]
    fn time_offset_shifts_the_clock_and_is_shared_between_clones() {
        let offset = TimeOffset::new();
        assert_eq!(offset.get(), 0);
        let shared = offset.clone();
        shared.set(-5_000);
        assert_eq!(offset.get(), -5_000);
        let local = timestamp();
        let corrected = offset.now();
        assert!(corrected + 5_000 >= local && corrected + 5_000 <= local + 1_000);
    }

    #[test]
    fn time_offset_update_uses_round_trip_midpoint() {
        let offset = TimeOffset::new();
        // Sent at 1000, answered at 1100: the server stamped ~1050.
        offset.update(3_050, 1_000, 1_100);
        assert_eq!(offset.get(), 2_000);
        offset.update(950, 1_000, 1_100);
        assert_eq!(offset.get(), -100);
    }

    #[test]
    fn ws_params_signature_matches_the_documentation() {
        // "SIGNED request example (HMAC)" in the Spot WebSocket API docs.
        let secret = SensitiveString::from(
            "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j",
        );
        let params = [
            ("symbol", "BTCUSDT".to_string()),
            ("side", "SELL".to_string()),
            ("type", "LIMIT".to_string()),
            ("timeInForce", "GTC".to_string()),
            ("quantity", "0.01000000".to_string()),
            ("price", "52000.00".to_string()),
            ("recvWindow", "100".to_string()),
            ("timestamp", "1645423376532".to_string()),
            (
                "apiKey",
                "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A".to_string(),
            ),
        ];
        assert_eq!(
            sign_ws_params(&secret, &params),
            "aa1b5712c094bc4e57c05a1a5c1fd8d88dcd628338ea863fec7b88e59fe2db24"
        );
    }

    #[test]
    fn test_sign_empty_query_has_no_leading_amp() {
        let api_secret = SensitiveString("secret".to_string());
        let signed = sign_query(&api_secret, 123, "");
        assert!(
            signed.starts_with("timestamp=123&signature="),
            "got {signed}"
        );
    }
}
