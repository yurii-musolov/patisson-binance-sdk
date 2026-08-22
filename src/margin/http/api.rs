use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    margin::{
        BorrowRepayType, ContingencyType, IsIsolated, MarginLevelStatus, OrderListOrderStatus,
        OrderListStatus, OrderResponseType, OrderSide, OrderStatus, OrderType, STPMode,
        SideEffectType, TimeInForce,
    },
};

pub use crate::http::{ParsedHeaders as Headers, Response};

// ===== Margin metadata =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAllMarginAssetsParams {
    asset: Option<String>,
    recv_window: Option<u64>,
}

impl GetAllMarginAssetsParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn asset(mut self, value: impl Into<String>) -> Self {
        self.asset = Some(value.into());
        self
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarginAsset {
    pub asset_full_name: String,
    pub asset_name: String,
    pub is_borrowable: bool,
    pub is_mortgageable: bool,
    pub user_min_borrow: Decimal,
    pub user_min_repay: Decimal,
}

// ===== Cross margin account =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetMarginAccountParams {
    recv_window: Option<u64>,
}

impl GetMarginAccountParams {
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
pub struct MarginAccount {
    pub created: bool,
    pub borrow_enabled: bool,
    pub margin_level: Decimal,
    pub collateral_margin_level: Decimal,
    pub total_asset_of_btc: Decimal,
    pub total_liability_of_btc: Decimal,
    pub total_net_asset_of_btc: Decimal,
    pub total_collateral_value_in_usdt: Decimal,
    pub trade_enabled: bool,
    pub transfer_in_enabled: bool,
    pub transfer_out_enabled: bool,
    pub account_type: String,
    pub margin_level_status: MarginLevelStatus,
    pub user_assets: Vec<MarginUserAsset>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarginUserAsset {
    pub asset: String,
    pub borrowed: Decimal,
    pub free: Decimal,
    pub interest: Decimal,
    pub locked: Decimal,
    pub net_asset: Decimal,
}

// ===== Margin trading =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderRequest {
    symbol: String,
    is_isolated: Option<IsIsolated>,
    side: OrderSide,
    #[serde(rename = "type")]
    order_type: OrderType,
    quantity: Option<Decimal>,
    quote_order_qty: Option<Decimal>,
    price: Option<Decimal>,
    stop_price: Option<Decimal>,
    /// A unique id among open orders. Automatically generated if not sent.
    new_client_order_id: Option<String>,
    /// Used with LIMIT, STOP_LOSS_LIMIT, and TAKE_PROFIT_LIMIT to create an iceberg order.
    iceberg_qty: Option<Decimal>,
    /// Default ACK; MARKET and LIMIT default to FULL.
    new_order_resp_type: Option<OrderResponseType>,
    side_effect_type: Option<SideEffectType>,
    time_in_force: Option<TimeInForce>,
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
            is_isolated: None,
            quantity: None,
            quote_order_qty: None,
            price: None,
            stop_price: None,
            new_client_order_id: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            side_effect_type: None,
            time_in_force: None,
            self_trade_prevention_mode: None,
            recv_window: None,
        }
    }

    pub fn is_isolated(mut self, value: IsIsolated) -> Self {
        self.is_isolated = Some(value);
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
    pub fn stop_price(mut self, value: Decimal) -> Self {
        self.stop_price = Some(value);
        self
    }
    pub fn new_client_order_id(mut self, value: impl Into<String>) -> Self {
        self.new_client_order_id = Some(value.into());
        self
    }
    pub fn iceberg_qty(mut self, value: Decimal) -> Self {
        self.iceberg_qty = Some(value);
        self
    }
    pub fn new_order_resp_type(mut self, value: OrderResponseType) -> Self {
        self.new_order_resp_type = Some(value);
        self
    }
    pub fn side_effect_type(mut self, value: SideEffectType) -> Self {
        self.side_effect_type = Some(value);
        self
    }
    pub fn time_in_force(mut self, value: TimeInForce) -> Self {
        self.time_in_force = Some(value);
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
    pub client_order_id: String,
    pub transact_time: Timestamp,
    pub is_isolated: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderResponseResult {
    pub symbol: String,
    pub order_id: i64,
    pub client_order_id: String,
    pub transact_time: Timestamp,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub margin_buy_borrow_amount: Option<Decimal>,
    pub margin_buy_borrow_asset: Option<String>,
    pub is_isolated: bool,
    pub self_trade_prevention_mode: STPMode,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderResponseFull {
    pub symbol: String,
    pub order_id: i64,
    pub client_order_id: String,
    pub transact_time: Timestamp,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub fills: Vec<OrderFill>,
    pub margin_buy_borrow_amount: Option<Decimal>,
    pub margin_buy_borrow_asset: Option<String>,
    pub is_isolated: bool,
    pub self_trade_prevention_mode: STPMode,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderFill {
    pub price: Decimal,
    pub qty: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
}

// ===== Query order =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryOrderParams {
    symbol: String,
    is_isolated: Option<IsIsolated>,
    order_id: Option<i64>,
    orig_client_order_id: Option<String>,
    recv_window: Option<u64>,
}

impl QueryOrderParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            is_isolated: None,
            order_id: None,
            orig_client_order_id: None,
            recv_window: None,
        }
    }

    pub fn is_isolated(mut self, value: IsIsolated) -> Self {
        self.is_isolated = Some(value);
        self
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
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub stop_price: Option<Decimal>,
    pub iceberg_qty: Option<Decimal>,
    pub time: Timestamp,
    pub update_time: Timestamp,
    pub is_working: bool,
    pub is_isolated: bool,
    pub self_trade_prevention_mode: STPMode,
}

// ===== Max borrowable =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMaxBorrowableParams {
    asset: String,
    /// Required for isolated margin: the symbol whose isolated account to query.
    isolated_symbol: Option<String>,
    recv_window: Option<u64>,
}

