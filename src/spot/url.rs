// Mainnet

pub const BASE_URL_API: &str = "https://api.binance.com";
pub const BASE_URL_API_GCP: &str = "https://api-gcp.binance.com";
pub const BASE_URL_API1: &str = "https://api1.binance.com";
pub const BASE_URL_API2: &str = "https://api2.binance.com";
pub const BASE_URL_API3: &str = "https://api3.binance.com";
pub const BASE_URL_API4: &str = "https://api4.binance.com";

pub const BASE_URL_WEBSOCKET_API1: &str = "wss://ws-api.binance.com";
pub const BASE_URL_WEBSOCKET_API2: &str = "wss://ws-api.binance.com:443";
pub const BASE_URL_WEBSOCKET_API3: &str = "wss://ws-api.binance.com:9443";

pub const BASE_URL_STREAM1: &str = "wss://stream.binance.com";
pub const BASE_URL_STREAM2: &str = "wss://stream.binance.com:443";
pub const BASE_URL_STREAM3: &str = "wss://stream.binance.com:9443";

/// Market Data Only URLs
/// These URLs do not require any authentication (i.e. The API key is not necessary) and serve only public market data.
pub const BASE_URL_MARKET_DATA_API: &str = "https://data-api.binance.vision";
pub const BASE_URL_MARKET_DATA_STREAM1: &str = "wss://data-stream.binance.vision";
pub const BASE_URL_MARKET_DATA_STREAM2: &str = "wss://data-stream.binance.vision:443";
pub const BASE_URL_MARKET_DATA_STREAM3: &str = "wss://data-stream.binance.vision:9443";

// Demo
pub const BASE_URL_DEMO_API: &str = "https://demo-api.binance.com";

// Testnet

pub const BASE_URL_TESTNET_API: &str = "https://testnet.binance.vision";

pub const BASE_URL_TESTNET_WEBSOCKET_API1: &str = "wss://ws-api.testnet.binance.vision";
pub const BASE_URL_TESTNET_WEBSOCKET_API2: &str = "wss://ws-api.testnet.binance.vision:9443";

pub const BASE_URL_TESTNET_STREAM1: &str = "wss://stream.testnet.binance.com";
pub const BASE_URL_TESTNET_STREAM2: &str = "wss://stream.testnet.binance.com:443";
pub const BASE_URL_TESTNET_STREAM3: &str = "wss://stream.testnet.binance.com:9443";

pub enum Path {
    // General endpoints.
    Ping,
    Time,
    ExchangeInfo,

    // Market Data endpoints.
    Depth,
    Trades,
    HistoricalTrades,
    AggTrades,
    KLines,
    UIKLines,
    AvgPrice,
    Ticker24hr,
    TickerTradingDay,
    TickerPrice,
    TickerBook,
    Ticker,

    // Trading endpoints
    Order,
    OrderTest,
    OpenOrders,
    AllOrders,
    OrderCancelReplace,
    OrderAmendKeepPriority,
    OrderListOCO,
    OrderListOTO,
    OrderListOTOCO,
    OrderList,
    SOROrder,
    SOROrderTest,

    // Account endpoints
    Account,
    AccountCommission,
    MyTrades,
    RateLimitOrder,

    // User data stream lifecycle
    UserDataStream,

    // Websocket endpoints
    WebSocketApiV3,
    WebSocket,
    Stream,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            // General endpoints.
            Self::Ping => "/api/v3/ping",
            Self::Time => "/api/v3/time",
            Self::ExchangeInfo => "/api/v3/exchangeInfo",

            // Market Data endpoints.
            Self::Depth => "/api/v3/depth",
            Self::Trades => "/api/v3/trades",
            Self::HistoricalTrades => "/api/v3/historicalTrades",
            Self::AggTrades => "/api/v3/aggTrades",
            Self::KLines => "/api/v3/klines",
            Self::UIKLines => "/api/v3/uiKlines",
            Self::AvgPrice => "/api/v3/avgPrice",
            Self::Ticker24hr => "/api/v3/ticker/24hr",
            Self::TickerTradingDay => "/api/v3/ticker/tradingDay",
            Self::TickerPrice => "/api/v3/ticker/price",
            Self::TickerBook => "/api/v3/ticker/bookTicker",
            Self::Ticker => "/api/v3/ticker",

            // Trading endpoints
            Self::Order => "/api/v3/order",
            Self::OrderTest => "/api/v3/order/test",
            Self::OpenOrders => "/api/v3/openOrders",
            Self::AllOrders => "/api/v3/allOrders",
            Self::OrderCancelReplace => "/api/v3/order/cancelReplace",
            Self::OrderAmendKeepPriority => "/api/v3/order/amend/keepPriority",
            Self::OrderListOCO => "/api/v3/orderList/oco",
            Self::OrderListOTO => "/api/v3/orderList/oto",
            Self::OrderListOTOCO => "/api/v3/orderList/otoco",
            Self::OrderList => "/api/v3/orderList",
            Self::SOROrder => "/api/v3/sor/order",
            Self::SOROrderTest => "/api/v3/sor/order/test",

            // Account endpoints
            Self::Account => "/api/v3/account",
            Self::AccountCommission => "/api/v3/account/commission",
            Self::MyTrades => "/api/v3/myTrades",
            Self::RateLimitOrder => "/api/v3/rateLimit/order",

            // User data stream lifecycle
            Self::UserDataStream => "/api/v3/userDataStream",

            // Websocket endpoints
            Self::WebSocketApiV3 => "/ws-api/v3",
            Self::WebSocket => "/ws",
            Self::Stream => "/stream",
        };

        write!(f, "{s}")
    }
}

pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
pub const HEADER_X_MBX_TIME_UNIT: &str = "X-MBX-TIME-UNIT";
pub const HEADER_VALUE_MICROSECOND: &str = "MICROSECOND";
