/// For APIs that only send public market data
pub const BASE_URL_API_DATA: &str = "https://data-api.binance.vision";
pub const BASE_URL_API: &str = "https://api.binance.com";
pub const BASE_URL_API_GCP: &str = "https://api-gcp.binance.com";
pub const BASE_URL_API1: &str = "https://api1.binance.com";
pub const BASE_URL_API2: &str = "https://api2.binance.com";
pub const BASE_URL_API3: &str = "https://api3.binance.com";
pub const BASE_URL_API4: &str = "https://api4.binance.com";

pub const BASE_URL_STREAM_DATA1: &str = "wss://data-stream.binance.vision:9443";
pub const BASE_URL_STREAM_DATA2: &str = "wss://data-stream.binance.vision:443";

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
    OrderCancelReplace,
    OrderAmendKeepPriority,
    OrderListOCO,
    OrderListOTO,
    OrderListOTOCO,
    OrderList,
    SOROrder,
    SOROrderTest,

    RateLimitOrder,
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
            Self::OrderCancelReplace => "/api/v3/order/cancelReplace",
            Self::OrderAmendKeepPriority => "/api/v3/order/amend/keepPriority",
            Self::OrderListOCO => "/api/v3/orderList/oco",
            Self::OrderListOTO => "/api/v3/orderList/oto",
            Self::OrderListOTOCO => "/api/v3/orderList/otoco",
            Self::OrderList => "/api/v3/orderList",
            Self::SOROrder => "/api/v3/sor/order",
            Self::SOROrderTest => "/api/v3/sor/order/test",

            Self::RateLimitOrder => "/api/v3/rateLimit/order",
        };

        write!(f, "{}", s)
    }
}

// TODO: X-MBX-USED-WEIGHT-(intervalNum)(intervalLetter)
pub const HEADER_RETRY_AFTER: &str = "Retry-After";
pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
