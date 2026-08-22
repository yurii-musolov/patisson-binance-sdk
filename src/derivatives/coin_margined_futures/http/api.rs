use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    derivatives::coin_margined_futures::{
        ContractType, KlineInterval, MarginType, OrderResponseType, OrderSide, OrderStatus,
        OrderType, PermissionSets, PositionSide, RateLimitInterval, RateLimiter, SymbolStatus,
        TimeInForce, UnderlyingType, WorkingType,
    },
};

pub use crate::http::{ParsedHeaders as Headers, Response};

// ===== General =====

#[derive(Debug, Deserialize, PartialEq)]
pub struct TestConnectivity {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: Timestamp,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    pub timezone: String,
    pub server_time: Timestamp,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<SymbolInfo>,
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
    pub pair: String,
    pub contract_type: ContractType,
    /// Contract size in base asset (e.g. 100 for BTCUSD perpetual).
    pub contract_size: i64,
    pub delivery_date: Timestamp,
    pub onboard_date: Timestamp,
    pub contract_status: SymbolStatus,
    pub base_asset: String,
    pub quote_asset: String,
    pub margin_asset: String,
    pub price_precision: i8,
    pub quantity_precision: i8,
    pub base_asset_precision: i8,
    pub quote_precision: i8,
    pub equal_qty_precision: i8,
    pub max_move_order_limit: i32,
    pub maint_margin_percent: Decimal,
    pub required_margin_percent: Decimal,
    pub underlying_type: UnderlyingType,
    pub underlying_sub_type: Vec<String>, //  `PoW`, `Layer-1`, `Layer-2`, `Infrastructure`, `Payment`, `Storage`, `Meme`, `DeFi`, `Gaming`, `NFT`, ...
    pub trigger_protect: Decimal,
    pub liquidation_fee: Decimal,
    pub market_take_bound: Decimal,
    pub filters: Vec<SymbolFilter>,
    pub order_types: Vec<OrderType>,
    pub time_in_force: Vec<TimeInForce>,
    pub permission_sets: Vec<PermissionSets>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "filterType")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolFilter {
    PriceFilter(SymbolFilterPriceFilter),
    LotSize(SymbolFilterLotSize),
    MarketLotSize(SymbolFilterMarketLotSize),
    MaxNumOrders(SymbolFilterMaxNumOrders),
    MaxNumAlgoOrders(SymbolFilterMaxNumAlgoOrders),
    PercentPrice(SymbolFilterPercentPrice),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolFilterPriceFilter {
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub tick_size: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolFilterLotSize {
    pub max_qty: Decimal,
    pub step_size: Decimal,
    pub min_qty: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolFilterMarketLotSize {
    pub step_size: Decimal,
    pub max_qty: Decimal,
    pub min_qty: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolFilterMaxNumOrders {
    pub limit: i32,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolFilterMaxNumAlgoOrders {
    pub limit: i32,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolFilterPercentPrice {
    pub multiplier_decimal: Decimal,
    pub multiplier_up: Decimal,
    pub multiplier_down: Decimal,
}

// ===== Market Data =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderBookParams {
    symbol: String,
    /// Default 500. Valid: 5, 10, 20, 50, 100, 500, 1000.
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
    pub symbol: String,
    pub pair: String,
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
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
pub struct GetKlineListParams {
    symbol: String,
    interval: KlineInterval,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    /// Default 500; Maximum 1500.
    limit: Option<u64>,
}

impl GetKlineListParams {
    pub fn new(symbol: impl Into<String>, interval: KlineInterval) -> Self {
        Self {
            symbol: symbol.into(),
            interval,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }

    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }

    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }

    pub fn limit(mut self, value: u64) -> Self {
        self.limit = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Kline(
    Timestamp, // open time
    Decimal,   // open
    Decimal,   // high
    Decimal,   // low
    Decimal,   // close
    Decimal,   // volume (contracts)
    Timestamp, // close time
    Decimal,   // base asset volume
    u64,       // number of trades
    Decimal,   // taker buy volume (contracts)
    Decimal,   // taker buy base asset volume
    String,    // ignore
);

impl Kline {
    pub fn time_open(&self) -> Timestamp {
        self.0
    }
    pub fn open(&self) -> Decimal {
        self.1
    }
    pub fn high(&self) -> Decimal {
        self.2
    }
    pub fn low(&self) -> Decimal {
        self.3
    }
    pub fn close(&self) -> Decimal {
        self.4
    }
    pub fn volume(&self) -> Decimal {
        self.5
    }
    pub fn time_close(&self) -> Timestamp {
        self.6
    }
    pub fn base_asset_volume(&self) -> Decimal {
        self.7
    }
    pub fn trade_count(&self) -> u64 {
        self.8
    }
    pub fn taker_buy_volume(&self) -> Decimal {
        self.9
    }
    pub fn taker_buy_base_asset_volume(&self) -> Decimal {
        self.10
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetSymbolPriceTickerParams {
    symbol: String,
}

impl GetSymbolPriceTickerParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolPriceTicker {
    pub symbol: String,
    pub ps: String,
    pub price: Decimal,
    pub time: Timestamp,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetSymbolOrderBookTickerParams {
    symbol: String,
}

impl GetSymbolOrderBookTickerParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolOrderBookTicker {
    pub symbol: String,
    pub pair: String,
    pub bid_price: Decimal,
    pub bid_qty: Decimal,
    pub ask_price: Decimal,
    pub ask_qty: Decimal,
    pub time: Timestamp,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetPremiumIndexParams {
    pub(super) symbol: Option<String>,
    pair: Option<String>,
}

impl GetPremiumIndexParams {
    pub fn new() -> Self {
        Self {
            symbol: None,
            pair: None,
        }
    }

    pub fn symbol(mut self, value: impl Into<String>) -> Self {
        self.symbol = Some(value.into());
        self
    }
    pub fn pair(mut self, value: impl Into<String>) -> Self {
        self.pair = Some(value.into());
        self
    }
}

impl Default for GetPremiumIndexParams {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PremiumIndex {
    pub symbol: String,
    pub pair: String,
    pub mark_price: Decimal,
    pub index_price: Decimal,
    pub estimated_settle_price: Decimal,
    /// Empty string for delivery (non-perpetual) symbols.
    pub last_funding_rate: String,
    /// Empty string for delivery (non-perpetual) symbols.
    pub interest_rate: String,
    pub next_funding_time: Timestamp,
    pub time: Timestamp,
}

// ===== Trading =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderRequest {
    symbol: String,
    side: OrderSide,
    #[serde(rename = "type")]
    order_type: OrderType,
    position_side: Option<PositionSide>,
    time_in_force: Option<TimeInForce>,
    /// Quantity in contracts (COIN-M).
    quantity: Option<Decimal>,
    reduce_only: Option<bool>,
    price: Option<Decimal>,
    new_client_order_id: Option<String>,
    stop_price: Option<Decimal>,
    close_position: Option<bool>,
    activation_price: Option<Decimal>,
    /// Trailing-stop callback rate (0.1 .. 5).
    callback_rate: Option<Decimal>,
    working_type: Option<WorkingType>,
    price_protect: Option<bool>,
    new_order_resp_type: Option<OrderResponseType>,
    /// Max 60000.
    recv_window: Option<u64>,
}

impl NewOrderRequest {
    pub fn new(symbol: impl Into<String>, side: OrderSide, order_type: OrderType) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type,
            position_side: None,
            time_in_force: None,
            quantity: None,
            reduce_only: None,
            price: None,
            new_client_order_id: None,
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_order_resp_type: None,
            recv_window: None,
        }
    }

    pub fn position_side(mut self, value: PositionSide) -> Self {
        self.position_side = Some(value);
        self
    }
    pub fn time_in_force(mut self, value: TimeInForce) -> Self {
        self.time_in_force = Some(value);
        self
    }
    pub fn quantity(mut self, value: Decimal) -> Self {
        self.quantity = Some(value);
        self
    }
    pub fn reduce_only(mut self, value: bool) -> Self {
        self.reduce_only = Some(value);
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
    pub fn stop_price(mut self, value: Decimal) -> Self {
        self.stop_price = Some(value);
        self
    }
    pub fn close_position(mut self, value: bool) -> Self {
        self.close_position = Some(value);
        self
    }
    pub fn activation_price(mut self, value: Decimal) -> Self {
        self.activation_price = Some(value);
        self
    }
    pub fn callback_rate(mut self, value: Decimal) -> Self {
        self.callback_rate = Some(value);
        self
    }
    pub fn working_type(mut self, value: WorkingType) -> Self {
        self.working_type = Some(value);
        self
    }
    pub fn price_protect(mut self, value: bool) -> Self {
        self.price_protect = Some(value);
        self
    }
    pub fn new_order_resp_type(mut self, value: OrderResponseType) -> Self {
        self.new_order_resp_type = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderResponse {
    pub symbol: String,
    pub pair: String,
    pub order_id: i64,
    pub client_order_id: String,
    pub status: OrderStatus,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub position_side: PositionSide,
    pub price: Decimal,
    pub avg_price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cum_base: Decimal,
    pub time_in_force: TimeInForce,
    pub reduce_only: bool,
    pub close_position: bool,
    pub stop_price: Decimal,
    pub working_type: WorkingType,
    pub price_protect: bool,
    pub orig_type: OrderType,
    pub update_time: Timestamp,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryOrderParams {
    symbol: String,
    order_id: Option<i64>,
    orig_client_order_id: Option<String>,
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
    pub pair: String,
    pub order_id: i64,
    pub client_order_id: String,
    pub status: OrderStatus,
    pub price: Decimal,
    pub avg_price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cum_base: Decimal,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub position_side: PositionSide,
    pub stop_price: Decimal,
    pub working_type: WorkingType,
    pub price_protect: bool,
    pub orig_type: OrderType,
    pub reduce_only: bool,
    pub close_position: bool,
    pub time: Timestamp,
    pub update_time: Timestamp,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderParams {
    symbol: String,
    order_id: Option<i64>,
    orig_client_order_id: Option<String>,
    recv_window: Option<u64>,
}

impl CancelOrderParams {
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

/// Response of `DELETE /dapi/v1/order`. Distinct shape from [`Order`]: no
/// `avgPrice`/`time`, and the cumulative-fill field is named `cumQty` here.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderResponse {
    pub symbol: String,
    pub pair: String,
    pub order_id: i64,
    pub client_order_id: String,
    pub status: OrderStatus,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cum_qty: Decimal,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub position_side: PositionSide,
    pub stop_price: Decimal,
    pub working_type: WorkingType,
    pub price_protect: bool,
    pub orig_type: OrderType,
    pub reduce_only: bool,
    pub close_position: bool,
    pub update_time: Timestamp,
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelAllOpenOrdersParams {
    symbol: String,
    recv_window: Option<u64>,
}

impl CancelAllOpenOrdersParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            recv_window: None,
        }
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

/// Generic `{code, msg}` acknowledgement returned by a few COIN-M Futures
/// trading endpoints (cancel-all-open-orders, change-margin-type, change-position-mode).
#[derive(Debug, Deserialize, PartialEq)]
pub struct ActionResult {
    pub code: i64,
    pub msg: String,
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOpenOrdersParams {
    pub(super) symbol: Option<String>,
    pair: Option<String>,
    recv_window: Option<u64>,
}

impl GetOpenOrdersParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol(mut self, value: impl Into<String>) -> Self {
        self.symbol = Some(value.into());
        self
    }
    pub fn pair(mut self, value: impl Into<String>) -> Self {
        self.pair = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAllOrdersParams {
    symbol: Option<String>,
    pair: Option<String>,
    order_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    /// Default 50; Maximum 100.
    limit: Option<u64>,
    recv_window: Option<u64>,
}

impl GetAllOrdersParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol(mut self, value: impl Into<String>) -> Self {
        self.symbol = Some(value.into());
        self
    }
    pub fn pair(mut self, value: impl Into<String>) -> Self {
        self.pair = Some(value.into());
        self
    }
    pub fn order_id(mut self, value: i64) -> Self {
        self.order_id = Some(value);
        self
    }
    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }
    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }
    pub fn limit(mut self, value: u64) -> Self {
        self.limit = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountTradeListParams {
    symbol: Option<String>,
    pair: Option<String>,
    order_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    from_id: Option<i64>,
    /// Default 50; Maximum 1000.
    limit: Option<u64>,
    recv_window: Option<u64>,
}

impl GetAccountTradeListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol(mut self, value: impl Into<String>) -> Self {
        self.symbol = Some(value.into());
        self
    }
    pub fn pair(mut self, value: impl Into<String>) -> Self {
        self.pair = Some(value.into());
        self
    }
    pub fn order_id(mut self, value: i64) -> Self {
        self.order_id = Some(value);
        self
    }
    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }
    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }
    pub fn from_id(mut self, value: i64) -> Self {
        self.from_id = Some(value);
        self
    }
    pub fn limit(mut self, value: u64) -> Self {
        self.limit = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub symbol: String,
    pub id: i64,
    pub order_id: i64,
    pub pair: String,
    pub side: OrderSide,
    pub price: Decimal,
    pub qty: Decimal,
    pub realized_pnl: Decimal,
    pub margin_asset: String,
    pub base_qty: Decimal,
    pub quote_qty: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    pub time: Timestamp,
    pub position_side: PositionSide,
    pub buyer: bool,
    pub maker: bool,
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetPositionInformationParams {
    margin_asset: Option<String>,
    pair: Option<String>,
    recv_window: Option<u64>,
}

impl GetPositionInformationParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn margin_asset(mut self, value: impl Into<String>) -> Self {
        self.margin_asset = Some(value.into());
        self
    }
    pub fn pair(mut self, value: impl Into<String>) -> Self {
        self.pair = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PositionInformation {
    pub symbol: String,
    pub position_amt: Decimal,
    pub entry_price: Decimal,
    pub break_even_price: Decimal,
    pub mark_price: Decimal,
    pub un_realized_profit: Decimal,
    pub liquidation_price: Decimal,
    pub leverage: Decimal,
    pub max_qty: Decimal,
    pub margin_type: MarginType,
    pub isolated_margin: Decimal,
    /// Binance sends this as a literal `"true"`/`"false"` string, not a JSON bool.
    pub is_auto_add_margin: String,
    pub position_side: PositionSide,
    pub update_time: Timestamp,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInitialLeverageParams {
    symbol: String,
    /// Target initial leverage: int from 1 to 125.
    leverage: i64,
    recv_window: Option<u64>,
}

impl ChangeInitialLeverageParams {
    pub fn new(symbol: impl Into<String>, leverage: i64) -> Self {
        Self {
            symbol: symbol.into(),
            leverage,
            recv_window: None,
        }
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInitialLeverageResponse {
    pub leverage: i64,
    pub max_qty: Decimal,
    pub symbol: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMarginTypeParams {
    symbol: String,
    margin_type: MarginType,
    recv_window: Option<u64>,
}

impl ChangeMarginTypeParams {
    pub fn new(symbol: impl Into<String>, margin_type: MarginType) -> Self {
        Self {
            symbol: symbol.into(),
            margin_type,
            recv_window: None,
        }
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangePositionModeParams {
    /// `true` selects Hedge Mode, `false` selects One-way Mode.
    dual_side_position: bool,
    recv_window: Option<u64>,
}

impl ChangePositionModeParams {
    pub fn new(dual_side_position: bool) -> Self {
        Self {
            dual_side_position,
            recv_window: None,
        }
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetCurrentPositionModeParams {
    recv_window: Option<u64>,
}

impl GetCurrentPositionModeParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentPositionMode {
    pub dual_side_position: bool,
}

// ===== User data stream =====

/// `POST /dapi/v1/listenKey` response.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ListenKey {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

/// Response of the listenKey keepalive/close calls (`PUT` / `DELETE`). The
/// body is an empty JSON object `{}`.
#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct EmptyResponse {}

// ===== Account =====

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountInformationParams {
    recv_window: Option<u64>,
}

impl GetAccountInformationParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetBalanceParams {
    recv_window: Option<u64>,
}

impl GetBalanceParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub account_alias: String,
    pub asset: String,
    pub balance: Decimal,
    pub withdraw_available: Decimal,
    pub cross_wallet_balance: Decimal,
    pub cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub update_time: Timestamp,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    pub assets: Vec<AccountAsset>,
    pub positions: Vec<AccountPosition>,
    pub can_deposit: bool,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub fee_tier: u32,
    pub update_time: Timestamp,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountAsset {
    pub asset: String,
    pub wallet_balance: Decimal,
    pub unrealized_profit: Decimal,
    pub margin_balance: Decimal,
    pub maint_margin: Decimal,
    pub initial_margin: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub available_balance: Decimal,
    pub cross_wallet_balance: Decimal,
    pub cross_un_pnl: Decimal,
    pub update_time: Timestamp,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountPosition {
    pub symbol: String,
    pub position_amt: Decimal,
    pub initial_margin: Decimal,
    pub maint_margin: Decimal,
    pub unrealized_profit: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub leverage: Decimal,
    pub isolated: bool,
    pub position_side: PositionSide,
    pub entry_price: Decimal,
    pub max_qty: Decimal,
    pub update_time: Timestamp,
}

#[cfg(test)]
mod tests {

    use rust_decimal::dec;

    use crate::{derivatives::coin_margined_futures::PermissionSets, serde::deserialize_json};

    use super::*;

    #[test]
    fn deserialize_exchange_info() {
        let json = r#"{
            "timezone": "UTC",
            "serverTime": 1781510419499,
            "rateLimits": [
                {
                "rateLimitType": "REQUEST_WEIGHT",
                "interval": "MINUTE",
                "intervalNum": 1,
                "limit": 2400
                },
                {
                "rateLimitType": "ORDERS",
                "interval": "MINUTE",
                "intervalNum": 1,
                "limit": 1200
                }
            ],
            "exchangeFilters": [],
            "symbols": [
                {
                    "symbol": "BTCUSD_PERP",
                    "pair": "BTCUSD",
                    "contractType": "PERPETUAL",
                    "deliveryDate": 4133404800000,
                    "onboardDate": 1597042800000,
                    "contractStatus": "TRADING",
                    "contractSize": 100,
                    "maintMarginPercent": "2.5000",
                    "requiredMarginPercent": "5.0000",
                    "baseAsset": "BTC",
                    "quoteAsset": "USD",
                    "marginAsset": "BTC",
                    "pricePrecision": 1,
                    "quantityPrecision": 0,
                    "baseAssetPrecision": 8,
                    "quotePrecision": 8,
                    "underlyingType": "COIN",
                    "underlyingSubType": [
                        "PoW"
                    ],
                    "triggerProtect": "0.0500",
                    "liquidationFee": "0.015000",
                    "marketTakeBound": "0.05",
                    "equalQtyPrecision": 4,
                    "maxMoveOrderLimit": 10000,
                    "filters": [
                        {
                        "minPrice": "1000",
                        "filterType": "PRICE_FILTER",
                        "maxPrice": "4520958",
                        "tickSize": "0.1"
                        },
                        {
                        "filterType": "LOT_SIZE",
                        "maxQty": "1000000",
                        "stepSize": "1",
                        "minQty": "1"
                        },
                        {
                        "stepSize": "1",
                        "filterType": "MARKET_LOT_SIZE",
                        "maxQty": "60000",
                        "minQty": "1"
                        },
                        {
                        "filterType": "MAX_NUM_ORDERS",
                        "limit": 200
                        },
                        {
                        "limit": 20,
                        "filterType": "MAX_NUM_ALGO_ORDERS"
                        },
                        {
                        "multiplierDecimal": "4",
                        "multiplierUp": "1.0500",
                        "filterType": "PERCENT_PRICE",
                        "multiplierDown": "0.9500"
                        }
                    ],
                    "orderTypes": [
                        "LIMIT",
                        "MARKET",
                        "STOP",
                        "STOP_MARKET",
                        "TAKE_PROFIT",
                        "TAKE_PROFIT_MARKET",
                        "TRAILING_STOP_MARKET"
                    ],
                    "timeInForce": [
                        "GTC",
                        "IOC",
                        "FOK",
                        "GTX"
                    ],
                    "permissionSets": [
                        "GRID"
                    ]
                }
            ]
        }"#;

        let rate_limits = vec![
            RateLimit {
                rate_limit_type: RateLimiter::RequestWeight,
                interval: RateLimitInterval::Minute,
                interval_num: 1,
                limit: 2400,
            },
            RateLimit {
                rate_limit_type: RateLimiter::Orders,
                interval: RateLimitInterval::Minute,
                interval_num: 1,
                limit: 1200,
            },
        ];
        let filters = vec![
            SymbolFilter::PriceFilter(SymbolFilterPriceFilter {
                min_price: dec!(1000),
                max_price: dec!(4520958),
                tick_size: dec!(0.1),
            }),
            SymbolFilter::LotSize(SymbolFilterLotSize {
                max_qty: dec!(1000000),
                step_size: dec!(1),
                min_qty: dec!(1),
            }),
            SymbolFilter::MarketLotSize(SymbolFilterMarketLotSize {
                step_size: dec!(1),
                max_qty: dec!(60000),
                min_qty: dec!(1),
            }),
            SymbolFilter::MaxNumOrders(SymbolFilterMaxNumOrders { limit: 200 }),
            SymbolFilter::MaxNumAlgoOrders(SymbolFilterMaxNumAlgoOrders { limit: 20 }),
            SymbolFilter::PercentPrice(SymbolFilterPercentPrice {
                multiplier_decimal: dec!(4),
                multiplier_up: dec!(1.0500),
                multiplier_down: dec!(0.9500),
            }),
        ];
        let order_types = vec![
            OrderType::Limit,
            OrderType::Market,
            OrderType::Stop,
            OrderType::StopMarket,
            OrderType::TakeProfit,
            OrderType::TakeProfitMarket,
            OrderType::TrailingStopMarket,
        ];
        let time_in_force = vec![
            TimeInForce::GTC,
            TimeInForce::IOC,
            TimeInForce::FOK,
            TimeInForce::GTX,
        ];
        let permission_sets = vec![PermissionSets::GRID];
        let symbol = SymbolInfo {
            symbol: "BTCUSD_PERP".to_string(),
            pair: "BTCUSD".to_string(),
            contract_type: ContractType::Perpetual,
            contract_size: 100,
            delivery_date: 4133404800000,
            onboard_date: 1597042800000,
            contract_status: SymbolStatus::Trading,
            base_asset: "BTC".to_string(),
            quote_asset: "USD".to_string(),
            margin_asset: "BTC".to_string(),
            price_precision: 1,
            quantity_precision: 0,
            base_asset_precision: 8,
            quote_precision: 8,
            equal_qty_precision: 4,
            max_move_order_limit: 10000,
            maint_margin_percent: dec!(2.5000),
            required_margin_percent: dec!(5.0000),
            underlying_type: UnderlyingType::COIN,
            underlying_sub_type: vec!["PoW".to_string()],
            trigger_protect: dec!(0.0500),
            liquidation_fee: dec!(0.015000),
            market_take_bound: dec!(0.05),
            filters,
            order_types,
            time_in_force,
            permission_sets,
        };
        let expected = ExchangeInfo {
            timezone: "UTC".into(),
            server_time: 1781510419499,
            rate_limits,
            symbols: vec![symbol],
        };

        let message = deserialize_json(json).unwrap();

        assert_eq!(expected, message);
    }
}
