//! Binance Margin Trading endpoints.
//!
//! Margin shares the spot REST host (`api.binance.com`) — margin-specific
//! routes live under `/sapi/v1/margin/...`. Market data and connectivity
//! probes are not duplicated here: use [`crate::spot::http::PublicClient`]
//! for `/api/v3/*` (ping, time, exchange info, klines, depth, tickers, …).

// Mainnet
pub const BASE_URL_API: &str = "https://api.binance.com";

/// Base URL for the margin user data WebSocket stream.
///
/// Margin user data flows over the spot stream host. After creating a
/// `listenKey` via REST, connect to `<BASE_URL_STREAM>/ws/<listenKey>`.
pub const BASE_URL_STREAM: &str = "wss://stream.binance.com:9443";

// Testnet
pub const BASE_URL_TESTNET_API: &str = "https://testnet.binance.vision";
pub const BASE_URL_TESTNET_STREAM: &str = "wss://stream.testnet.binance.vision:9443";

pub enum Path {
    // Margin account
    Account,
    IsolatedAccount,

    // Margin trading
    Order,

    // Margin metadata
    AllAssets,
    AllPairs,

    // Margin borrow / repay
    MaxBorrowable,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Account => "/sapi/v1/margin/account",
            Self::IsolatedAccount => "/sapi/v1/margin/isolated/account",
            Self::Order => "/sapi/v1/margin/order",
            Self::AllAssets => "/sapi/v1/margin/allAssets",
            Self::AllPairs => "/sapi/v1/margin/allPairs",
            Self::MaxBorrowable => "/sapi/v1/margin/maxBorrowable",
        };
        write!(f, "{s}")
    }
}

pub const HEADER_RETRY_AFTER: &str = "Retry-After";
pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
