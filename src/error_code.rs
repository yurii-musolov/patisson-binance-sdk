//! Numeric error codes returned by the Binance API across all products
//! (spot, margin, derivatives).
//!
//! The codes themselves are universal — `-1021` means "invalid timestamp"
//! whether you got it from `/api/v3/order` or `/fapi/v1/order` — so the type
//! lives at the crate root and is re-exported from each product module
//! (`binance::spot::ErrorCode`, `binance::margin::ErrorCode`, …).
//!
//! `ErrorCode` is a transparent newtype over `i64`. Use the named constants
//! and `is_*` predicates for common cases; fall back to [`ErrorCode::raw`]
//! for product-specific codes (the -4xxx / -5xxx ranges on futures are not
//! exhaustively classified here).

use serde::{Deserialize, Serialize};

/// Numeric Binance error code, as returned in the `code` field of an error
/// response body.
///
/// Kept as a newtype around `i64` rather than a closed enum because Binance
/// adds new codes over time and product-specific endpoints use codes outside
/// the spot range. Use the named constants for the common ones and the
/// `is_*` predicates for category-based handling; fall back to
/// [`ErrorCode::raw`] for anything not classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ErrorCode(pub i64);

impl ErrorCode {
    pub const fn new(code: i64) -> Self {
        Self(code)
    }

    /// Raw numeric code as returned by Binance.
    pub const fn raw(self) -> i64 {
        self.0
    }
}

impl From<i64> for ErrorCode {
    fn from(code: i64) -> Self {
        Self(code)
    }
}