impl GetMaxBorrowableParams {
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            isolated_symbol: None,
            recv_window: None,
        }
    }

    pub fn isolated_symbol(mut self, value: impl Into<String>) -> Self {
        self.isolated_symbol = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaxBorrowable {
    pub amount: Decimal,
    /// Account's current borrow limit for the asset.
    pub borrow_limit: Decimal,
}

// ===== Cancel order =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderParams {
    symbol: String,
    is_isolated: Option<IsIsolated>,
    order_id: Option<i64>,
    orig_client_order_id: Option<String>,
    new_client_order_id: Option<String>,
    recv_window: Option<u64>,
}

impl CancelOrderParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            is_isolated: None,
            order_id: None,
            orig_client_order_id: None,
            new_client_order_id: None,
            recv_window: None,
        }
    }

    pub fn is_isolated(mut self, value: IsIsolated) -> Self {
        self.is_isolated = Some(value);
        self
    }
    pub fn order_id(mut self, value: i64) -> Self {
        self.order_id = Some(value);
        self
    }
    pub fn orig_client_order_id(mut self, value: impl Into<String>) -> Self {
        self.orig_client_order_id = Some(value.into());
        self
    }
    pub fn new_client_order_id(mut self, value: impl Into<String>) -> Self {
        self.new_client_order_id = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanceledOrder {
    pub symbol: String,
    pub is_isolated: bool,
    pub order_id: i64,
    pub orig_client_order_id: String,
    pub client_order_id: String,
    pub price: Decimal,
    pub orig_qty: Decimal,
    pub executed_qty: Decimal,
    pub cummulative_quote_qty: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
}

/// One entry of [`CancelAllOpenOrdersParams`]'s response. `cancel_order`
/// (single order) can only ever cancel a plain order, but
/// `cancel_all_open_orders` cancels everything on the symbol at once —
/// including OCO order lists, whose entries are shaped completely
/// differently (no top-level `orderId`/`price`/`status` at all). `untagged`
/// tries [`CanceledOrder`] first and falls back to [`CanceledOrderList`].
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CanceledOrderOrList {
    Order(CanceledOrder),
    OrderList(CanceledOrderList),
}

/// Minimal per-leg reference inside a canceled order list's `orders` array —
/// distinct from [`CanceledOrder`], which is the shape of each entry in
/// `order_reports` instead.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderListOrderRef {
    pub symbol: String,
    pub order_id: i64,
    pub client_order_id: String,
}

/// An OCO order list canceled as a side effect of `cancel_all_open_orders`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanceledOrderList {
    pub order_list_id: i64,
    pub contingency_type: ContingencyType,
    pub list_status_type: OrderListStatus,
    pub list_order_status: OrderListOrderStatus,
    pub list_client_order_id: String,
    pub transaction_time: Timestamp,
    pub symbol: String,
    pub is_isolated: bool,
    pub orders: Vec<OrderListOrderRef>,
    pub order_reports: Vec<CanceledOrder>,
}

