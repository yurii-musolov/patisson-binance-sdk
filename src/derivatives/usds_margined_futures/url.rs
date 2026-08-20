//! Binance USDⓈ-M Futures endpoints (fapi).

// Mainnet
pub const BASE_URL_API: &str = "https://fapi.binance.com";
pub const BASE_URL_WEBSOCKET_API: &str = "wss://ws-fapi.binance.com";
pub const BASE_URL_STREAM: &str = "wss://fstream.binance.com";

// Testnet
pub const BASE_URL_TESTNET_API: &str = "https://testnet.binancefuture.com";
pub const BASE_URL_TESTNET_WEBSOCKET_API: &str = "wss://testnet.binancefuture.com";
pub const BASE_URL_TESTNET_STREAM: &str = "wss://fstream.binancefuture.com";

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

    // Account
    AccountV2,
    AccountV3,
    BalanceV2,

    // User data stream lifecycle
    ListenKey,

    // WebSocket
    WebSocketApi,
    Stream,
    Public,
    Market,
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
