use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::fmt::{self, Display, Formatter};

use crate::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SensitiveString(String);

impl Display for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "REDACTED")
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
    let query = format!("{}&timestamp={}", query, timestamp);
    let signature = hmac_sha256(api_secret.expose(), &query);
    format!("{}&signature={}", query, signature)
}

/// Return milliseconds.
pub fn timestamp() -> Timestamp {
    std::time::UNIX_EPOCH.elapsed().unwrap().as_millis() as Timestamp
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
}
