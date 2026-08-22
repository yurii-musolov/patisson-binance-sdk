//! Enum definitions for the Binance Margin Trading API.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum OrderSide {
    BUY,
    SELL,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum TimeInForce {
    /// Good Til Canceled.
    GTC,
    /// Immediate Or Cancel.
    IOC,
    /// Fill or Kill.
    FOK,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    PendingCancel,
    Rejected,
    Expired,
    ExpiredInMatch,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum OrderResponseType {
    ACK,
    RESULT,
    FULL,
}

/// Order list contingency type — currently Binance only issues `OCO`.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ContingencyType {
    OCO,
}

/// Order List Status — top-level lifecycle state of an order list.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderListStatus {
    /// Responding to a failed action (e.g. order list placement or cancellation).
    Response,
    /// The order list has been placed or there is an update to its status.
    ExecStarted,
    /// The order list has finished executing and is no longer active.
    AllDone,
}

/// Order List Order Status — aggregate status of the list's legs.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderListOrderStatus {
    /// The order list has been placed or there is an update to its status.
    Executing,
    /// The order list has completed execution and is no longer active.
    AllDone,
    /// The list status is responding to a failed action, either during
    /// order list placement or cancellation.
    Reject,
}

/// Margin order side-effect — controls auto-borrow/repay behaviour.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SideEffectType {
    NoSideEffect,
    MarginBuy,
    AutoRepay,
    AutoBorrowRepay,
}

/// Self-trade prevention mode.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum STPMode {
    None,
    ExpireMaker,
    ExpireTaker,
    ExpireBoth,
    Decrement,
}

/// Direction of a borrow/repay execution against the margin account.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BorrowRepayType {
    Borrow,
    Repay,
}

/// Whether a margin operation targets the isolated or cross-margin account.
///
/// Serialised as the string `"TRUE"` / `"FALSE"` that Binance expects.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum IsIsolated {
    True,
    False,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarginLevelStatus {
    Excessive,
    Normal,
    MarginCall,
    PreLiquidation,
    ForceLiquidation,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum KlineInterval {
    #[serde(rename = "1s")]
    Second1,
    #[serde(rename = "1m")]
    Minute1,
    #[serde(rename = "3m")]
    Minute3,
    #[serde(rename = "5m")]
    Minute5,
    #[serde(rename = "15m")]
    Minute15,
    #[serde(rename = "30m")]
    Minute30,
    #[serde(rename = "1h")]
    Hour1,
    #[serde(rename = "2h")]
    Hour2,
    #[serde(rename = "4h")]
    Hour4,
    #[serde(rename = "6h")]
    Hour6,
    #[serde(rename = "8h")]
    Hour8,
    #[serde(rename = "12h")]
    Hour12,
    #[serde(rename = "1d")]
    Day1,
    #[serde(rename = "3d")]
    Day3,
    #[serde(rename = "1w")]
    Week1,
    #[serde(rename = "1M")]
    Month1,
}
