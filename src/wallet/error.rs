use std::time::Duration;

use serde::Deserialize;

use crate::{http::SendError, rate_limit::RateLimitSource};

// Numeric error codes are universal across Binance products; the type lives
// at the crate root. Re-exported here so `binance::wallet::ErrorCode` resolves.
pub use crate::ErrorCode;

#[derive(Debug)]
pub enum Error {
    Api(ApiError),
    Io(std::io::Error),
    Msg(String),
    Reqwest(reqwest::Error),
    /// Either local budget exhausted (no request was sent) or the server
    /// returned 429/418. `source` distinguishes; `retry_after` is the
    /// minimum back-off before retrying.
    RateLimited {
        retry_after: Duration,
        source: RateLimitSource,
    },
    SerdeJson(serde_json::Error),
    SerdeUrlEncoded(serde_urlencoded::ser::Error),
    SerdePathToError(serde_path_to_error::Error<serde_json::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Api(error) => write!(f, "API error: code: {}, msg: {}", error.code, error.msg),
            Error::Io(error) => write!(f, "I/O error: {error}"),
            Error::Msg(msg) => write!(f, "{msg}"),
            Error::Reqwest(error) => write!(f, "reqwest error: {error}"),
            Error::RateLimited {
                retry_after,
                source,
            } => write!(
                f,
                "rate limited ({source:?}): retry after {}s",
                retry_after.as_secs()
            ),
            Error::SerdeJson(error) => write!(f, "serde_json error: {error}"),
            Error::SerdeUrlEncoded(error) => write!(f, "serde_urlencoded error: {error}"),
            Error::SerdePathToError(error) => write!(
                f,
                "serde_path_to_error error: path: {}, msg: {}",
                error.path(),
                error.inner()
            ),
        }
    }
}

impl From<SendError> for Error {
    fn from(err: SendError) -> Self {
        match err {
            SendError::Reqwest(e) => Self::Reqwest(e),
            SendError::RateLimited {
                retry_after,
                source,
            } => Self::RateLimited {
                retry_after,
                source,
            },
        }
    }
}

impl std::error::Error for Error {}

/// Body shape Binance Wallet returns on errors: `{"code":-XXXX,"msg":"..."}`.
///
/// `code` uses the shared [`crate::ErrorCode`] newtype — see its docs for
/// the named constants and classification predicates. Wallet endpoints
/// occasionally surface -4xxx codes (e.g. -4060 withdraw amount must be
/// greater than zero) that aren't classified by the shared predicates; use
/// `code.raw()` to handle those.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ApiError {
    pub code: ErrorCode,
    pub msg: String,
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Error::Api(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Msg(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Msg(msg.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}

impl From<serde_urlencoded::ser::Error> for Error {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Error::SerdeUrlEncoded(err)
    }
}

impl From<serde_path_to_error::Error<serde_json::Error>> for Error {
    fn from(err: serde_path_to_error::Error<serde_json::Error>) -> Self {
        Error::SerdePathToError(err)
    }
}
