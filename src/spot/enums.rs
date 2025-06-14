//! ENUM Definitions
//!
//! This will apply for both REST API and WebSocket API.

use rust_decimal::Decimal;
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
pub enum OrderType {
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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

impl std::fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            Self::Second1 => "1s",
            Self::Minute1 => "1m",
            Self::Minute3 => "3m",
            Self::Minute5 => "5m",
            Self::Minute15 => "15m",
            Self::Minute30 => "30m",
            Self::Hour1 => "1h",
            Self::Hour2 => "2h",
            Self::Hour4 => "4h",
            Self::Hour6 => "6h",
            Self::Hour8 => "8h",
            Self::Hour12 => "12h",
            Self::Day1 => "1d",
            Self::Day3 => "3d",
            Self::Week1 => "1w",
            Self::Month1 => "1M",
        };
        write!(f, "{value}")
    }
}

/// Self trade prevention (STP) Mode.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum STPMode {
    None,
    ExpireMaker,
    ExpireTaker,
    ExpireBoth,
    Decrement,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityType {
    /// Endpoint can be accessed freely.
    None,
    /// Endpoint requires sending a valid API-Key and signature.
    Trade,
    /// Endpoint requires sending a valid API-Key and signature.
    UserData,
    /// Endpoint requires sending a valid API-Key.
    UserStream,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "filterType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExchangeFilter {
    PriceFilter { tick_size: Decimal },
    LotSize { step_size: Decimal },
    // TODO:
}
