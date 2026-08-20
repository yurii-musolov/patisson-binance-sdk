//! Binance COIN-M Futures endpoints (dapi).

// Mainnet
pub const BASE_URL_API: &str = "https://dapi.binance.com";
pub const BASE_URL_WEBSOCKET_API: &str = "wss://ws-dapi.binance.com";
pub const BASE_URL_STREAM: &str = "wss://dstream.binance.com";

// Testnet
pub const BASE_URL_TESTNET_API: &str = "https://testnet.binancefuture.com";
pub const BASE_URL_TESTNET_STREAM: &str = "wss://dstream.binancefuture.com";

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
    PositionRisk,
    Leverage,
    MarginType,
    PositionSideDual,

    // Account
    Account,
    Balance,

    // User data stream lifecycle
    ListenKey,

    // WebSocket
    WebSocketApi,
    Stream,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Ping => "/dapi/v1/ping",
            Self::Time => "/dapi/v1/time",
            Self::ExchangeInfo => "/dapi/v1/exchangeInfo",
            Self::Depth => "/dapi/v1/depth",
            Self::KLines => "/dapi/v1/klines",
            Self::TickerPrice => "/dapi/v1/ticker/price",
            Self::TickerBookTicker => "/dapi/v1/ticker/bookTicker",
            Self::PremiumIndex => "/dapi/v1/premiumIndex",
            Self::Order => "/dapi/v1/order",
            Self::OpenOrders => "/dapi/v1/openOrders",
            Self::AllOpenOrders => "/dapi/v1/allOpenOrders",
            Self::AllOrders => "/dapi/v1/allOrders",
            Self::UserTrades => "/dapi/v1/userTrades",
            Self::PositionRisk => "/dapi/v1/positionRisk",
            Self::Leverage => "/dapi/v1/leverage",
            Self::MarginType => "/dapi/v1/marginType",
            Self::PositionSideDual => "/dapi/v1/positionSide/dual",
            Self::Account => "/dapi/v1/account",
            Self::Balance => "/dapi/v1/balance",
            Self::ListenKey => "/dapi/v1/listenKey",
            Self::WebSocketApi => "/ws-dapi/v1",
            Self::Stream => "/stream",
        };
        write!(f, "{s}")
    }
}

pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