impl From<ErrorCode> for i64 {
    fn from(code: ErrorCode) -> Self {
        code.0
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ===== Named constants for commonly-checked codes =====

impl ErrorCode {
    // 10xx — general server / network
    pub const UNKNOWN: Self = Self(-1000);
    pub const DISCONNECTED: Self = Self(-1001);
    pub const UNAUTHORIZED: Self = Self(-1002);
    pub const TOO_MANY_REQUESTS: Self = Self(-1003);
    pub const UNEXPECTED_RESP: Self = Self(-1006);
    pub const TIMEOUT: Self = Self(-1007);
    pub const SERVER_BUSY: Self = Self(-1008);
    pub const TOO_MANY_ORDERS: Self = Self(-1015);
    pub const SERVICE_SHUTTING_DOWN: Self = Self(-1016);
    pub const UNSUPPORTED_OPERATION: Self = Self(-1020);
    pub const INVALID_TIMESTAMP: Self = Self(-1021);
    pub const INVALID_SIGNATURE: Self = Self(-1022);
    pub const TOO_MANY_CONNECTIONS: Self = Self(-1034);

    // 11xx — request validation
    pub const ILLEGAL_CHARS: Self = Self(-1100);
    pub const TOO_MANY_PARAMETERS: Self = Self(-1101);
    pub const MANDATORY_PARAM_EMPTY_OR_MALFORMED: Self = Self(-1102);
    pub const UNKNOWN_PARAM: Self = Self(-1103);
    pub const BAD_SYMBOL: Self = Self(-1121);
    pub const INVALID_LISTEN_KEY: Self = Self(-1125);
    pub const TOO_MANY_MESSAGES: Self = Self(-1181);
    pub const TOO_MANY_SUBSCRIPTIONS: Self = Self(-1191);

    // 20xx — trading / matching engine
    pub const NEW_ORDER_REJECTED: Self = Self(-2010);
    pub const CANCEL_REJECTED: Self = Self(-2011);
    pub const NO_SUCH_ORDER: Self = Self(-2013);
    pub const BAD_API_KEY_FMT: Self = Self(-2014);
    pub const REJECTED_MBX_KEY: Self = Self(-2015);
    pub const NO_TRADING_WINDOW: Self = Self(-2016);
    pub const ORDER_AMEND_REJECTED: Self = Self(-2038);
    pub const CLIENT_ORDER_ID_INVALID: Self = Self(-2039);
}

// ===== Classification methods =====

impl ErrorCode {
    /// Authentication / signing failure. The request will not succeed without
    /// fixing the API key, signature or timestamp.
    ///
    /// Covers: `UNAUTHORIZED` (-1002), `INVALID_TIMESTAMP` (-1021),
    /// `INVALID_SIGNATURE` (-1022), `BAD_API_KEY_FMT` (-2014),
    /// `REJECTED_MBX_KEY` (-2015).
    pub fn is_auth(self) -> bool {
        matches!(
            self,
            Self::UNAUTHORIZED
                | Self::INVALID_TIMESTAMP
                | Self::INVALID_SIGNATURE
                | Self::BAD_API_KEY_FMT
                | Self::REJECTED_MBX_KEY
        )
    }

    /// Local clock drifted outside the server's `recvWindow`. Re-sync time
    /// and retry. Code: `INVALID_TIMESTAMP` (-1021).
    pub fn is_invalid_timestamp(self) -> bool {
        self == Self::INVALID_TIMESTAMP
    }

    /// HMAC signature doesn't match. Usually a wrong API secret or a payload
    /// that was modified after signing. Code: `INVALID_SIGNATURE` (-1022).
    pub fn is_invalid_signature(self) -> bool {
        self == Self::INVALID_SIGNATURE
    }

    /// API key format is invalid (length / characters). Code: -2014.
    pub fn is_bad_api_key_format(self) -> bool {
        self == Self::BAD_API_KEY_FMT
    }

    /// API key was rejected because it's been deleted/disabled, the source
    /// IP isn't on its allowlist, or it lacks the permission this endpoint
    /// needs. Binance reports all three as code -2015.
    pub fn is_api_key_rejected(self) -> bool {
        self == Self::REJECTED_MBX_KEY
    }

    /// The key lacks the permission required for this action. Same wire code
    /// as [`Self::is_api_key_rejected`] (-2015); offered as a separate
    /// predicate purely for caller intent.
    pub fn is_wrong_permissions(self) -> bool {
        self == Self::REJECTED_MBX_KEY
    }

    /// Request was malformed — missing/extra parameter, bad value, bad
    /// symbol, etc. Range: -1199..=-1100.
    ///
    /// Product-specific codes outside this range (e.g. futures -4001) are
    /// not covered; use [`Self::raw`] to handle them.
    pub fn is_bad_request(self) -> bool {
        (-1199..=-1100).contains(&self.0)
    }

    /// Rate-limit / throughput error. Caller is sending too much.
    ///
    /// Covers: `TOO_MANY_REQUESTS` (-1003), `TOO_MANY_ORDERS` (-1015),
    /// `TOO_MANY_CONNECTIONS` (-1034), `TOO_MANY_MESSAGES` (-1181),
    /// `TOO_MANY_SUBSCRIPTIONS` (-1191).
    pub fn is_rate_limited(self) -> bool {
        matches!(
            self,
            Self::TOO_MANY_REQUESTS
                | Self::TOO_MANY_ORDERS
                | Self::TOO_MANY_CONNECTIONS
                | Self::TOO_MANY_MESSAGES
                | Self::TOO_MANY_SUBSCRIPTIONS
        )
    }

    /// Server-side problem; the request itself may have been valid.
    ///
    /// Covers: `UNKNOWN` (-1000), `DISCONNECTED` (-1001), `UNEXPECTED_RESP`
    /// (-1006), `TIMEOUT` (-1007), `SERVER_BUSY` (-1008),
    /// `SERVICE_SHUTTING_DOWN` (-1016).
    pub fn is_server_error(self) -> bool {
        matches!(
            self,
            Self::UNKNOWN
                | Self::DISCONNECTED
                | Self::UNEXPECTED_RESP
                | Self::TIMEOUT
                | Self::SERVER_BUSY
                | Self::SERVICE_SHUTTING_DOWN
        )
    }

    /// Worth retrying — server-side or rate-limit. Auth / bad-request errors
    /// are not transient and will fail again on retry.
    pub fn is_transient(self) -> bool {
        self.is_server_error() || self.is_rate_limited()
    }

    /// Matching engine rejected the order (filter / price / quantity rules,
    /// cancel-replace failure, amend rejected).
    ///
    /// Covers: `NEW_ORDER_REJECTED` (-2010), `CANCEL_REJECTED` (-2011),
    /// `ORDER_AMEND_REJECTED` (-2038).
    pub fn is_order_rejected(self) -> bool {
        matches!(
            self,
            Self::NEW_ORDER_REJECTED | Self::CANCEL_REJECTED | Self::ORDER_AMEND_REJECTED
        )
    }

    /// The referenced order ID doesn't exist (already filled/canceled or
    /// never placed). Code: `NO_SUCH_ORDER` (-2013).
    pub fn is_no_such_order(self) -> bool {
        self == Self::NO_SUCH_ORDER
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::deserialize_json;

    #[test]
    fn deserializes_from_bare_integer() {
        let code: ErrorCode = deserialize_json("-1102").unwrap();
        assert_eq!(code, ErrorCode::MANDATORY_PARAM_EMPTY_OR_MALFORMED);
        assert_eq!(code.raw(), -1102);
    }

    #[test]
    fn unknown_code_still_parses() {
        // A code Binance might add tomorrow — must not break parsing.
        let code: ErrorCode = deserialize_json("-9999").unwrap();
        assert_eq!(code.raw(), -9999);
        assert!(!code.is_auth());
        assert!(!code.is_bad_request());
        assert!(!code.is_transient());
    }

    #[test]
    fn classification_predicates() {
        assert!(ErrorCode::UNAUTHORIZED.is_auth());
        assert!(ErrorCode::INVALID_SIGNATURE.is_auth());
        assert!(ErrorCode::REJECTED_MBX_KEY.is_auth());
        assert!(ErrorCode::REJECTED_MBX_KEY.is_wrong_permissions());
        assert!(ErrorCode::REJECTED_MBX_KEY.is_api_key_rejected());

        assert!(ErrorCode::BAD_SYMBOL.is_bad_request());
        assert!(ErrorCode::MANDATORY_PARAM_EMPTY_OR_MALFORMED.is_bad_request());
        assert!(!ErrorCode::UNKNOWN.is_bad_request());

        assert!(ErrorCode::TOO_MANY_REQUESTS.is_rate_limited());
        assert!(ErrorCode::TOO_MANY_REQUESTS.is_transient());

        assert!(ErrorCode::SERVICE_SHUTTING_DOWN.is_server_error());
        assert!(ErrorCode::SERVICE_SHUTTING_DOWN.is_transient());

        assert!(ErrorCode::NEW_ORDER_REJECTED.is_order_rejected());
        assert!(ErrorCode::NO_SUCH_ORDER.is_no_such_order());
    }
}
