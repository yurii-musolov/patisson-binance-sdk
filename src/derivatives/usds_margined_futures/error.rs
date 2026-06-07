use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
    Api(ApiError),
    Io(std::io::Error),
    Msg(String),
    Reqwest(reqwest::Error),
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

impl std::error::Error for Error {}

/// Body shape Binance Futures returns on errors: `{"code":-XXXX,"msg":"..."}`.
///
/// `code` is intentionally a plain `i64`: USDⓈ-M Futures has codes spread
/// across the -1xxx, -2xxx, -4xxx and -5xxx ranges and a closed enum would be
/// difficult to keep complete. Callers can match on numeric ranges or map to
/// their own typed errors.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ApiError {
    pub code: i64,
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
