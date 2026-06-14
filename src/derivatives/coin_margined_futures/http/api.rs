use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    derivatives::coin_margined_futures::{
        ContractType, KlineInterval, OrderResponseType, OrderSide, OrderStatus, OrderType,
        PositionSide, RateLimitInterval, RateLimiter, SymbolStatus, TimeInForce, WorkingType,
    },
};

#[derive(Debug, PartialEq)]
pub struct Response<T> {
    pub result: T,
    pub headers: Headers,
}

#[derive(Debug, PartialEq)]
pub struct Headers {
    pub retry_after: Option<Timestamp>,
}

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
    pub contract_size: Decimal,
    pub delivery_date: Timestamp,
    pub onboard_date: Timestamp,
    pub contract_status: SymbolStatus,
    pub base_asset: String,
    pub quote_asset: String,
    pub margin_asset: String,
    pub price_precision: u8,
    pub quantity_precision: u8,
    pub base_asset_precision: u8,
    pub quote_precision: u8,
    pub equal_qty_precision: u8,
    pub order_types: Vec<OrderType>,
    pub time_in_force: Vec<TimeInForce>,
}

// ===== Market Data =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderBookParams {
    symbol: String,
    /// Default 500. Valid: 5, 10, 20, 50, 100, 500, 1000.
    limit: Option<u64>,
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
    recv_window: Option<i64>,
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
    pub fn recv_window(mut self, value: i64) -> Self {
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
    recv_window: Option<i64>,
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
    pub fn recv_window(mut self, value: i64) -> Self {
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

// ===== Account =====

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountInformationParams {
    recv_window: Option<i64>,
}

impl GetAccountInformationParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: i64) -> Self {
        self.recv_window = Some(value);
        self
    }
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
