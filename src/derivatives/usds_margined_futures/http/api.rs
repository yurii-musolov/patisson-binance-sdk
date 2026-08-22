use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    derivatives::usds_margined_futures::{
        ContractType, KlineInterval, MarginType, OrderResponseType, OrderSide, OrderStatus,
        OrderType, PositionSide, PriceMatch, RateLimitInterval, RateLimiter, STPMode, SymbolStatus,
        TimeInForce, WorkingType,
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
    pub status: SymbolStatus,
    pub base_asset: String,
    pub quote_asset: String,
    pub margin_asset: String,
    pub price_precision: u8,
    pub quantity_precision: u8,
    pub base_asset_precision: u8,
    pub quote_precision: u8,
    pub order_types: Vec<OrderType>,
    pub time_in_force: Vec<TimeInForce>,
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
    /// Message output time
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Transaction time
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

/// Tuple-array kline returned by /fapi/v1/klines.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Kline(
    Timestamp, // open time
    Decimal,   // open
    Decimal,   // high
    Decimal,   // low
    Decimal,   // close
    Decimal,   // volume
    Timestamp, // close time
    Decimal,   // quote asset volume
    u64,       // number of trades
    Decimal,   // taker buy base asset volume
    Decimal,   // taker buy quote asset volume
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
    pub fn quote_asset_volume(&self) -> Decimal {
        self.7
    }
    pub fn trade_count(&self) -> u64 {
        self.8
    }
    pub fn taker_buy_base_asset_volume(&self) -> Decimal {
        self.9
    }
    pub fn taker_buy_quote_asset_volume(&self) -> Decimal {
        self.10
    }
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
    /// Order cancellation deadline in epoch-ms. Required when `time_in_force = GTD`.
    good_till_date: Option<Timestamp>,
    /// Computed order price mode (e.g. OPPONENT, QUEUE). Mutually exclusive
    /// with `price`.
    price_match: Option<PriceMatch>,
    self_trade_prevention_mode: Option<STPMode>,
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
            good_till_date: None,
            price_match: None,
            self_trade_prevention_mode: None,
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
    pub fn good_till_date(mut self, value: Timestamp) -> Self {
        self.good_till_date = Some(value);
        self
    }
    pub fn price_match(mut self, value: PriceMatch) -> Self {
        self.price_match = Some(value);
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
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderResponse {
    pub symbol: String,
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
    pub cum_quote: Decimal,
    pub time_in_force: TimeInForce,
    pub reduce_only: bool,
    pub close_position: bool,
    pub stop_price: Decimal,
    pub working_type: WorkingType,
    pub price_protect: bool,
    pub orig_type: OrderType,
    pub update_time: Timestamp,
    /// Present when `time_in_force = GTD`.
    pub good_till_date: Option<Timestamp>,
    /// Present when `price_match` was set (non-`NONE`).
    pub price_match: Option<PriceMatch>,
    pub self_trade_prevention_mode: Option<STPMode>,
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
    pub order_id: i64,
    pub client_order_id: String,
    pub status: OrderStatus,
    pub price: Decimal,
    pub avg_price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cum_quote: Decimal,
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
    /// Absent from the cancel-order response (only `updateTime` is present
    /// there); present on order-status / open-orders / all-orders responses.
    #[serde(default)]
    pub time: Option<Timestamp>,
    pub update_time: Timestamp,
    /// Present on list-style responses (open/all orders, cancel); absent from
    /// the single order-status response.
    #[serde(default)]
    pub pair: Option<String>,
    #[serde(default)]
    pub cum_base: Option<Decimal>,
    /// Only present for `TRAILING_STOP_MARKET` orders.
    #[serde(default)]
    pub activate_price: Option<Decimal>,
    /// Callback rate. Only present for `TRAILING_STOP_MARKET` orders.
    #[serde(default)]
    pub price_rate: Option<Decimal>,
    #[serde(default)]
    pub price_match: Option<PriceMatch>,
    #[serde(default)]
    pub self_trade_prevention_mode: Option<STPMode>,
    /// Present when `time_in_force = GTD`.
    #[serde(default)]
    pub good_till_date: Option<Timestamp>,
}

// ===== Trading (order management) =====

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

#[derive(Debug, Serialize, PartialEq)]
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

/// Body shape Binance returns for actions with no natural payload: cancel
/// all open orders, change margin type, change position mode.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ActionResult {
    pub code: i64,
    pub msg: String,
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOpenOrdersParams {
    pub(super) symbol: Option<String>,
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
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAllOrdersParams {
    symbol: String,
    order_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    /// Default 500; Maximum 1000.
    limit: Option<u64>,
    recv_window: Option<u64>,
}

impl GetAllOrdersParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            start_time: None,
            end_time: None,
            limit: None,
            recv_window: None,
        }
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

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountTradeListParams {
    symbol: String,
    order_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    from_id: Option<i64>,
    /// Default 500; Maximum 1000.
    limit: Option<u64>,
    recv_window: Option<u64>,
}

impl GetAccountTradeListParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            start_time: None,
            end_time: None,
            from_id: None,
            limit: None,
            recv_window: None,
        }
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
pub struct AccountTrade {
    pub symbol: String,
    pub id: i64,
    pub order_id: i64,
    pub side: OrderSide,
    pub price: Decimal,
    pub qty: Decimal,
    pub quote_qty: Decimal,
    #[serde(default)]
    pub realized_pnl: Option<Decimal>,
    #[serde(default)]
    pub margin_asset: Option<String>,
    pub commission: Decimal,
    pub commission_asset: String,
    pub time: Timestamp,
    pub position_side: PositionSide,
    pub buyer: bool,
    pub maker: bool,
}

// ===== Trading (leverage / margin type / position mode) =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInitialLeverageParams {
    symbol: String,
    /// Target initial leverage: 1 to 125.
    leverage: u8,
    recv_window: Option<u64>,
}

