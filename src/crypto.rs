use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fmt::{self, Display, Formatter};

use crate::spot::Timestamp;

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

pub fn make_sign(api_secret: SensitiveString) -> impl Fn(&str) -> String {
    move |s: &str| format!("{}&signature={}", s, hmac_sha256(api_secret.expose(), s))
}

/// Return milliseconds.
pub fn timestamp() -> Timestamp {
    std::time::UNIX_EPOCH.elapsed().unwrap().as_millis() as Timestamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_query() {
        let api_secret = SensitiveString(
            "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j".to_string(),
        );
        let sign = make_sign(api_secret);
        let query = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        let signature = "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71";
        let expected = format!("{query}&signature={signature}");

        let body = sign(query);

        assert_eq!(expected, body);
    }
}
