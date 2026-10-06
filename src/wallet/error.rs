use std::time::Duration;

use serde::Deserialize;

use crate::{http::SendError, rate_limit::RateLimitSource};

// Numeric error codes are universal across Binance products; the type lives
// at the crate root. Re-exported here so `binance::wallet::ErrorCode` resolves.
pub use crate::ErrorCode;

#[derive(Debug)]
/// Errors returned by this product's clients.
pub enum Error {
    /// Binance answered with an error body (`{"code":...,"msg":...}`).
    Api(ApiError),
    /// Non-2xx response whose body is not Binance's `{"code":...,"msg":...}`
    /// error, e.g. an HTML page from a proxy or CDN on 502/503.
    Http {
        /// HTTP status.
        status: reqwest::StatusCode,
        /// Response body as received.
        body: String,
    },
    /// I/O error.
    Io(std::io::Error),
    /// The configured API key isn't a valid HTTP header value (e.g. contains
    /// a newline or non-ASCII byte).
    InvalidApiKey(reqwest::header::InvalidHeaderValue),
    /// Other error, described by the message.
    Msg(String),
    /// Transport error (connect, TLS, timeout, ...).
    Reqwest(reqwest::Error),
    /// Either local budget exhausted (no request was sent) or the server
    /// returned 429/418. `source` distinguishes; `retry_after` is the
    /// minimum back-off before retrying.
    RateLimited {
        /// Minimum wait before retrying.
        retry_after: Duration,
        /// Whether the local limiter or the server refused the request.
        source: RateLimitSource,
        /// Binance's decoded error body when the server returned it
        /// (429/418); `None` when the request was rejected locally.
        api_err: Option<ApiError>,
    },
    /// JSON (de)serialization error.
    SerdeJson(serde_json::Error),
    /// Query string serialization error.
    SerdeUrlEncoded(serde_urlencoded::ser::Error),
    /// Response didn't match the model; the path says where.
    SerdePathToError(serde_path_to_error::Error<serde_json::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Api(error) => write!(f, "API error: code: {}, msg: {}", error.code, error.msg),
            Error::Http { status, body } => write!(
                f,
                "HTTP error: status: {status}, body: {}",
                crate::http::body_excerpt(body)
            ),
            Error::Io(error) => write!(f, "I/O error: {error}"),
            Error::InvalidApiKey(error) => write!(f, "invalid API key: {error}"),
            Error::Msg(msg) => write!(f, "{msg}"),
            Error::Reqwest(error) => write!(f, "reqwest error: {error}"),
            Error::RateLimited {
                retry_after,
                source,
                api_err,
            } => {
                write!(
                    f,
                    "rate limited ({source:?}): retry after {}s",
                    retry_after.as_secs()
                )?;
                if let Some(err) = api_err {
                    write!(f, " (code: {}, msg: {})", err.code, err.msg)?;
                }
                Ok(())
            }
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
                body,
            } => Self::RateLimited {
                retry_after,
                source,
                api_err: body.and_then(|b| crate::serde::deserialize_json::<ApiError>(&b).ok()),
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            Error::InvalidApiKey(error) => Some(error),
            Error::Reqwest(error) => Some(error),
            Error::SerdeJson(error) => Some(error),
            Error::SerdeUrlEncoded(error) => Some(error),
            Error::SerdePathToError(error) => Some(error),
            _ => None,
        }
    }
}

impl Error {
    /// Binance's error code, from an API error or a rate-limit response.
    pub fn api_code(&self) -> Option<ErrorCode> {
        match self {
            Error::Api(error) => Some(error.code),
            Error::RateLimited {
                api_err: Some(error),
                ..
            } => Some(error.code),
            _ => None,
        }
    }

    /// The same request may succeed if sent again: a network failure, a
    /// 5xx response, a server-side error code, rate limiting (after
    /// [`Self::retry_after`]) or `-1021` (after `sync_time`).
    ///
    /// For order placement check [`Self::is_execution_status_unknown`]
    /// first: the order may already exist.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Api(error) => error.code.is_transient() || error.code.is_invalid_timestamp(),
            Error::RateLimited { .. } => true,
            Error::Http { status, .. } => status.is_server_error(),
            Error::Reqwest(error) => error.is_connect() || error.is_timeout() || error.is_request(),
            _ => false,
        }
    }

    /// How long to wait before retrying a rate-limited request.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Error::RateLimited { retry_after, .. } => Some(*retry_after),
            _ => None,
        }
    }

    /// The request may have been executed by Binance even though no
    /// successful response arrived: `-1006`/`-1007` error codes, a 5xx
    /// response without an error body, or a timeout after the request was
    /// sent. For order placement or cancellation, query the order before
    /// retrying.
    pub fn is_execution_status_unknown(&self) -> bool {
        match self {
            Error::Api(error) => error.code.is_execution_status_unknown(),
            Error::Http { status, .. } => status.is_server_error(),
            Error::Reqwest(error) => error.is_timeout() && !error.is_connect(),
            _ => false,
        }
    }
}

impl From<crate::http::UnexpectedResponse> for Error {
    fn from(err: crate::http::UnexpectedResponse) -> Self {
        Error::Http {
            status: err.status,
            body: err.body,
        }
    }
}

/// Body shape Binance Wallet returns on errors: `{"code":-XXXX,"msg":"..."}`.
///
/// `code` uses the shared [`crate::ErrorCode`] newtype — see its docs for
/// the named constants and classification predicates. Wallet endpoints
/// occasionally surface -4xxx codes (e.g. -4060 withdraw amount must be
/// greater than zero) that aren't classified by the shared predicates; use
/// `code.raw()` to handle those.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ApiError {
    /// Binance error code.
    pub code: ErrorCode,
    /// Binance error message.
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

impl From<reqwest::header::InvalidHeaderValue> for Error {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        Error::InvalidApiKey(err)
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
