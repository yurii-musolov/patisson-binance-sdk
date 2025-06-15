use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::spot::{
    ExchangeFilter, KlineInterval, OrderType, RateLimitInterval, RateLimiter, STPMode, SymbolStatus,
};

pub type Timestamp = u64;

#[derive(Debug, PartialEq)]
pub struct Response<T> {
    pub result: T,
    pub headers: Headers,
}

#[derive(Debug, PartialEq)]
pub struct Headers {
    pub retry_after: Option<Timestamp>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TestConnectivity {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    server_time: Timestamp,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangeInfoParams {
    /// Example: curl -X GET "https://api.binance.com/api/v3/exchangeInfo?symbol=BNBBTC"
    pub symbol: Option<String>,
    /// Examples: curl -X GET "https://api.binance.com/api/v3/exchangeInfo?symbols=%5B%22BNBBTC%22,%22BTCUSDT%22%5D"
    /// or
    /// curl -g -X GET 'https://api.binance.com/api/v3/exchangeInfo?symbols=["BTCUSDT","BNBBTC"]'
    /// TODO: Check serialization.
    pub symbols: Option<Vec<String>>,
    /// Examples: curl -X GET "https://api.binance.com/api/v3/exchangeInfo?permissions=SPOT"
    /// or
    /// curl -X GET "https://api.binance.com/api/v3/exchangeInfo?permissions=%5B%22MARGIN%22%2C%22LEVERAGED%22%5D"
    /// or
    /// curl -g -X GET 'https://api.binance.com/api/v3/exchangeInfo?permissions=["MARGIN","LEVERAGED"]'
    /// TODO: Check serialization.
    pub permissions: Option<Vec<String>>,
    /// Controls whether the content of the permissionSets field is populated or not. Defaults to true
    pub show_permission_sets: Option<bool>,
    /// Filters symbols that have this tradingStatus. Valid values: TRADING, HALT, BREAK
    /// Cannot be used in combination with symbols or symbol.
    pub symbol_status: Option<SymbolStatus>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    timezone: String,
    server_time: Timestamp,
    rate_limits: Vec<RateLimit>,
    exchange_filters: Vec<ExchangeFilter>,
    symbols: Vec<SymbolInfo>,
    /// Optional field. Present only when SOR is available.
    /// LINK: https://github.com/binance/binance-spot-api-docs/blob/master/faqs/sor_faq.md
    sors: Option<Vec<SOR>>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    rate_limit_type: RateLimiter,
    interval: RateLimitInterval,
    interval_num: u64,
    limit: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    symbol: String,
    status: SymbolStatus,
    base_asset: String,
    base_asset_precision: u8, // value range: [0:8]
    quote_asset: String,
    // INFO: 'quote_precision' will be removed in future api versions (v4+)
    quote_asset_precision: u8,      // value range: [0:8]
    base_commission_precision: u8,  // value range: [0:8]
    quote_commission_precision: u8, // value range: [0:8]
    order_types: Vec<OrderType>,
    iceberg_allowed: bool,
    oco_allowed: bool,
    oto_allowed: bool,
    quote_order_qty_market_allowed: bool,
    allow_trailing_stop: bool,
    cancel_replace_allowed: bool,
    amend_allowed: bool,
    is_spot_trading_allowed: bool,
    is_margin_trading_allowed: bool,
    filters: Vec<Filter>,
    permissions: Vec<String>,
    permission_sets: Vec<Vec<String>>,
    default_self_trade_prevention_mode: STPMode,
    allowed_self_trade_prevention_modes: Vec<STPMode>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    // TODO:
}

/// Smart Order Routing (SOR).
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SOR {
    base_asset: String,
    symbols: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderBookParams {
    pub symbol: String,
    /// Default: 100; Maximum: 5000.
    /// If limit > 5000, only 5000 entries will be returned.
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    last_update_id: u64,
    bids: Vec<OrderLevel>,
    asks: Vec<OrderLevel>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct OrderLevel(Decimal, Decimal);

impl OrderLevel {
    pub fn price(&self) -> Decimal {
        self.0
    }
    pub fn qty(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetRecentTradesParams {
    pub symbol: String,
    /// Default: 500; Maximum: 1000.
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecentTrade {
    id: u64,
    price: Decimal,
    qty: Decimal,
    quote_qty: Decimal,
    time: Timestamp,
    is_buyer_maker: bool,
    is_best_match: bool,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOlderTradesParams {
    pub symbol: String,
    /// Default: 500; Maximum: 1000.
    pub limit: Option<u64>,
    /// TradeId to fetch from. Default gets most recent trades.
    pub from_id: Option<u64>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAggregateTradesParams {
    pub symbol: String,
    /// ID to get aggregate trades from INCLUSIVE.
    pub from_id: Option<u64>,
    /// Timestamp in ms to get aggregate trades from INCLUSIVE.
    pub start_time: Option<Timestamp>,
    /// Timestamp in ms to get aggregate trades until INCLUSIVE.
    pub end_time: Option<Timestamp>,
    /// Default: 500; Maximum: 1000.
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AggregateTrade {
    /// Aggregate tradeId
    #[serde(rename = "a")]
    id: u64,
    /// Price
    #[serde(rename = "p")]
    price: Decimal,
    /// Quantity
    #[serde(rename = "q")]
    qty: Decimal,
    /// First tradeId
    #[serde(rename = "f")]
    first_trade_id: u64,
    /// Last tradeId
    #[serde(rename = "l")]
    last_trade_id: u64,
    /// Timestamp
    #[serde(rename = "T")]
    time: Timestamp,
    /// Was the buyer the maker?
    #[serde(rename = "m")]
    is_buyer_maker: bool,
    /// Was the trade the best price match?
    #[serde(rename = "M")]
    is_best_match: bool,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetKlineListParams {
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub time_zone: Option<String>,
    /// Default: 500; Maximum: 1000.
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Kline(
    Timestamp, // Kline open time
    Decimal,   // Open price
    Decimal,   // High price
    Decimal,   // Low price
    Decimal,   // Close price
    Decimal,   // Volume
    Timestamp, // Kline Close time
    Decimal,   // Quote asset volume
    u64,       // Number of trades
    Decimal,   // Taker buy base asset volume
    Decimal,   // Taker buy quote asset volume
    String,    // DEPRECATED: Unused field, ignore.
);

impl Kline {
    /// Kline open time
    pub fn time_open(&self) -> Timestamp {
        self.0
    }
    /// Open price
    pub fn open(&self) -> Decimal {
        self.1
    }
    /// High price
    pub fn high(&self) -> Decimal {
        self.2
    }
    /// Low price
    pub fn low(&self) -> Decimal {
        self.3
    }
    /// Close price
    pub fn close(&self) -> Decimal {
        self.4
    }
    /// Volume
    pub fn volume(&self) -> Decimal {
        self.5
    }
    /// Kline Close time
    pub fn time_close(&self) -> Timestamp {
        self.6
    }
    /// Quote asset volume
    pub fn quote_asset_volume(&self) -> Decimal {
        self.7
    }
    /// Number of trades
    pub fn id(&self) -> u64 {
        self.8
    }
    /// Taker buy base asset volume
    pub fn taker_buy_base_asset_volume(&self) -> Decimal {
        self.9
    }
    /// Taker buy quote asset volume
    pub fn taker_buy_quote_asset_volume(&self) -> Decimal {
        self.10
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetCurrentAveragePriceParams {
    pub symbol: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentAveragePrice {
    /// Average price interval (in minutes)
    pub mins: u64,
    /// Average price
    pub price: Decimal,
    /// Last trade time
    pub close_time: Timestamp,
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::spot::serde::deserialize_str;

    use super::*;

    #[test]
    fn deserialize_response_exchange_info() {
        let json = r#"{
            "timezone": "UTC",
            "serverTime": 1565246363776,
            "rateLimits": [],
            "exchangeFilters": [],
            "symbols": [
                {
                    "symbol": "ETHBTC",
                    "status": "TRADING",
                    "baseAsset": "ETH",
                    "baseAssetPrecision": 8,
                    "quoteAsset": "BTC",
                    "quotePrecision": 8,
                    "quoteAssetPrecision": 8,
                    "baseCommissionPrecision": 8,
                    "quoteCommissionPrecision": 8,
                    "orderTypes": [
                        "LIMIT",
                        "LIMIT_MAKER",
                        "MARKET",
                        "STOP_LOSS",
                        "STOP_LOSS_LIMIT",
                        "TAKE_PROFIT",
                        "TAKE_PROFIT_LIMIT"
                    ],
                    "icebergAllowed": true,
                    "ocoAllowed": true,
                    "otoAllowed": true,
                    "quoteOrderQtyMarketAllowed": true,
                    "allowTrailingStop": false,
                    "cancelReplaceAllowed":false,
                    "amendAllowed":false,
                    "isSpotTradingAllowed": true,
                    "isMarginTradingAllowed": true,
                    "filters": [],
                    "permissions": [],
                    "permissionSets": [
                        [
                            "SPOT",
                            "MARGIN"
                        ]
                    ],
                    "defaultSelfTradePreventionMode": "NONE",
                    "allowedSelfTradePreventionModes": [
                        "NONE"
                    ]
                }
            ],
            "sors": [
                {
                    "baseAsset": "BTC",
                    "symbols": [
                        "BTCUSDT",
                        "BTCUSDC"
                    ]
                }
            ]
        }"#;
        let expected = ExchangeInfo {
            timezone: String::from("UTC"),
            server_time: 1565246363776,
            rate_limits: vec![],
            exchange_filters: vec![],
            symbols: vec![SymbolInfo {
                symbol: String::from("ETHBTC"),
                status: SymbolStatus::Trading,
                base_asset: String::from("ETH"),
                base_asset_precision: 8,
                quote_asset: String::from("BTC"),
                quote_asset_precision: 8,
                base_commission_precision: 8,
                quote_commission_precision: 8,
                order_types: vec![
                    OrderType::Limit,
                    OrderType::LimitMaker,
                    OrderType::Market,
                    OrderType::StopLoss,
                    OrderType::StopLossLimit,
                    OrderType::TakeProfit,
                    OrderType::TakeProfitLimit,
                ],
                iceberg_allowed: true,
                oco_allowed: true,
                oto_allowed: true,
                quote_order_qty_market_allowed: true,
                allow_trailing_stop: false,
                cancel_replace_allowed: false,
                amend_allowed: false,
                is_spot_trading_allowed: true,
                is_margin_trading_allowed: true,
                filters: vec![],
                permissions: vec![],
                permission_sets: vec![vec![String::from("SPOT"), String::from("MARGIN")]],
                default_self_trade_prevention_mode: STPMode::None,
                allowed_self_trade_prevention_modes: vec![STPMode::None],
            }],
            sors: Some(vec![SOR {
                base_asset: String::from("BTC"),
                symbols: vec![String::from("BTCUSDT"), String::from("BTCUSDC")],
            }]),
        };

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_order_book() {
        let json = r#"{
            "lastUpdateId": 1027024,
            "bids": [
                [
                "4.00000000",
                "431.00000000"
                ]
            ],
            "asks": [
                [
                "4.00000200",
                "12.00000000"
                ]
            ]
        }"#;
        let expected = OrderBook {
            last_update_id: 1027024,
            bids: vec![OrderLevel(dec!(4.00000000), dec!(431.00000000))],
            asks: vec![OrderLevel(dec!(4.00000200), dec!(12.00000000))],
        };

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }
}
