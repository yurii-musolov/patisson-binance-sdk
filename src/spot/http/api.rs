use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    serde::serialize_option_as_json,
    spot::{
        AccountType, ExchangeFilter, KlineInterval, OrderResponseType, OrderSide, OrderStatus,
        OrderType, RateLimitInterval, RateLimiter, STPMode, SymbolStatus, TimeInForce,
        WorkingFloor,
    },
};

pub use crate::http::ParsedHeaders as Headers;

#[derive(Debug, PartialEq)]
pub struct Response<T> {
    pub result: T,
    pub headers: Headers,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TestConnectivity {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: Timestamp,
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangeInfoParams {
    /// Single symbol: `?symbol=BNBBTC`. Cannot be combined with `symbols`.
    symbol: Option<String>,
    /// Multiple symbols, sent as a JSON-array literal:
    /// `?symbols=["BTCUSDT","BNBBTC"]` (URL-encoded). Cannot be combined with `symbol`.
    #[serde(serialize_with = "serialize_option_as_json")]
    symbols: Option<Vec<String>>,
    /// Permission filter, sent as a JSON-array literal:
    /// `?permissions=["MARGIN","LEVERAGED"]` (URL-encoded). A single
    /// permission may also be sent as a one-element vec.
    #[serde(serialize_with = "serialize_option_as_json")]
    permissions: Option<Vec<String>>,
    /// Controls whether the content of the permissionSets field is populated or not. Defaults to true
    show_permission_sets: Option<bool>,
    /// Filters symbols that have this tradingStatus. Valid values: TRADING, HALT, BREAK
    /// Cannot be used in combination with symbols or symbol.
    symbol_status: Option<SymbolStatus>,
}

impl GetExchangeInfoParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = Some(symbols);
        self
    }

    pub fn permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    pub fn show_permission_sets(mut self, value: bool) -> Self {
        self.show_permission_sets = Some(value);
        self
    }

    pub fn symbol_status(mut self, value: SymbolStatus) -> Self {
        self.symbol_status = Some(value);
        self
    }
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

/// Per-symbol trading filters from `/api/v3/exchangeInfo`.
///
/// Reference: <https://developers.binance.com/docs/binance-spot-api-docs/filters>
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "filterType")]
pub enum Filter {
    #[serde(rename = "PRICE_FILTER", rename_all = "camelCase")]
    PriceFilter {
        min_price: Decimal,
        max_price: Decimal,
        tick_size: Decimal,
    },
    #[serde(rename = "PERCENT_PRICE_BY_SIDE", rename_all = "camelCase")]
    PercentPriceBySide {
        bid_multiplier_up: Decimal,
        bid_multiplier_down: Decimal,
        ask_multiplier_up: Decimal,
        ask_multiplier_down: Decimal,
        avg_price_mins: u64,
    },
    #[serde(rename = "LOT_SIZE", rename_all = "camelCase")]
    LotSize {
        min_qty: Decimal,
        max_qty: Decimal,
        step_size: Decimal,
    },
    #[serde(rename = "MIN_NOTIONAL", rename_all = "camelCase")]
    MinNotional {
        min_notional: Decimal,
        apply_to_market: bool,
        avg_price_mins: u64,
    },
    #[serde(rename = "NOTIONAL", rename_all = "camelCase")]
    Notional {
        min_notional: Decimal,
        apply_min_to_market: bool,
        max_notional: Decimal,
        apply_max_to_market: bool,
        avg_price_mins: u64,
    },
    #[serde(rename = "ICEBERG_PARTS", rename_all = "camelCase")]
    IcebergParts { limit: u64 },
    #[serde(rename = "MARKET_LOT_SIZE", rename_all = "camelCase")]
    MarketLotSize {
        min_qty: Decimal,
        max_qty: Decimal,
        step_size: Decimal,
    },
    #[serde(rename = "MAX_NUM_ORDERS", rename_all = "camelCase")]
    MaxNumOrders { max_num_orders: u64 },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS", rename_all = "camelCase")]
    MaxNumAlgoOrders { max_num_algo_orders: u64 },
    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS", rename_all = "camelCase")]
    MaxNumIcebergOrders { max_num_iceberg_orders: u64 },
    #[serde(rename = "MAX_POSITION", rename_all = "camelCase")]
    MaxPosition { max_position: Decimal },
    #[serde(rename = "TRAILING_DELTA", rename_all = "camelCase")]
    TrailingDelta {
        min_trailing_above_delta: u64,
        max_trailing_above_delta: u64,
        min_trailing_below_delta: u64,
        max_trailing_below_delta: u64,
    },
    /// Catch-all for filter types not yet modelled. Lets new Binance filter
    /// types deserialize without breaking existing callers.
    #[serde(other)]
    Unknown,
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
    symbol: String,
    /// Default: 100; Maximum: 5000.
    /// If limit > 5000, only 5000 entries will be returned.
    pub(super) limit: Option<u64>,
}