impl ChangeInitialLeverageParams {
    pub fn new(symbol: impl Into<String>, leverage: u8) -> Self {
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
pub struct Leverage {
    pub leverage: u8,
    pub max_notional_value: Decimal,
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
    dual_side_position: bool,
    recv_window: Option<u64>,
}

impl ChangePositionModeParams {
    /// `true` = Hedge Mode (LONG/SHORT); `false` = One-way Mode (BOTH).
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
pub struct PositionMode {
    pub dual_side_position: bool,
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetPositionInformationParams {
    symbol: Option<String>,
    recv_window: Option<u64>,
}

impl GetPositionInformationParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol(mut self, value: impl Into<String>) -> Self {
        self.symbol = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    pub position_side: PositionSide,
    pub position_amt: Decimal,
    pub entry_price: Decimal,
    #[serde(default)]
    pub break_even_price: Option<Decimal>,
    pub mark_price: Decimal,
    pub un_realized_profit: Decimal,
    pub liquidation_price: Decimal,
    pub isolated_margin: Decimal,
    pub notional: Decimal,
    pub margin_asset: String,
    pub isolated_wallet: Decimal,
    pub initial_margin: Decimal,
    pub maint_margin: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub adl: u8,
    #[serde(default)]
    pub bid_notional: Option<Decimal>,
    #[serde(default)]
    pub ask_notional: Option<Decimal>,
    pub update_time: Timestamp,
    pub leverage: Decimal,
    pub isolated: bool,
}

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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    pub total_initial_margin: Decimal,
    pub total_maint_margin: Decimal,
    pub total_wallet_balance: Decimal,
    pub total_unrealized_profit: Decimal,
    pub total_margin_balance: Decimal,
    pub total_position_initial_margin: Decimal,
    pub total_open_order_initial_margin: Decimal,
    pub total_cross_wallet_balance: Decimal,
    pub total_cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub assets: Vec<AccountAsset>,
    pub positions: Vec<AccountPosition>,
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
    pub cross_wallet_balance: Decimal,
    pub cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub update_time: Timestamp,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountPosition {
    pub symbol: String,
    pub initial_margin: Decimal,
    pub maint_margin: Decimal,
    pub unrealized_profit: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub leverage: Decimal,
    pub isolated: bool,
    pub entry_price: Decimal,
    pub max_notional: Decimal,
    pub position_side: PositionSide,
    pub position_amt: Decimal,
    pub update_time: Timestamp,
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountBalanceParams {
    recv_window: Option<u64>,
}

impl GetAccountBalanceParams {
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
pub struct AccountBalance {
    pub account_alias: String,
    pub asset: String,
    pub balance: Decimal,
    pub cross_wallet_balance: Decimal,
    pub cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub margin_available: bool,
    pub update_time: Timestamp,
}

// ===== User data stream =====

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
    pub listen_key: String,
}

/// Returned by keepalive and close operations on the user data stream
/// (`PUT` / `DELETE`). The body is an empty JSON object `{}`.
#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct EmptyResponse {}

// ===== Market Data (tickers / mark price) =====

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
    pub bid_price: Decimal,
    pub bid_qty: Decimal,
    pub ask_price: Decimal,
    pub ask_qty: Decimal,
    pub time: Timestamp,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMarkPriceParams {
    symbol: String,
}

impl GetMarkPriceParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
        }
    }
}

/// Mark price, index price and funding rate for a perpetual symbol.
/// `estimated_settle_price` and `interest_rate` are only present near
/// delivery for `CURRENT_QUARTER` / `NEXT_QUARTER` contracts.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarkPrice {
    pub symbol: String,
    pub mark_price: Decimal,
    pub index_price: Decimal,
    #[serde(default)]
    pub estimated_settle_price: Option<Decimal>,
    pub last_funding_rate: Decimal,
    #[serde(default)]
    pub interest_rate: Option<Decimal>,
    pub next_funding_time: Timestamp,
    pub time: Timestamp,
}