// ===== Cancel all open orders on a symbol =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelAllOpenOrdersParams {
    symbol: String,
    is_isolated: Option<IsIsolated>,
    recv_window: Option<u64>,
}

impl CancelAllOpenOrdersParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            is_isolated: None,
            recv_window: None,
        }
    }

    pub fn is_isolated(mut self, value: IsIsolated) -> Self {
        self.is_isolated = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

// ===== Current open orders =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetOpenOrdersParams {
    /// If omitted, open orders for all symbols are returned — much heavier;
    /// see `COST_OPEN_ORDERS_ALL` at the call site.
    pub(super) symbol: Option<String>,
    is_isolated: Option<IsIsolated>,
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
    pub fn is_isolated(mut self, value: IsIsolated) -> Self {
        self.is_isolated = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

// ===== All orders =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAllOrdersParams {
    symbol: String,
    is_isolated: Option<IsIsolated>,
    order_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    limit: Option<u64>,
    recv_window: Option<u64>,
}

impl GetAllOrdersParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            is_isolated: None,
            order_id: None,
            start_time: None,
            end_time: None,
            limit: None,
            recv_window: None,
        }
    }

    pub fn is_isolated(mut self, value: IsIsolated) -> Self {
        self.is_isolated = Some(value);
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

// ===== Account trade list =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountTradeListParams {
    symbol: String,
    is_isolated: Option<IsIsolated>,
    order_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    from_id: Option<i64>,
    limit: Option<u64>,
    recv_window: Option<u64>,
}

impl GetAccountTradeListParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            is_isolated: None,
            order_id: None,
            start_time: None,
            end_time: None,
            from_id: None,
            limit: None,
            recv_window: None,
        }
    }

    pub fn is_isolated(mut self, value: IsIsolated) -> Self {
        self.is_isolated = Some(value);
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
    pub commission: Decimal,
    pub commission_asset: String,
    pub id: i64,
    pub is_best_match: bool,
    pub is_buyer: bool,
    pub is_maker: bool,
    pub order_id: i64,
    pub price: Decimal,
    pub qty: Decimal,
    pub symbol: String,
    pub is_isolated: bool,
    pub time: Timestamp,
}

// ===== Borrow / repay execution =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BorrowRepayParams {
    asset: String,
    is_isolated: IsIsolated,
    amount: Decimal,
    #[serde(rename = "type")]
    order_type: BorrowRepayType,
    /// Required for isolated margin: the symbol whose isolated account to act on.
    symbol: Option<String>,
    recv_window: Option<u64>,
}

