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

    // Trading
    Order,

    // Account
    Account,

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
            Self::Order => "/dapi/v1/order",
            Self::Account => "/dapi/v1/account",
            Self::WebSocketApi => "/ws-dapi/v1",
            Self::Stream => "/stream",
        };
        write!(f, "{s}")
    }
}

pub const HEADER_RETRY_AFTER: &str = "Retry-After";
pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
