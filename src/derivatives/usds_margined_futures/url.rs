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

    // Trading
    Order,

    // Account
    AccountV2,
    AccountV3,

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
            Self::Order => "/fapi/v1/order",
            Self::AccountV2 => "/fapi/v2/account",
            Self::AccountV3 => "/fapi/v3/account",
            Self::WebSocketApi => "/ws-fapi/v1",
            Self::Stream => "/stream",
            Self::Public => "/public",
            Self::Market => "/market",
            Self::Private => "/private",
        };
        write!(f, "{s}")
    }
}

pub const HEADER_RETRY_AFTER: &str = "Retry-After";
pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
