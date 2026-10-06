//! Enum definitions for the Binance USDⓈ-M Futures API.
//!
//! These apply to both REST and WebSocket endpoints.

// Wire models: names mirror Binance's documentation field by field; they are
// documented where the meaning isn't obvious. Full coverage comes later.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum OrderSide {
    BUY,
    SELL,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Position side. In Hedge Mode use LONG/SHORT; in One-Way Mode use BOTH.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    Both,
    Long,
    Short,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    Stop,
    StopMarket,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
    /// Liquidation order; only seen in user data stream order updates.
    #[serde(skip_serializing)]
    Liquidation,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum TimeInForce {
    /// Good Til Canceled.
    GTC,
    /// Immediate Or Cancel.
    IOC,
    /// Fill or Kill.
    FOK,
    /// Good Til Crossing (post-only).
    GTX,
    /// Good Till Date.
    GTD,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Order price match modes for USDⓈ-M Futures (`priceMatch` field).
///
/// When set on a LIMIT order, Binance will compute the actual price from
/// the current order book instead of taking the `price` field. Mutually
/// exclusive with `price`.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum PriceMatch {
    /// No price match — use the `price` field.
    #[serde(rename = "NONE")]
    None,
    /// Best opposite-side price.
    #[serde(rename = "OPPONENT")]
    Opponent,
    /// 5th opposite-side level.
    #[serde(rename = "OPPONENT_5")]
    Opponent5,
    /// 10th opposite-side level.
    #[serde(rename = "OPPONENT_10")]
    Opponent10,
    /// 20th opposite-side level.
    #[serde(rename = "OPPONENT_20")]
    Opponent20,
    /// Best same-side price.
    #[serde(rename = "QUEUE")]
    Queue,
    /// 5th same-side level.
    #[serde(rename = "QUEUE_5")]
    Queue5,
    /// 10th same-side level.
    #[serde(rename = "QUEUE_10")]
    Queue10,
    /// 20th same-side level.
    #[serde(rename = "QUEUE_20")]
    Queue20,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
    ExpiredInMatch,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum OrderResponseType {
    ACK,
    RESULT,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Trigger price source for STOP / TAKE_PROFIT orders.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkingType {
    MarkPrice,
    ContractPrice,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarginType {
    /// The user data stream sends `isolated` in `ACCOUNT_UPDATE`.
    #[serde(alias = "isolated")]
    Isolated,
    /// The user data stream sends `cross` in `ACCOUNT_UPDATE`.
    #[serde(alias = "cross", alias = "crossed")]
    Crossed,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractType {
    Perpetual,
    CurrentMonth,
    NextMonth,
    CurrentQuarter,
    NextQuarter,
    PerpetualDelivering,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    PendingTrading,
    Trading,
    PreDelivering,
    Delivering,
    Delivered,
    PreSettle,
    Settling,
    Close,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Self-trade prevention mode.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum STPMode {
    None,
    ExpireTaker,
    ExpireMaker,
    ExpireBoth,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimiter {
    RequestWeight,
    Orders,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitInterval {
    Second,
    Minute,
    Day,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum KlineInterval {
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

/// Algo order family (`algoType`).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlgoType {
    /// Conditional orders: `STOP`, `TAKE_PROFIT`, `STOP_MARKET`,
    /// `TAKE_PROFIT_MARKET`, `TRAILING_STOP_MARKET`.
    Conditional,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}