impl BorrowRepayParams {
    pub fn new(
        asset: impl Into<String>,
        is_isolated: IsIsolated,
        amount: Decimal,
        order_type: BorrowRepayType,
    ) -> Self {
        Self {
            asset: asset.into(),
            is_isolated,
            amount,
            order_type,
            symbol: None,
            recv_window: None,
        }
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
pub struct BorrowRepayResult {
    pub tran_id: i64,
}

// ===== Borrow / repay records =====

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BorrowRepayStatus {
    Confirmed,
    Pending,
    Failed,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetBorrowRepayRecordsParams {
    #[serde(rename = "type")]
    order_type: BorrowRepayType,
    asset: Option<String>,
    isolated_symbol: Option<String>,
    tx_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    current: Option<u64>,
    size: Option<u64>,
    archived: Option<bool>,
    recv_window: Option<u64>,
}

impl GetBorrowRepayRecordsParams {
    pub fn new(order_type: BorrowRepayType) -> Self {
        Self {
            order_type,
            asset: None,
            isolated_symbol: None,
            tx_id: None,
            start_time: None,
            end_time: None,
            current: None,
            size: None,
            archived: None,
            recv_window: None,
        }
    }

    pub fn asset(mut self, value: impl Into<String>) -> Self {
        self.asset = Some(value.into());
        self
    }
    pub fn isolated_symbol(mut self, value: impl Into<String>) -> Self {
        self.isolated_symbol = Some(value.into());
        self
    }
    pub fn tx_id(mut self, value: i64) -> Self {
        self.tx_id = Some(value);
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
    pub fn current(mut self, value: u64) -> Self {
        self.current = Some(value);
        self
    }
    pub fn size(mut self, value: u64) -> Self {
        self.size = Some(value);
        self
    }
    pub fn archived(mut self, value: bool) -> Self {
        self.archived = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BorrowRepayRecords {
    pub rows: Vec<BorrowRepayRecord>,
    pub total: i64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BorrowRepayRecord {
    #[serde(rename = "type")]
    pub order_type: BorrowRepayType,
    pub isolated_symbol: Option<String>,
    pub amount: Decimal,
    pub asset: String,
    pub interest: Decimal,
    pub principal: Decimal,
    pub status: BorrowRepayStatus,
    pub timestamp: Timestamp,
    pub tx_id: i64,
}

// ===== Isolated margin account info =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetIsolatedMarginAccountParams {
    /// Comma-separated list of symbols, max 5. Without it, all isolated
    /// symbols with non-zero assets/liabilities/borrow history are returned.
    symbols: Option<String>,
    recv_window: Option<u64>,
}

impl GetIsolatedMarginAccountParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbols(mut self, value: impl Into<String>) -> Self {
        self.symbols = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedMarginAccount {
    pub assets: Vec<IsolatedMarginAsset>,
    pub total_asset_of_btc: Decimal,
    pub total_liability_of_btc: Decimal,
    pub total_net_asset_of_btc: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedMarginAsset {
    pub base_asset: IsolatedMarginAssetDetail,
    pub quote_asset: IsolatedMarginAssetDetail,
    pub symbol: String,
    pub isolated_created: bool,
    pub enabled: bool,
    pub margin_level: Decimal,
    pub margin_level_status: MarginLevelStatus,
    pub margin_ratio: Decimal,
    pub index_price: Decimal,
    pub liquidate_price: Decimal,
    pub liquidate_rate: Decimal,
    pub trade_enabled: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedMarginAssetDetail {
    pub asset: String,
    pub borrow_enabled: bool,
    pub borrowed: Decimal,
    pub free: Decimal,
    pub interest: Decimal,
    pub locked: Decimal,
    pub net_asset: Decimal,
    pub net_asset_of_btc: Decimal,
    pub repay_enabled: bool,
    pub total_asset: Decimal,
}

// ===== Isolated margin symbols =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAllIsolatedMarginSymbolsParams {
    symbol: Option<String>,
    recv_window: Option<u64>,
}

impl GetAllIsolatedMarginSymbolsParams {
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
pub struct IsolatedMarginSymbol {
    pub base: String,
    pub quote: String,
    pub symbol: String,
    pub is_margin_trade: bool,
    pub is_buy_allowed: bool,
    pub is_sell_allowed: bool,
}

// ===== Margin interest rate history =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMarginInterestRateHistoryParams {
    asset: String,
    vip_level: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    limit: Option<u64>,
    recv_window: Option<u64>,
}

impl GetMarginInterestRateHistoryParams {
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            vip_level: None,
            start_time: None,
            end_time: None,
            limit: None,
            recv_window: None,
        }
    }

    pub fn vip_level(mut self, value: i64) -> Self {
        self.vip_level = Some(value);
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InterestRateRecord {
    pub asset: String,
    pub daily_interest_rate: Decimal,
    pub timestamp: Timestamp,
    pub vip_level: i64,
}

// ===== Margin price index =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetPriceIndexParams {
    symbol: String,
}

impl GetPriceIndexParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PriceIndex {
    pub calc_time: Timestamp,
    pub price: Decimal,
    pub symbol: String,
}

// ===== Max transfer-out amount =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMaxTransferOutAmountParams {
    asset: String,
    isolated_symbol: Option<String>,
    recv_window: Option<u64>,
}

impl GetMaxTransferOutAmountParams {
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            isolated_symbol: None,
            recv_window: None,
        }
    }

    pub fn isolated_symbol(mut self, value: impl Into<String>) -> Self {
        self.isolated_symbol = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaxTransferable {
    pub amount: Decimal,
}

// ===== Force liquidation record =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetForceLiquidationRecordParams {
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    isolated_symbol: Option<String>,
    current: Option<u64>,
    size: Option<u64>,
    recv_window: Option<u64>,
}

impl GetForceLiquidationRecordParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }
    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }
    pub fn isolated_symbol(mut self, value: impl Into<String>) -> Self {
        self.isolated_symbol = Some(value.into());
        self
    }
    pub fn current(mut self, value: u64) -> Self {
        self.current = Some(value);
        self
    }
    pub fn size(mut self, value: u64) -> Self {
        self.size = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ForceLiquidationRecords {
    pub rows: Vec<ForceLiquidationRecord>,
    pub total: i64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ForceLiquidationRecord {
    pub avg_price: Decimal,
    pub executed_qty: Decimal,
    pub order_id: i64,
    pub price: Decimal,
    pub qty: Decimal,
    pub side: OrderSide,
    pub symbol: String,
    pub time_in_force: TimeInForce,
    pub is_isolated: bool,
    pub updated_time: Timestamp,
}

// ===== User data stream =====

/// Response from `POST /sapi/v1/userDataStream{,/isolated}`.
///
/// Use the returned `listen_key` to connect to
/// `wss://stream.binance.com:9443/ws/<listen_key>` and consume margin user
/// data events. Keys live for 60 minutes from creation/keepalive — call
/// `keepalive_listen_key` every 30 minutes to extend.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
    pub listen_key: String,
}

/// Returned by keepalive and close operations on the user data stream
/// (`PUT` / `DELETE`). The body is an empty JSON object `{}`.
#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct EmptyResponse {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::deserialize_json;

    #[test]
    fn deserialize_listen_key() {
        let json =
            r#"{"listenKey":"pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"}"#;
        let parsed: ListenKey = deserialize_json(json).unwrap();
        assert_eq!(parsed.listen_key.len(), 64);
    }

    #[test]
    fn deserialize_empty_response() {
        let json = r#"{}"#;
        let parsed: EmptyResponse = deserialize_json(json).unwrap();
        assert_eq!(parsed, EmptyResponse {});
    }

    #[test]
    fn deserialize_canceled_order_or_list_mixed_array() {
        let json = r#"[
            {
                "symbol": "LTCBTC",
                "isIsolated": false,
                "orderId": 28,
                "origClientOrderId": "myOrder1",
                "clientOrderId": "cancelMyOrder1",
                "price": "1.00000000",
                "origQty": "10.00000000",
                "executedQty": "8.00000000",
                "cummulativeQuoteQty": "8.00000000",
                "status": "CANCELED",
                "timeInForce": "GTC",
                "type": "LIMIT",
                "side": "SELL"
            },
            {
                "orderListId": 1929,
                "contingencyType": "OCO",
                "listStatusType": "ALL_DONE",
                "listOrderStatus": "ALL_DONE",
                "listClientOrderId": "2inzWQdDvZLHbbAmAozX2N",
                "transactionTime": 1585230948299,
                "symbol": "BTCUSDT",
                "isIsolated": false,
                "orders": [
                    {"symbol": "BTCUSDT", "orderId": 20, "clientOrderId": "CwOOIPHSmYywx6jZX77TdL"},
                    {"symbol": "BTCUSDT", "orderId": 21, "clientOrderId": "461cPg51vQjV3zIMOXNz39"}
                ],
                "orderReports": [
                    {
                        "symbol": "BTCUSDT",
                        "isIsolated": false,
                        "orderId": 20,
                        "origClientOrderId": "CwOOIPHSmYywx6jZX77TdL",
                        "clientOrderId": "pXLV6Hz6mprAcVYpVMTGgx",
                        "price": "0.668611",
                        "origQty": "0.690354",
                        "executedQty": "0.000000",
                        "cummulativeQuoteQty": "0.000000",
                        "status": "CANCELED",
                        "timeInForce": "GTC",
                        "type": "STOP_LOSS_LIMIT",
                        "side": "SELL"
                    },
                    {
                        "symbol": "BTCUSDT",
                        "isIsolated": false,
                        "orderId": 21,
                        "origClientOrderId": "461cPg51vQjV3zIMOXNz39",
                        "clientOrderId": "pXLV6Hz6mprAcVYpVMTGgx",
                        "price": "0.008791",
                        "origQty": "0.690354",
                        "executedQty": "0.000000",
                        "cummulativeQuoteQty": "0.000000",
                        "status": "CANCELED",
                        "timeInForce": "GTC",
                        "type": "LIMIT_MAKER",
                        "side": "SELL"
                    }
                ]
            }
        ]"#;
        let parsed: Vec<CanceledOrderOrList> = deserialize_json(json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(matches!(parsed[0], CanceledOrderOrList::Order(_)));
        match &parsed[1] {
            CanceledOrderOrList::OrderList(list) => {
                assert_eq!(list.order_list_id, 1929);
                assert_eq!(list.orders.len(), 2);
                assert_eq!(list.order_reports.len(), 2);
            }
            other => panic!("expected OrderList, got {other:?}"),
        }
    }
}
