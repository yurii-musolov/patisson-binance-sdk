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
    OpenOrders,
    AllOrders,
    MyTrades,
    ForceLiquidationRec,

    // Margin metadata
    AllAssets,
    AllPairs,
    IsolatedAllPairs,
    PriceIndex,

    // Margin borrow / repay
    MaxBorrowable,
    BorrowRepay,
    InterestRateHistory,

    // Margin transfer
    MaxTransferable,

    // User data stream lifecycle
    UserDataStream,
    UserDataStreamIsolated,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Account => "/sapi/v1/margin/account",
            Self::IsolatedAccount => "/sapi/v1/margin/isolated/account",
            Self::Order => "/sapi/v1/margin/order",
            Self::OpenOrders => "/sapi/v1/margin/openOrders",
            Self::AllOrders => "/sapi/v1/margin/allOrders",
            Self::MyTrades => "/sapi/v1/margin/myTrades",
            Self::ForceLiquidationRec => "/sapi/v1/margin/forceLiquidationRec",
            Self::AllAssets => "/sapi/v1/margin/allAssets",
            Self::AllPairs => "/sapi/v1/margin/allPairs",
            Self::IsolatedAllPairs => "/sapi/v1/margin/isolated/pair",
            Self::PriceIndex => "/sapi/v1/margin/priceIndex",
            Self::MaxBorrowable => "/sapi/v1/margin/maxBorrowable",
            Self::BorrowRepay => "/sapi/v1/margin/borrow-repay",
            Self::InterestRateHistory => "/sapi/v1/margin/interestRateHistory",
            Self::MaxTransferable => "/sapi/v1/margin/maxTransferable",
            Self::UserDataStream => "/sapi/v1/userDataStream",
            Self::UserDataStreamIsolated => "/sapi/v1/userDataStream/isolated",
        };
        write!(f, "{s}")
    }
}

pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
