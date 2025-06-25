use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::spot::{
    ErrorCode, ExchangeFilter, KlineInterval, OrderResponseType, OrderSide, OrderStatus, OrderType,
    RateLimitInterval, RateLimiter, STPMode, SymbolStatus, TimeInForce, WorkingFloor,
};

pub type Timestamp = u128;

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
pub struct ResponseError {
    pub code: ErrorCode,
    pub msg: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TestConnectivity {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: Timestamp,
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
    pub timezone: String,
    pub server_time: Timestamp,
    pub rate_limits: Vec<RateLimit>,
    pub exchange_filters: Vec<ExchangeFilter>,
    pub symbols: Vec<SymbolInfo>,
    /// Optional field. Present only when SOR is available.
    /// LINK: https://github.com/binance/binance-spot-api-docs/blob/master/faqs/sor_faq.md
    pub sors: Option<Vec<SOR>>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: RateLimiter,
    pub interval: RateLimitInterval,
    pub interval_num: u64,
    pub limit: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: SymbolStatus,
    pub base_asset: String,
    pub base_asset_precision: u8, // value range: [0:8]
    pub quote_asset: String,
    // INFO: 'quote_precision' will be removed in future api versions (v4+)
    pub quote_asset_precision: u8,      // value range: [0:8]
    pub base_commission_precision: u8,  // value range: [0:8]
    pub quote_commission_precision: u8, // value range: [0:8]
    pub order_types: Vec<OrderType>,
    pub iceberg_allowed: bool,
    pub oco_allowed: bool,
    pub oto_allowed: bool,
    pub quote_order_qty_market_allowed: bool,
    pub allow_trailing_stop: bool,
    pub cancel_replace_allowed: bool,
    pub amend_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<Filter>,
    pub permissions: Vec<String>,
    pub permission_sets: Vec<Vec<String>>,
    pub default_self_trade_prevention_mode: STPMode,
    pub allowed_self_trade_prevention_modes: Vec<STPMode>,
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
    pub base_asset: String,
    pub symbols: Vec<String>,
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
    pub last_update_id: i64,
    pub bids: Vec<OrderLevel>,
    pub asks: Vec<OrderLevel>,
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
    pub id: i64,
    pub price: Decimal,
    pub qty: Decimal,
    pub quote_qty: Decimal,
    pub time: Timestamp,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOlderTradesParams {
    pub symbol: String,
    /// Default: 500; Maximum: 1000.
    pub limit: Option<u64>,
    /// TradeId to fetch from. Default gets most recent trades.
    pub from_id: Option<i64>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAggregateTradesParams {
    pub symbol: String,
    /// ID to get aggregate trades from INCLUSIVE.
    pub from_id: Option<i64>,
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
    pub id: i64,
    /// Price
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity
    #[serde(rename = "q")]
    pub qty: Decimal,
    /// First tradeId
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    /// Last tradeId
    #[serde(rename = "l")]
    pub last_trade_id: i64,
    /// Timestamp
    #[serde(rename = "T")]
    pub time: Timestamp,
    /// Was the buyer the maker?
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    /// Was the trade the best price match?
    #[serde(rename = "M")]
    pub is_best_match: bool,
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

/// Supported values: FULL or MINI.
/// If none provided, the default is FULL
#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GetTickerPriceChangeStatisticsParams {
    Mini(SymbolOrSymbols),
    Full(SymbolOrSymbols),
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SymbolOrSymbols {
    /// Parameter symbol and symbols cannot be used in combination.
    /// If neither parameter is sent, tickers for all symbols will be returned in an array.
    pub symbol: Option<String>,
    /// Examples of accepted format for the symbols parameter: ["BTCUSDT","BNBUSDT"]
    /// TODO: check serialization
    /// or
    /// %5B%22BTCUSDT%22,%22BNBUSDT%22%5D
    pub symbols: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TickerPriceChangeStatistic {
    MiniElement(TickerPriceChangeStatisticMini),
    MiniList(Vec<TickerPriceChangeStatisticMini>),
    FullElement(TickerPriceChangeStatisticFull),
    FullList(Vec<TickerPriceChangeStatisticFull>),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TickerPriceChangeStatisticFull {
    pub symbol: String,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub weighted_avg_price: Decimal,
    pub prev_close_price: Decimal,
    pub last_price: Decimal,
    pub last_qty: Decimal,
    pub bid_price: Decimal,
    pub bid_qty: Decimal,
    pub ask_price: Decimal,
    pub ask_qty: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub open_time: Timestamp,
    pub close_time: Timestamp,
    /// First traded
    pub first_id: i64,
    /// Last traded
    pub last_id: i64,
    /// Trade count
    pub count: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TickerPriceChangeStatisticMini {
    /// Symbol Name
    pub symbol: String,
    /// Opening price of the Interval
    pub open_price: Decimal,
    /// Highest price in the interval
    pub high_price: Decimal,
    /// Lowest  price in the interval
    pub low_price: Decimal,
    /// Closing price of the interval
    pub last_price: Decimal,
    /// Total trade volume (in base asset)
    pub volume: Decimal,
    /// Total trade volume (in quote asset)
    pub quote_volume: Decimal,
    /// Start of the ticker interval
    pub open_time: Timestamp,
    /// End of the ticker interval
    pub close_time: Timestamp,
    /// First tradeId considered
    pub first_id: i64,
    /// Last tradeId considered
    pub last_id: i64,
    /// Total trade count
    pub count: u64,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderParams {
    pub symbol: String,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub time_in_force: Option<TimeInForce>,
    pub quantity: Option<Decimal>,
    pub quote_order_qty: Option<Decimal>,
    pub price: Option<Decimal>,
    /// A unique id among open orders. Automatically generated if not sent.
    /// Orders with the same newClientOrderID can be accepted only when the previous one is filled, otherwise the order will be rejected.
    pub new_client_order_id: Option<String>,
    pub strategy_id: Option<i64>,
    /// The value cannot be less than 1000000.
    pub strategy_type: Option<i64>,
    /// Used with STOP_LOSS, STOP_LOSS_LIMIT, TAKE_PROFIT, and TAKE_PROFIT_LIMIT orders.
    pub stop_price: Option<Decimal>,
    /// See Trailing Stop order FAQ.
    pub trailing_delta: Option<i64>,
    /// Used with LIMIT, STOP_LOSS_LIMIT, and TAKE_PROFIT_LIMIT to create an iceberg order.
    pub iceberg_qty: Option<Decimal>,
    /// Set the response JSON. ACK, RESULT, or FULL; MARKET and LIMIT order types default to FULL, all other orders default to ACK.
    /// Mandatory - because there is a problem with deserialization of untagged enum Order
    pub new_order_resp_type: OrderResponseType,
    /// The allowed enums is dependent on what is configured on the symbol. The possible supported values are: STP Modes.
    pub self_trade_prevention_mode: Option<STPMode>,
    /// The value cannot be greater than 60000
    pub recv_window: Option<i64>,
    pub timestamp: Timestamp,
}

impl NewOrderParams {
    pub fn validate(&self) -> bool {
        match self.order_type {
            OrderType::Limit => {
                self.time_in_force.is_some() && self.quantity.is_some() && self.price.is_some()
            }
            OrderType::Market => {
                // MARKET orders using the quantity field specifies the amount of the base asset the user wants to buy or sell at the market price.
                // E.g. MARKET order on BTCUSDT will specify how much BTC the user is buying or selling.

                // MARKET orders using quoteOrderQty specifies the amount the user wants to spend (when buying) or receive (when selling) the quote asset; the correct quantity will be determined based on the market liquidity and quoteOrderQty.
                // E.g. Using the symbol BTCUSDT:
                // BUY side, the order will buy as many BTC as quoteOrderQty USDT can.
                // SELL side, the order will sell as much BTC needed to receive quoteOrderQty USDT.
                self.quantity.is_some() || self.quote_order_qty.is_some()
            }
            OrderType::StopLoss => {
                // This will execute a MARKET order when the conditions are met. (e.g. stopPrice is met or trailingDelta is activated)
                self.quantity.is_some()
                    && (self.stop_price.is_some() || self.trailing_delta.is_some())
            }
            OrderType::StopLossLimit => {
                self.time_in_force.is_some()
                    && self.quantity.is_some()
                    && self.price.is_some()
                    && (self.stop_price.is_some() || self.trailing_delta.is_some())
            }
            OrderType::TakeProfit => {
                // This will execute a MARKET order when the conditions are met. (e.g. stopPrice is met or trailingDelta is activated)
                self.quantity.is_some()
                    && (self.stop_price.is_some() || self.trailing_delta.is_some())
            }
            OrderType::TakeProfitLimit => {
                self.time_in_force.is_some()
                    && self.quantity.is_some()
                    && self.price.is_some()
                    && (self.stop_price.is_some() || self.trailing_delta.is_some())
            }
            OrderType::LimitMaker => {
                // This is a LIMIT order that will be rejected if the order immediately matches and trades as a taker.
                // This is also known as a POST-ONLY order.
                self.quantity.is_some() && self.price.is_some()
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Order {
    Ack(OrderAck),
    Result(OrderResult),
    Full(OrderFull),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderAck {
    pub symbol: String,
    pub order_id: i64,
    /// Unless it's part of an order list, value will be -1
    pub order_list_id: i64,
    pub client_order_id: String,
    pub transact_time: Timestamp,
    /// Quantity for the iceberg order
    /// Appears only if the parameter icebergQty was sent in the request.
    pub iceberg_qty: Option<Decimal>,
    /// When used in combination with symbol, can be used to query a prevented match.
    /// Appears only if the order expired due to STP.
    pub prevented_match_id: Option<i64>,
    /// Order quantity that expired due to STP
    /// Appears only if the order expired due to STP.
    pub prevented_quantity: Option<Decimal>,
    /// Price when the algorithmic order will be triggered
    /// Appears for STOP_LOSS. TAKE_PROFIT, STOP_LOSS_LIMIT and TAKE_PROFIT_LIMIT orders.
    pub stop_price: Option<Decimal>,
    /// Can be used to label an order that's part of an order strategy.
    /// Appears if the parameter was populated in the request.
    pub strategy_id: Option<i64>,
    /// Can be used to label an order that is using an order strategy.
    /// Appears if the parameter was populated in the request.
    pub strategy_type: Option<i64>,
    /// Delta price change required before order activation
    /// Appears for Trailing Stop Orders.
    pub trailing_delta: Option<i64>,
    /// Time when the trailing order is now active and tracking price changes
    /// Appears only for Trailing Stop Orders.
    pub trailing_time: Option<i64>,
    /// Field that determines whether order used SOR
    /// Appears when placing orders using SOR
    pub used_sor: Option<bool>,
    /// Field that determines whether the order is being filled by the SOR or by the order book the order was submitted to.
    /// Appears when placing orders using SOR
    pub working_floor: Option<WorkingFloor>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderResult {
    pub symbol: String,
    pub order_id: i64,
    /// Unless it's part of an order list, value will be -1
    pub order_list_id: i64,
    pub client_order_id: String,
    pub transact_time: Timestamp,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub orig_quote_order_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub working_time: Timestamp,
    pub self_trade_prevention_mode: STPMode,
    /// Quantity for the iceberg order
    /// Appears only if the parameter icebergQty was sent in the request.
    pub iceberg_qty: Option<Decimal>,
    /// When used in combination with symbol, can be used to query a prevented match.
    /// Appears only if the order expired due to STP.
    pub prevented_match_id: Option<i64>,
    /// Order quantity that expired due to STP
    /// Appears only if the order expired due to STP.
    pub prevented_quantity: Option<Decimal>,
    /// Price when the algorithmic order will be triggered
    /// Appears for STOP_LOSS. TAKE_PROFIT, STOP_LOSS_LIMIT and TAKE_PROFIT_LIMIT orders.
    pub stop_price: Option<Decimal>,
    /// Can be used to label an order that's part of an order strategy.
    /// Appears if the parameter was populated in the request.
    pub strategy_id: Option<i64>,
    /// Can be used to label an order that is using an order strategy.
    /// Appears if the parameter was populated in the request.
    pub strategy_type: Option<i64>,
    /// Delta price change required before order activation
    /// Appears for Trailing Stop Orders.
    pub trailing_delta: Option<i64>,
    /// Time when the trailing order is now active and tracking price changes
    /// Appears only for Trailing Stop Orders.
    pub trailing_time: Option<i64>,
    /// Field that determines whether order used SOR
    /// Appears when placing orders using SOR
    pub used_sor: Option<bool>,
    /// Field that determines whether the order is being filled by the SOR or by the order book the order was submitted to.
    /// Appears when placing orders using SOR
    pub working_floor: Option<WorkingFloor>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderFull {
    pub symbol: String,
    pub order_id: i64,
    /// Unless it's part of an order list, value will be -1
    pub order_list_id: i64,
    pub client_order_id: String,
    pub transact_time: Timestamp,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub orig_quote_order_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub working_time: Timestamp,
    pub self_trade_prevention_mode: STPMode,
    pub fills: Vec<OrderFill>,
    /// Quantity for the iceberg order
    /// Appears only if the parameter icebergQty was sent in the request.
    pub iceberg_qty: Option<Decimal>,
    /// When used in combination with symbol, can be used to query a prevented match.
    /// Appears only if the order expired due to STP.
    pub prevented_match_id: Option<i64>,
    /// Order quantity that expired due to STP
    /// Appears only if the order expired due to STP.
    pub prevented_quantity: Option<Decimal>,
    /// Price when the algorithmic order will be triggered
    /// Appears for STOP_LOSS. TAKE_PROFIT, STOP_LOSS_LIMIT and TAKE_PROFIT_LIMIT orders.
    pub stop_price: Option<Decimal>,
    /// Can be used to label an order that's part of an order strategy.
    /// Appears if the parameter was populated in the request.
    pub strategy_id: Option<i64>,
    /// Can be used to label an order that is using an order strategy.
    /// Appears if the parameter was populated in the request.
    pub strategy_type: Option<i64>,
    /// Delta price change required before order activation
    /// Appears for Trailing Stop Orders.
    pub trailing_delta: Option<i64>,
    /// Time when the trailing order is now active and tracking price changes
    /// Appears only for Trailing Stop Orders.
    pub trailing_time: Option<i64>,
    /// Field that determines whether order used SOR
    /// Appears when placing orders using SOR
    pub used_sor: Option<bool>,
    /// Field that determines whether the order is being filled by the SOR or by the order book the order was submitted to.
    /// Appears when placing orders using SOR
    pub working_floor: Option<WorkingFloor>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderFill {
    pub price: Decimal,
    pub qty: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    pub trade_id: i64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CommissionRates {
    Empty(CommissionRatesEmpty),
    Full(CommissionRatesFull),
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CommissionRatesEmpty {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRatesFull {
    /// Standard commission rates on trades from the order.
    pub standard_commission_for_order: CommissionForOrder,
    /// Tax commission rates for trades from the order.
    pub tax_commission_for_order: CommissionForOrder,
    /// Discount on standard commissions when paying in BNB.
    pub discount: Discount,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommissionForOrder {
    pub maker: Decimal,
    pub taker: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Discount {
    pub enabled_for_account: bool,
    pub enabled_for_symbol: bool,
    pub discount_asset: String,
    /// Standard commission is reduced by this rate when paying commission in BNB.
    pub discount: Decimal,
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

    #[test]
    fn deserialize_response_order_ack() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "orderId": 28,
            "orderListId": -1,
            "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
            "transactTime": 1507725176595
        }"#;
        let expected = Order::Ack(OrderAck {
            symbol: String::from("BTCUSDT"),
            order_id: 28,
            order_list_id: -1,
            client_order_id: String::from("6gCrw2kRUAF9CvJDGP16IP"),
            transact_time: 1507725176595,
            iceberg_qty: None,
            prevented_match_id: None,
            prevented_quantity: None,
            stop_price: None,
            strategy_id: None,
            strategy_type: None,
            trailing_delta: None,
            trailing_time: None,
            used_sor: None,
            working_floor: None,
        });

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_order_result() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "orderId": 28,
            "orderListId": -1,
            "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
            "transactTime": 1507725176595,
            "price": "0.00000000",
            "origQty": "10.00000000",
            "executedQty": "10.00000000",
            "origQuoteOrderQty": "0.000000",
            "cummulativeQuoteQty": "10.00000000",
            "status": "FILLED",
            "timeInForce": "GTC",
            "type": "MARKET",
            "side": "SELL",
            "workingTime": 1507725176595,
            "selfTradePreventionMode": "NONE"
        }"#;
        // INFO: not work: Order::Result(OrderResult {})
        let expected = OrderResult {
            symbol: String::from("BTCUSDT"),
            order_id: 28,
            order_list_id: -1,
            client_order_id: String::from("6gCrw2kRUAF9CvJDGP16IP"),
            transact_time: 1507725176595,
            price: dec!(0.00000000),
            orig_qty: dec!(10.00000000),
            executed_qty: dec!(10.00000000),
            orig_quote_order_qty: dec!(0.00000000),
            cummulative_quote_qty: dec!(10.00000000),
            status: OrderStatus::Filled,
            time_in_force: TimeInForce::GTC,
            order_type: OrderType::Market,
            side: OrderSide::SELL,
            working_time: 1507725176595,
            self_trade_prevention_mode: STPMode::None,
            iceberg_qty: None,
            prevented_match_id: None,
            prevented_quantity: None,
            stop_price: None,
            strategy_id: None,
            strategy_type: None,
            trailing_delta: None,
            trailing_time: None,
            used_sor: None,
            working_floor: None,
        };

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_order_full() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "orderId": 28,
            "orderListId": -1,
            "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
            "transactTime": 1507725176595,
            "price": "0.00000000",
            "origQty": "10.00000000",
            "executedQty": "10.00000000",
            "origQuoteOrderQty": "0.000000",
            "cummulativeQuoteQty": "10.00000000",
            "status": "FILLED",
            "timeInForce": "GTC",
            "type": "MARKET",
            "side": "SELL",
            "workingTime": 1507725176595,
            "selfTradePreventionMode": "NONE",
            "fills": [
                {
                    "price": "4000.00000000",
                    "qty": "1.00000000",
                    "commission": "4.00000000",
                    "commissionAsset": "USDT",
                    "tradeId": 56
                },
                {
                    "price": "3999.00000000",
                    "qty": "5.00000000",
                    "commission": "19.99500000",
                    "commissionAsset": "USDT",
                    "tradeId": 57
                },
                {
                    "price": "3998.00000000",
                    "qty": "2.00000000",
                    "commission": "7.99600000",
                    "commissionAsset": "USDT",
                    "tradeId": 58
                },
                {
                    "price": "3997.00000000",
                    "qty": "1.00000000",
                    "commission": "3.99700000",
                    "commissionAsset": "USDT",
                    "tradeId": 59
                },
                {
                    "price": "3995.00000000",
                    "qty": "1.00000000",
                    "commission": "3.99500000",
                    "commissionAsset": "USDT",
                    "tradeId": 60
                }
            ]
        }"#;
        // INFO: not work: Order::Full(OrderFull {})
        let expected = OrderFull {
            symbol: String::from("BTCUSDT"),
            order_id: 28,
            order_list_id: -1,
            client_order_id: String::from("6gCrw2kRUAF9CvJDGP16IP"),
            transact_time: 1507725176595,
            price: dec!(0.00000000),
            orig_qty: dec!(10.00000000),
            executed_qty: dec!(10.00000000),
            orig_quote_order_qty: dec!(0.00000000),
            cummulative_quote_qty: dec!(10.00000000),
            status: OrderStatus::Filled,
            time_in_force: TimeInForce::GTC,
            order_type: OrderType::Market,
            side: OrderSide::SELL,
            working_time: 1507725176595,
            self_trade_prevention_mode: STPMode::None,
            fills: vec![
                OrderFill {
                    price: dec!(4000.00000000),
                    qty: dec!(1.00000000),
                    commission: dec!(4.00000000),
                    commission_asset: String::from("USDT"),
                    trade_id: 56,
                },
                OrderFill {
                    price: dec!(3999.00000000),
                    qty: dec!(5.00000000),
                    commission: dec!(19.99500000),
                    commission_asset: String::from("USDT"),
                    trade_id: 57,
                },
                OrderFill {
                    price: dec!(3998.00000000),
                    qty: dec!(2.00000000),
                    commission: dec!(7.99600000),
                    commission_asset: String::from("USDT"),
                    trade_id: 58,
                },
                OrderFill {
                    price: dec!(3997.00000000),
                    qty: dec!(1.00000000),
                    commission: dec!(3.99700000),
                    commission_asset: String::from("USDT"),
                    trade_id: 59,
                },
                OrderFill {
                    price: dec!(3995.00000000),
                    qty: dec!(1.00000000),
                    commission: dec!(3.99500000),
                    commission_asset: String::from("USDT"),
                    trade_id: 60,
                },
            ],
            iceberg_qty: None,
            prevented_match_id: None,
            prevented_quantity: None,
            stop_price: None,
            strategy_id: None,
            strategy_type: None,
            trailing_delta: None,
            trailing_time: None,
            used_sor: None,
            working_floor: None,
        };

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_test_order_commission_rates_empty() {
        let json = r#"{}"#;
        let expected = CommissionRates::Empty(CommissionRatesEmpty {});

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_test_order_commission_rates_full() {
        let json = r#"{
            "standardCommissionForOrder": {
                "maker": "0.00000112",
                "taker": "0.00000114"
            },
            "taxCommissionForOrder": {
                "maker": "0.00000112",
                "taker": "0.00000114"
            },
            "discount": {
                "enabledForAccount": true,
                "enabledForSymbol": true,
                "discountAsset": "BNB",
                "discount": "0.25000000"
            }
        }"#;
        // INFO: not working: let expected = CommissionRates::Full(CommissionRatesFull {})
        let expected = CommissionRatesFull {
            standard_commission_for_order: CommissionForOrder {
                maker: dec!(0.00000112),
                taker: dec!(0.00000114),
            },
            tax_commission_for_order: CommissionForOrder {
                maker: dec!(0.00000112),
                taker: dec!(0.00000114),
            },
            discount: Discount {
                enabled_for_account: true,
                enabled_for_symbol: true,
                discount_asset: String::from("BNB"),
                discount: dec!(0.25000000),
            },
        };

        let current = deserialize_str(json).unwrap();

        assert_eq!(expected, current);
    }
}
