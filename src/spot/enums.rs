//! ENUM Definitions
//!
//! This will apply for both REST API and WebSocket API.

use serde::{Deserialize, Serialize};

/// Symbol status.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    Trading,
    EndOfDay,
    Halt,
    Break,
}

/// Account and Symbol Permissions.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum AccountAndSymbolPermissions {
    #[serde(rename = "SPOT")]
    Spot,
    #[serde(rename = "MARGIN")]
    Margin,
    #[serde(rename = "LEVERAGED")]
    Leveraged,
    #[serde(rename = "TRD_GRP_002")]
    TrdGrp002,
    #[serde(rename = "TRD_GRP_003")]
    TrdGrp003,
    #[serde(rename = "TRD_GRP_004")]
    TrdGrp004,
    #[serde(rename = "TRD_GRP_005")]
    TrdGrp005,
    #[serde(rename = "TRD_GRP_006")]
    TrdGrp006,
    #[serde(rename = "TRD_GRP_007")]
    TrdGrp007,
    #[serde(rename = "TRD_GRP_008")]
    TrdGrp008,
    #[serde(rename = "TRD_GRP_009")]
    TrdGrp009,
    #[serde(rename = "TRD_GRP_010")]
    TrdGrp010,
    #[serde(rename = "TRD_GRP_011")]
    TrdGrp011,
    #[serde(rename = "TRD_GRP_012")]
    TrdGrp012,
    #[serde(rename = "TRD_GRP_013")]
    TrdGrp013,
    #[serde(rename = "TRD_GRP_014")]
    TrdGrp014,
    #[serde(rename = "TRD_GRP_015")]
    TrdGrp015,
    #[serde(rename = "TRD_GRP_016")]
    TrdGrp016,
    #[serde(rename = "TRD_GRP_017")]
    TrdGrp017,
    #[serde(rename = "TRD_GRP_018")]
    TrdGrp018,
    #[serde(rename = "TRD_GRP_019")]
    TrdGrp019,
    #[serde(rename = "TRD_GRP_020")]
    TrdGrp020,
    #[serde(rename = "TRD_GRP_021")]
    TrdGrp021,
    #[serde(rename = "TRD_GRP_022")]
    TrdGrp022,
    #[serde(rename = "TRD_GRP_023")]
    TrdGrp023,
    #[serde(rename = "TRD_GRP_024")]
    TrdGrp024,
    #[serde(rename = "TRD_GRP_025")]
    TrdGrp025,
}

/// Order status.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// The order has been accepted by the engine.
    New,
    /// The order is in a pending phase until the working order of an order list has been fully filled.
    PendingNew,
    /// A part of the order has been filled.
    PartiallyFilled,
    /// The order has been completed.
    Filled,
    /// The order has been canceled by the user.
    Canceled,
    /// Currently unused
    PendingCancel,
    /// The order was not accepted by the engine and not processed.
    Rejected,
    /// The order was canceled according to the order type's rules (e.g. LIMIT FOK orders with no fill, LIMIT IOC or MARKET orders that partially fill)
    /// or by the exchange, (e.g. orders canceled during liquidation, orders canceled during maintenance)
    Expired,
    /// The order was expired by the exchange due to STP. (e.g. an order with EXPIRE_TAKER will match with existing orders on the book with the same account or same tradeGroupId)
    ExpiredInMatch,
}

/// Order List Status.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderListStatus {
    /// This is used when the ListStatus is responding to a failed action. (E.g. order list placement or cancellation)
    Response,
    /// The order list has been placed or there is an update to the order list status.
    ExecStarted,
    /// The clientOrderId of an order in the order list has been changed.
    Updated,
    /// The order list has finished executing and thus is no longer active.
    AllDone,
}

/// Order List Order Status.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderListOrderStatus {
    /// Either an order list has been placed or there is an update to the status of the list.
    Executing,
    /// An order list has completed execution and thus no longer active.
    AllDone,
    /// The List Status is responding to a failed action either during order placement or order canceled.
    Reject,
}

/// ContingencyType
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ContingencyType {
    OCO,
    OTO,
}

/// Allocation type.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum AllocationType {
    SOR,
}

/// Order types.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderTypes {
    Limit,
    Market,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
}

/// Order Response Type.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum OrderResponseType {
    ACK,
    RESULT,
    FULL,
}

/// Working Floor.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum WorkingFloor {
    EXCHANGE,
    SOR,
}

/// Order side.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum OrderSide {
    BUY,
    SELL,
}

/// Time in force.
/// This sets how long an order will be active before expiration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum TimeInForce {
    /// Good Til Canceled
    /// An order will be on the book unless the order is canceled.
    GTC,
    /// Immediate Or Cancel
    /// An order will try to fill the order as much as it can before the order expires.
    IOC,
    /// Fill or Kill
    /// An order will expire if the full order cannot be filled upon execution.
    FOK,
}

/// Rate limiter.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimiter {
    RequestWeight,
    Orders,
    RawRequests,
}

/// Rate limit interval.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitInterval {
    Second,
    Minute,
    Day,
}

/// STP Mode.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum STPMode {
    None,
    ExpireMaker,
    ExpireTaker,
    ExpireBoth,
    Decrement,
}

/// Rate Limit
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    rate_limit_type: RateLimiter,
    interval: RateLimitInterval,
    interval_num: u32,
    limit: u32,
}
