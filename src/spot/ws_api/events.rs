//! Spot user data stream events (`user-data-stream.md` in the Spot API docs).

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    Timestamp,
    spot::{
        ContingencyType, ExecutionType, ExpiryReason, OrderListOrderStatus, OrderListStatus,
        OrderSide, OrderStatus, OrderType, PegOffsetType, PegPriceType, STPMode, TimeInForce,
    },
};

/// An account event, internally tagged by `e`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "e")]
#[allow(clippy::large_enum_variant)]
pub enum UserDataEvent {
    #[serde(rename = "outboundAccountPosition")]
    OutboundAccountPosition(OutboundAccountPosition),
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdate),
    #[serde(rename = "executionReport")]
    ExecutionReport(ExecutionReport),
    #[serde(rename = "listStatus")]
    ListStatus(ListStatus),
    #[serde(rename = "externalLockUpdate")]
    ExternalLockUpdate(ExternalLockUpdate),
    /// The subscription ended (unsubscribed, or its session logged out).
    #[serde(rename = "eventStreamTerminated")]
    EventStreamTerminated(EventStreamTerminated),
    /// The server is about to shut down; reconnect (sent without a
    /// `subscriptionId`).
    #[serde(rename = "serverShutdown")]
    ServerShutdown(EventStreamTerminated),
    /// An event type this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

/// Balances that changed (`outboundAccountPosition`).
#[derive(Debug, Deserialize, PartialEq)]
pub struct OutboundAccountPosition {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Time of the last account update.
    #[serde(rename = "u")]
    pub last_update_time: Timestamp,
    #[serde(rename = "B")]
    pub balances: Vec<AccountBalance>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AccountBalance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "f")]
    pub free: Decimal,
    #[serde(rename = "l")]
    pub locked: Decimal,
}

/// Deposit, withdrawal or transfer (`balanceUpdate`).
#[derive(Debug, Deserialize, PartialEq)]
pub struct BalanceUpdate {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "d")]
    pub delta: Decimal,
    #[serde(rename = "T")]
    pub clear_time: Timestamp,
}

/// Balance locked or unlocked by an external system, e.g. as margin
/// collateral (`externalLockUpdate`).
#[derive(Debug, Deserialize, PartialEq)]
pub struct ExternalLockUpdate {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "d")]
    pub delta: Decimal,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EventStreamTerminated {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
}

/// Order update (`executionReport`). Average price is
/// `cumulative_quote_qty / cumulative_filled_qty`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ExecutionReport {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "o")]
    pub order_type: OrderType,
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "P")]
    pub stop_price: Decimal,
    #[serde(rename = "F")]
    pub iceberg_qty: Decimal,
    /// `-1` when the order is not part of an order list.
    #[serde(rename = "g")]
    pub order_list_id: i64,
    /// Id of the order being canceled.
    #[serde(rename = "C")]
    pub orig_client_order_id: String,
    #[serde(rename = "x")]
    pub execution_type: ExecutionType,
    #[serde(rename = "X")]
    pub order_status: OrderStatus,
    /// `NONE`, or the reason the order was rejected.
    #[serde(rename = "r")]
    pub reject_reason: String,
    #[serde(rename = "i")]
    pub order_id: i64,
    #[serde(rename = "l")]
    pub last_executed_qty: Decimal,
    #[serde(rename = "z")]
    pub cumulative_filled_qty: Decimal,
    #[serde(rename = "L")]
    pub last_executed_price: Decimal,
    #[serde(rename = "n")]
    pub commission: Decimal,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    /// `-1` when the event is not a trade.
    #[serde(rename = "t")]
    pub trade_id: i64,
    #[serde(rename = "I")]
    pub execution_id: i64,
    /// The order is on the book.
    #[serde(rename = "w")]
    pub is_working: bool,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "M")]
    pub ignore: bool,
    #[serde(rename = "O")]
    pub order_creation_time: Timestamp,
    #[serde(rename = "Z")]
    pub cumulative_quote_qty: Decimal,
    #[serde(rename = "Y")]
    pub last_quote_qty: Decimal,
    #[serde(rename = "Q")]
    pub quote_order_qty: Decimal,
    #[serde(rename = "V")]
    pub self_trade_prevention_mode: STPMode,

    // Conditional fields: present only when the condition applies.
    /// Only while the order is working on the book.
    #[serde(rename = "W", default)]
    pub working_time: Option<Timestamp>,
    /// Trailing stop orders.
    #[serde(rename = "d", default)]
    pub trailing_delta: Option<i64>,
    #[serde(rename = "D", default)]
    pub trailing_time: Option<Timestamp>,
    #[serde(rename = "j", default)]
    pub strategy_id: Option<i64>,
    #[serde(rename = "J", default)]
    pub strategy_type: Option<i64>,
    /// Orders expired due to STP.
    #[serde(rename = "v", default)]
    pub prevented_match_id: Option<i64>,
    #[serde(rename = "A", default)]
    pub prevented_quantity: Option<Decimal>,
    #[serde(rename = "B", default)]
    pub last_prevented_quantity: Option<Decimal>,
    #[serde(rename = "u", default)]
    pub trade_group_id: Option<i64>,
    #[serde(rename = "U", default)]
    pub counter_order_id: Option<i64>,
    #[serde(rename = "Cs", default)]
    pub counter_symbol: Option<String>,
    #[serde(rename = "pl", default)]
    pub prevented_execution_qty: Option<Decimal>,
    #[serde(rename = "pL", default)]
    pub prevented_execution_price: Option<Decimal>,
    #[serde(rename = "pY", default)]
    pub prevented_execution_quote_qty: Option<Decimal>,
    /// Orders with allocations.
    #[serde(rename = "b", default)]
    pub match_type: Option<String>,
    #[serde(rename = "a", default)]
    pub allocation_id: Option<i64>,
    #[serde(rename = "k", default)]
    pub working_floor: Option<String>,
    #[serde(rename = "uS", default)]
    pub used_sor: Option<bool>,
    /// Pegged orders.
    #[serde(rename = "gP", default)]
    pub peg_price_type: Option<PegPriceType>,
    #[serde(rename = "gOT", default)]
    pub peg_offset_type: Option<PegOffsetType>,
    #[serde(rename = "gOV", default)]
    pub peg_offset_value: Option<i64>,
    #[serde(rename = "gp", default)]
    pub pegged_price: Option<Decimal>,
    /// Expired orders.
    #[serde(rename = "eR", default)]
    pub expiry_reason: Option<ExpiryReason>,
}

/// Order list update (`listStatus`), sent alongside `executionReport`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ListStatus {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "g")]
    pub order_list_id: i64,
    #[serde(rename = "c")]
    pub contingency_type: ContingencyType,
    #[serde(rename = "l")]
    pub list_status_type: OrderListStatus,
    #[serde(rename = "L")]
    pub list_order_status: OrderListOrderStatus,
    #[serde(rename = "r")]
    pub list_reject_reason: String,
    #[serde(rename = "C")]
    pub list_client_order_id: String,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "O")]
    pub orders: Vec<ListStatusOrder>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ListStatusOrder {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub order_id: i64,
    #[serde(rename = "c")]
    pub client_order_id: String,
}