impl GetOrderBookParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            limit: None,
        }
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: i64,
    pub bids: Vec<OrderLevel>,
    pub asks: Vec<OrderLevel>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct OrderLevel(Decimal, Decimal);

impl OrderLevel {
    pub fn price(&self) -> Decimal {
        self.0
    }
    pub fn qty(&self) -> Decimal {
        self.1
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetRecentTradesParams {
    symbol: String,
    /// Default: 500; Maximum: 1000.
    limit: Option<u64>,
}

impl GetRecentTradesParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            limit: None,
        }
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
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
    symbol: String,
    /// Default: 500; Maximum: 1000.
    limit: Option<u64>,
    /// TradeId to fetch from. Default gets most recent trades.
    from_id: Option<i64>,
}

impl GetOlderTradesParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            limit: None,
            from_id: None,
        }
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn from_id(mut self, from_id: i64) -> Self {
        self.from_id = Some(from_id);
        self
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAggregateTradesParams {
    symbol: String,
    /// ID to get aggregate trades from INCLUSIVE.
    from_id: Option<i64>,
    /// Timestamp in ms to get aggregate trades from INCLUSIVE.
    start_time: Option<Timestamp>,
    /// Timestamp in ms to get aggregate trades until INCLUSIVE.
    end_time: Option<Timestamp>,
    /// Default: 500; Maximum: 1000.
    limit: Option<u64>,
}

impl GetAggregateTradesParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            from_id: None,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }

    pub fn from_id(mut self, from_id: i64) -> Self {
        self.from_id = Some(from_id);
        self
    }

    pub fn start_time(mut self, start_time: Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    pub fn end_time(mut self, end_time: Timestamp) -> Self {
        self.end_time = Some(end_time);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
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
    symbol: String,
    interval: KlineInterval,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    time_zone: Option<String>,
    /// Default: 500; Maximum: 1000.
    limit: Option<u64>,
}

impl GetKlineListParams {
    pub fn new(symbol: impl Into<String>, interval: KlineInterval) -> Self {
        Self {
            symbol: symbol.into(),
            interval,
            start_time: None,
            end_time: None,
            time_zone: None,
            limit: None,
        }
    }

    pub fn start_time(mut self, start_time: Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    pub fn end_time(mut self, end_time: Timestamp) -> Self {
        self.end_time = Some(end_time);
        self
    }

    pub fn time_zone(mut self, time_zone: impl Into<String>) -> Self {
        self.time_zone = Some(time_zone.into());
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
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
    symbol: String,
}

impl GetCurrentAveragePriceParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
        }
    }
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

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct SymbolOrSymbols {
    /// Parameter symbol and symbols cannot be used in combination.
    /// If neither parameter is sent, tickers for all symbols will be returned in an array.
    symbol: Option<String>,
    /// Examples of accepted format for the symbols parameter: ["BTCUSDT","BNBUSDT"]
    /// TODO: check serialization
    /// or
    /// %5B%22BTCUSDT%22,%22BNBUSDT%22%5D
    symbols: Option<Vec<String>>,
}

impl SymbolOrSymbols {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = Some(symbols);
        self
    }
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
pub struct NewOrderRequest {
    symbol: String,
    side: OrderSide,
    #[serde(rename = "type")]
    r#type: OrderType,
    time_in_force: Option<TimeInForce>,
    quantity: Option<Decimal>,
    quote_order_qty: Option<Decimal>,
    price: Option<Decimal>,
    /// A unique id among open orders. Automatically generated if not sent.
    /// Orders with the same newClientOrderID can be accepted only when the previous one is filled, otherwise the order will be rejected.
    new_client_order_id: Option<String>,
    strategy_id: Option<i64>,
    /// The value cannot be less than 1000000.
    strategy_type: Option<i64>,
    /// Used with STOP_LOSS, STOP_LOSS_LIMIT, TAKE_PROFIT, and TAKE_PROFIT_LIMIT orders.
    stop_price: Option<Decimal>,
    /// See Trailing Stop order FAQ.
    trailing_delta: Option<i64>,
    /// Used with LIMIT, STOP_LOSS_LIMIT, and TAKE_PROFIT_LIMIT to create an iceberg order.
    iceberg_qty: Option<Decimal>,
    /// Set the response JSON. ACK, RESULT, or FULL; MARKET and LIMIT order types default to FULL, all other orders default to ACK.
    /// Mandatory - because there is a problem with deserialization of untagged enum Order
    new_order_resp_type: OrderResponseType,
    /// The allowed enums is dependent on what is configured on the symbol. The possible supported values are: STP Modes.
    self_trade_prevention_mode: Option<STPMode>,
    /// The value cannot be greater than 60000
    recv_window: Option<u64>,
    /// Only for test endpoint to place a new order.
    compute_commission_rates: Option<bool>,
}

impl NewOrderRequest {
    pub fn new(
        symbol: impl Into<String>,
        side: OrderSide,
        order_type: OrderType,
        new_order_resp_type: OrderResponseType,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            r#type: order_type,
            new_order_resp_type,
            time_in_force: None,
            quantity: None,
            quote_order_qty: None,
            price: None,
            new_client_order_id: None,
            strategy_id: None,
            strategy_type: None,
            stop_price: None,
            trailing_delta: None,
            iceberg_qty: None,
            self_trade_prevention_mode: None,
            recv_window: None,
            compute_commission_rates: None,
        }
    }

    pub fn time_in_force(mut self, value: TimeInForce) -> Self {
        self.time_in_force = Some(value);
        self
    }

    pub fn quantity(mut self, value: Decimal) -> Self {
        self.quantity = Some(value);
        self
    }

    pub fn quote_order_qty(mut self, value: Decimal) -> Self {
        self.quote_order_qty = Some(value);
        self
    }

    pub fn price(mut self, value: Decimal) -> Self {
        self.price = Some(value);
        self
    }

    pub fn new_client_order_id(mut self, value: impl Into<String>) -> Self {
        self.new_client_order_id = Some(value.into());
        self
    }

    pub fn strategy_id(mut self, value: i64) -> Self {
        self.strategy_id = Some(value);
        self
    }

    pub fn strategy_type(mut self, value: i64) -> Self {
        self.strategy_type = Some(value);
        self
    }

    pub fn stop_price(mut self, value: Decimal) -> Self {
        self.stop_price = Some(value);
        self
    }

    pub fn trailing_delta(mut self, value: i64) -> Self {
        self.trailing_delta = Some(value);
        self
    }

    pub fn iceberg_qty(mut self, value: Decimal) -> Self {
        self.iceberg_qty = Some(value);
        self
    }

    pub fn self_trade_prevention_mode(mut self, value: STPMode) -> Self {
        self.self_trade_prevention_mode = Some(value);
        self
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }

    pub fn compute_commission_rates(mut self, value: bool) -> Self {
        self.compute_commission_rates = Some(value);
        self
    }

    pub fn is_valid(&self) -> bool {
        match self.r#type {
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
pub enum NewOrderResponse {
    Full(NewOrderResponseFull),
    Result(NewOrderResponseResult),
    Ack(NewOrderResponseAck),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderResponseAck {
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
pub struct NewOrderResponseResult {
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
    pub r#type: OrderType,
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
pub struct NewOrderResponseFull {
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
    pub r#type: OrderType,
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
pub enum TestCommissionRates {
    Full(TestCommissionRatesFull),
    Empty(TestCommissionRatesEmpty),
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TestCommissionRatesEmpty {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestCommissionRatesFull {
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

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountInformationParams {
    /// When set to true, emits only the non-zero balances of an account.
    /// Default value: false
    omit_zero_balances: Option<bool>,
    /// The value cannot be greater than 60000
    recv_window: Option<u64>,
}

impl GetAccountInformationParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn omit_zero_balances(mut self, value: bool) -> Self {
        self.omit_zero_balances = Some(value);
        self
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    pub maker_commission: f64,
    pub taker_commission: f64,
    pub buyer_commission: f64,
    pub seller_commission: f64,
    pub commission_rates: CommissionRates,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub brokered: bool,
    pub require_self_trade_prevention: bool,
    pub prevent_sor: bool,
    pub update_time: Timestamp,
    pub account_type: AccountType,
    pub balances: Vec<Balance>,
    pub permissions: Option<Vec<String>>,
    pub permission_sets: Option<Vec<Vec<String>>>,
    pub uid: i64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRates {
    pub maker: Decimal,
    pub taker: Decimal,
    pub buyer: Decimal,
    pub seller: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryOrderParams {
    symbol: String,
    order_id: Option<i64>,
    orig_client_order_id: Option<String>,
    /// The value cannot be greater than 60000
    recv_window: Option<u64>,
}

impl QueryOrderParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            orig_client_order_id: None,
            recv_window: None,
        }
    }

    pub fn order_id(mut self, value: i64) -> Self {
        self.order_id = Some(value);
        self
    }

    pub fn orig_client_order_id(mut self, value: impl Into<String>) -> Self {
        self.orig_client_order_id = Some(value.into());
        self
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub symbol: String,
    pub order_id: i64,
    /// This field will always have a value of -1 if not an order list.
    pub order_list_id: i64,
    pub client_order_id: String,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub r#type: OrderType,
    pub side: OrderSide,
    /// Price when the algorithmic order will be triggered
    /// Appears for STOP_LOSS. TAKE_PROFIT, STOP_LOSS_LIMIT and TAKE_PROFIT_LIMIT orders.
    pub stop_price: Option<Decimal>,
    /// Quantity for the iceberg order
    /// Appears only if the parameter icebergQty was sent in the request.
    pub iceberg_qty: Option<Decimal>,
    pub time: Timestamp,
    pub update_time: Timestamp,
    pub is_working: bool,
    pub working_time: Timestamp,
    pub orig_quote_order_qty: Decimal,
    pub self_trade_prevention_mode: STPMode,
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::serde::deserialize_json;

    use super::*;

    #[test]
    fn deserialize_response_server_time() {
        let json = r#"{
            "serverTime": 1499827319559
        }"#;
        let expected = ServerTime {
            server_time: 1499827319559,
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

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

        let current = deserialize_json(json).unwrap();

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

        let current = deserialize_json(json).unwrap();

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
        let response = NewOrderResponseAck {
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
        };
        let expected = NewOrderResponse::Ack(response);

        let current = deserialize_json(json).unwrap();

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
        let response = NewOrderResponseResult {
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
            r#type: OrderType::Market,
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
        let expected = NewOrderResponse::Result(response);

        let current = deserialize_json(json).unwrap();

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
        let response = NewOrderResponseFull {
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
            r#type: OrderType::Market,
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
        let expected = NewOrderResponse::Full(response);

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_test_order_commission_rates_empty() {
        let json = r#"{}"#;
        let expected = TestCommissionRates::Empty(TestCommissionRatesEmpty {});

        let current = deserialize_json(json).unwrap();

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
        let rates = TestCommissionRatesFull {
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
        let expected = TestCommissionRates::Full(rates);

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_account_information() {
        let json = r#"{
            "makerCommission": 15,
            "takerCommission": 15,
            "buyerCommission": 0,
            "sellerCommission": 0,
            "commissionRates": {
                "maker": "0.00150000",
                "taker": "0.00150000",
                "buyer": "0.00000000",
                "seller": "0.00000000"
            },
            "canTrade": true,
            "canWithdraw": true,
            "canDeposit": true,
            "brokered": false,
            "requireSelfTradePrevention": false,
            "preventSor": false,
            "updateTime": 123456789,
            "accountType": "SPOT",
            "balances": [
                {
                "asset": "BTC",
                "free": "4723846.89208129",
                "locked": "0.00000000"
                },
                {
                "asset": "LTC",
                "free": "4763368.68006011",
                "locked": "0.00000000"
                }
            ],
            "permissions": [
                "SPOT"
            ],
            "uid": 354937868
        }"#;
        let expected = AccountInformation {
            maker_commission: 15.0,
            taker_commission: 15.0,
            buyer_commission: 0.0,
            seller_commission: 0.0,
            commission_rates: CommissionRates {
                maker: dec!(0.00150000),
                taker: dec!(0.00150000),
                buyer: dec!(0.00000000),
                seller: dec!(0.00000000),
            },
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            brokered: false,
            require_self_trade_prevention: false,
            prevent_sor: false,
            update_time: 123456789,
            account_type: AccountType::Spot,
            balances: vec![
                Balance {
                    asset: String::from("BTC"),
                    free: dec!(4723846.89208129),
                    locked: dec!(0.00000000),
                },
                Balance {
                    asset: String::from("LTC"),
                    free: dec!(4763368.68006011),
                    locked: dec!(0.00000000),
                },
            ],
            permissions: Some(vec![String::from("SPOT")]),
            permission_sets: None,
            uid: 354937868,
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }

    #[test]
    fn deserialize_response_query_order() {
        let json = r#"{
            "symbol": "LTCBTC",
            "orderId": 1,
            "orderListId": -1,
            "clientOrderId": "myOrder1",
            "price": "0.1",
            "origQty": "1.0",
            "executedQty": "0.0",
            "cummulativeQuoteQty": "0.0",
            "status": "NEW",
            "timeInForce": "GTC",
            "type": "LIMIT",
            "side": "BUY",
            "stopPrice": "0.0",
            "icebergQty": "0.0",
            "time": 1499827319559,
            "updateTime": 1499827319559,
            "isWorking": true,
            "workingTime":1499827319559,
            "origQuoteOrderQty": "0.000000",
            "selfTradePreventionMode": "NONE"
        }"#;
        let expected = Order {
            symbol: String::from("LTCBTC"),
            order_id: 1,
            order_list_id: -1,
            client_order_id: String::from("myOrder1"),
            price: dec!(0.1),
            orig_qty: dec!(1.0),
            executed_qty: dec!(0.0),
            cummulative_quote_qty: dec!(0.0),
            status: OrderStatus::New,
            time_in_force: TimeInForce::GTC,
            r#type: OrderType::Limit,
            side: OrderSide::BUY,
            stop_price: Some(dec!(0.0)),
            iceberg_qty: Some(dec!(0.0)),
            time: 1499827319559,
            update_time: 1499827319559,
            is_working: true,
            working_time: 1499827319559,
            orig_quote_order_qty: dec!(0.000000),
            self_trade_prevention_mode: STPMode::None,
        };

        let current = deserialize_json(json).unwrap();

        assert_eq!(expected, current);
    }
}
