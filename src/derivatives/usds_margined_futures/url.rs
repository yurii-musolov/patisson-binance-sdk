//! Binance USDⓈ-M Futures endpoints (fapi).

// Wire models: names mirror Binance's documentation field by field; they are
// documented where the meaning isn't obvious. Full coverage comes later.
#![allow(missing_docs)]

// Mainnet
pub const BASE_URL_API: &str = "https://fapi.binance.com";
pub const BASE_URL_WEBSOCKET_API: &str = "wss://ws-fapi.binance.com";
pub const BASE_URL_STREAM: &str = "wss://fstream.binance.com";

// Demo mode (https://demo.binance.com); needs a demo API key.
pub const BASE_URL_DEMO_API: &str = "https://demo-fapi.binance.com";
pub const BASE_URL_DEMO_STREAM: &str = "wss://demo-fstream.binance.com";

// Testnet
pub const BASE_URL_TESTNET_API: &str = "https://testnet.binancefuture.com";
pub const BASE_URL_TESTNET_WEBSOCKET_API: &str = "wss://testnet.binancefuture.com";
pub const BASE_URL_TESTNET_STREAM: &str = "wss://fstream.binancefuture.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Path {
    // General
    Ping,
    Time,
    ExchangeInfo,

    // Market Data
    Depth,
    KLines,
    TickerPrice,
    TickerBookTicker,
    PremiumIndex,

    // Trading
    Order,
    OpenOrders,
    AllOpenOrders,
    AllOrders,
    UserTrades,
    Leverage,
    MarginType,
    PositionSideDual,
    PositionRiskV3,

    // Algo (conditional) orders
    AlgoOrder,
    OpenAlgoOrders,
    AllAlgoOrders,
    AlgoOpenOrders,

    // Account
    AccountV2,
    AccountV3,
    BalanceV2,

    // User data stream lifecycle
    ListenKey,

    // WebSocket
    WebSocketApi,
    Stream,
    /// Order book streams (`depth`, `bookTicker`); see `ws::StreamName::path`.
    Public,
    /// All other market streams (`aggTrade`, `kline`, `markPrice`, ...).
    Market,
    /// User data streams: `<BASE_URL_STREAM>/private/ws/<listenKey>`.
    Private,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Ping => "/fapi/v1/ping",
            Self::Time => "/fapi/v1/time",
            Self::ExchangeInfo => "/fapi/v1/exchangeInfo",
            Self::Depth => "/fapi/v1/depth",
            Self::KLines => "/fapi/v1/klines",
            Self::TickerPrice => "/fapi/v1/ticker/price",
            Self::TickerBookTicker => "/fapi/v1/ticker/bookTicker",
            Self::PremiumIndex => "/fapi/v1/premiumIndex",
            Self::Order => "/fapi/v1/order",
            Self::AlgoOrder => "/fapi/v1/algoOrder",
            Self::OpenAlgoOrders => "/fapi/v1/openAlgoOrders",
            Self::AllAlgoOrders => "/fapi/v1/allAlgoOrders",
            Self::AlgoOpenOrders => "/fapi/v1/algoOpenOrders",
            Self::OpenOrders => "/fapi/v1/openOrders",
            Self::AllOpenOrders => "/fapi/v1/allOpenOrders",
            Self::AllOrders => "/fapi/v1/allOrders",
            Self::UserTrades => "/fapi/v1/userTrades",
            Self::Leverage => "/fapi/v1/leverage",
            Self::MarginType => "/fapi/v1/marginType",
            Self::PositionSideDual => "/fapi/v1/positionSide/dual",
            Self::PositionRiskV3 => "/fapi/v3/positionRisk",
            Self::AccountV2 => "/fapi/v2/account",
            Self::AccountV3 => "/fapi/v3/account",
            Self::BalanceV2 => "/fapi/v2/balance",
            Self::ListenKey => "/fapi/v1/listenKey",
            Self::WebSocketApi => "/ws-fapi/v1",
            Self::Stream => "/stream",
            Self::Public => "/public",
            Self::Market => "/market",
            Self::Private => "/private",
        };
        write!(f, "{s}")
    }
}

pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
