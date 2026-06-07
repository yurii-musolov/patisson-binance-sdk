//! Events pushed over a margin user data WebSocket stream.
//!
//! Margin shares the spot user data event shapes — `outboundAccountPosition`,
//! `balanceUpdate`, `executionReport`, `listStatus` — with `executionReport`
//! gaining an optional `isolatedSymbol` field when the order is placed in an
//! isolated margin account.

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    Timestamp,
    margin::{OrderSide, OrderStatus, OrderType, TimeInForce},
    ws::ReceivedMessage,
};

/// Top-level frame received on a margin user data stream. The wire format is
/// internally tagged by the `e` (event) field.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "e")]
#[allow(clippy::large_enum_variant)]
pub enum IncomingMessage {
    #[serde(rename = "outboundAccountPosition")]
    OutboundAccountPosition(OutboundAccountPositionEvent),
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdateEvent),
    #[serde(rename = "executionReport")]
    ExecutionReport(ExecutionReportEvent),
}

impl ReceivedMessage for IncomingMessage {
    fn server_shutdown_event_time(&self) -> Option<u64> {
        // Margin user data streams don't emit an app-level shutdown event;
        // the connection is dropped at the protocol level when the listenKey
        // expires. The shared driver handles that via its heartbeat / close
        // path.
        None
    }
}

/// Account-wide balance snapshot pushed whenever a balance changes.
///
/// Carries the full per-asset state — easier to consume than reconciling
/// individual `BalanceUpdate` deltas.
#[derive(PartialEq, Deserialize, Debug)]
pub struct OutboundAccountPositionEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Account last-update time (matches the corresponding REST snapshot).
    #[serde(rename = "u")]
    pub last_account_update: Timestamp,
    #[serde(rename = "B")]
    pub balances: Vec<AccountBalance>,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct AccountBalance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "f")]
    pub free: Decimal,
    #[serde(rename = "l")]
    pub locked: Decimal,
}

/// Single-asset balance change (deposit, withdrawal, transfer between wallets).
#[derive(PartialEq, Deserialize, Debug)]
pub struct BalanceUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "a")]
    pub asset: String,
    /// Signed delta — positive for credits, negative for debits.
    #[serde(rename = "d")]
    pub delta: Decimal,
    /// Clear time (when the balance change was applied).
    #[serde(rename = "T")]
    pub clear_time: Timestamp,
}

/// Order lifecycle event: NEW, TRADE, CANCELED, REPLACED, REJECTED, EXPIRED,
/// TRADE_PREVENTION.
///
/// One event is emitted for every state transition; consume the stream to
/// reconcile order state without polling.
#[derive(PartialEq, Deserialize, Debug)]
pub struct ExecutionReportEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    /// Caller-supplied client order id.
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "o")]
    pub order_type: OrderType,
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    /// Order quantity (base asset).
    #[serde(rename = "q")]
    pub orig_qty: Decimal,
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Stop price (for STOP_LOSS / TAKE_PROFIT variants).
    #[serde(rename = "P")]
    pub stop_price: Decimal,
    /// Iceberg quantity.
    #[serde(rename = "F")]
    pub iceberg_qty: Decimal,
    /// Original client order id (set on cancel / replace).
    #[serde(rename = "C")]
    pub orig_client_order_id: String,
    /// Current execution type — NEW / CANCELED / REPLACED / REJECTED / TRADE /
    /// EXPIRED / TRADE_PREVENTION. Returned as a raw string to keep this
    /// resilient to new variants Binance may add.
    #[serde(rename = "x")]
    pub execution_type: String,
    #[serde(rename = "X")]
    pub order_status: OrderStatus,
    /// Reject reason (or `"NONE"`).
    #[serde(rename = "r")]
    pub reject_reason: String,
    #[serde(rename = "i")]
    pub order_id: i64,
    /// Quantity executed in this trade (0 if no fill).
    #[serde(rename = "l")]
    pub last_executed_qty: Decimal,
    /// Cumulative filled quantity.
    #[serde(rename = "z")]
    pub cumulative_filled_qty: Decimal,
    /// Price of this trade (0 if no fill).
    #[serde(rename = "L")]
    pub last_executed_price: Decimal,
    /// Commission for this fill.
    #[serde(rename = "n")]
    pub commission: Decimal,
    /// Commission asset — null if no fill.
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    /// Trade id — -1 if no fill.
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Whether the order is currently on the book.
    #[serde(rename = "w")]
    pub is_on_book: bool,
    /// Whether this trade was the maker side.
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "O")]
    pub order_creation_time: Timestamp,
    /// Cumulative quote-asset value transacted.
    #[serde(rename = "Z")]
    pub cumulative_quote_qty: Decimal,
    /// Quote-asset value of this fill.
    #[serde(rename = "Y")]
    pub last_quote_qty: Decimal,
    /// Quote order qty (when the order was placed by quote amount).
    #[serde(rename = "Q")]
    pub quote_order_qty: Decimal,
    /// Margin-only: symbol of the isolated margin account this order belongs
    /// to. Absent (== None) for cross-margin orders.
    #[serde(rename = "isolatedSymbol", default)]
    pub isolated_symbol: Option<String>,
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use super::*;
    use crate::serde::deserialize_json;

    #[test]
    fn deserialize_outbound_account_position() {
        let json = r#"{
            "e": "outboundAccountPosition",
            "E": 1564034571105,
            "u": 1564034571073,
            "B": [
                {"a": "ETH", "f": "10000.000000", "l": "0.000000"},
                {"a": "USDT", "f": "1000.50", "l": "100.00"}
            ]
        }"#;
        let parsed: IncomingMessage = deserialize_json(json).unwrap();
        let IncomingMessage::OutboundAccountPosition(event) = parsed else {
            panic!("expected OutboundAccountPosition, got {parsed:?}");
        };
        assert_eq!(event.event_time, 1564034571105);
        assert_eq!(event.last_account_update, 1564034571073);
        assert_eq!(event.balances.len(), 2);
        assert_eq!(event.balances[1].asset, "USDT");
        assert_eq!(event.balances[1].free, dec!(1000.50));
        assert_eq!(event.balances[1].locked, dec!(100.00));
    }

    #[test]
    fn deserialize_balance_update() {
        let json = r#"{
            "e": "balanceUpdate",
            "E": 1573200697110,
            "a": "BTC",
            "d": "100.00000000",
            "T": 1573200697068
        }"#;
        let parsed: IncomingMessage = deserialize_json(json).unwrap();
        let IncomingMessage::BalanceUpdate(event) = parsed else {
            panic!("expected BalanceUpdate, got {parsed:?}");
        };
        assert_eq!(event.asset, "BTC");
        assert_eq!(event.delta, dec!(100.0));
    }

    #[test]
    fn deserialize_execution_report_cross_margin_omits_isolated_symbol() {
        let json = r#"{
            "e": "executionReport",
            "E": 1499405658658,
            "s": "ETHBTC",
            "c": "myOrderId",
            "S": "BUY",
            "o": "LIMIT",
            "f": "GTC",
            "q": "1.00000000",
            "p": "0.10264410",
            "P": "0.00000000",
            "F": "0.00000000",
            "C": "",
            "x": "NEW",
            "X": "NEW",
            "r": "NONE",
            "i": 4293153,
            "l": "0.00000000",
            "z": "0.00000000",
            "L": "0.00000000",
            "n": "0",
            "N": null,
            "T": 1499405658657,
            "t": -1,
            "w": true,
            "m": false,
            "O": 1499405658657,
            "Z": "0.00000000",
            "Y": "0.00000000",
            "Q": "0.00000000"
        }"#;
        let parsed: IncomingMessage = deserialize_json(json).unwrap();
        let IncomingMessage::ExecutionReport(event) = parsed else {
            panic!("expected ExecutionReport, got {parsed:?}");
        };
        assert_eq!(event.symbol, "ETHBTC");
        assert_eq!(event.order_id, 4293153);
        assert_eq!(event.order_status, OrderStatus::New);
        assert_eq!(event.execution_type, "NEW");
        assert_eq!(event.commission_asset, None);
        assert_eq!(event.isolated_symbol, None);
    }

    #[test]
    fn deserialize_execution_report_isolated_margin_carries_isolated_symbol() {
        // Same as above plus `isolatedSymbol`.
        let json = r#"{
            "e": "executionReport",
            "E": 1499405658658,
            "s": "ETHBTC",
            "c": "myOrderId",
            "S": "SELL",
            "o": "MARKET",
            "f": "IOC",
            "q": "1.00000000",
            "p": "0.00000000",
            "P": "0.00000000",
            "F": "0.00000000",
            "C": "",
            "x": "TRADE",
            "X": "FILLED",
            "r": "NONE",
            "i": 4293153,
            "l": "1.00000000",
            "z": "1.00000000",
            "L": "0.10264410",
            "n": "0.00010264",
            "N": "BTC",
            "T": 1499405658657,
            "t": 12345,
            "w": false,
            "m": true,
            "O": 1499405658657,
            "Z": "0.10264410",
            "Y": "0.10264410",
            "Q": "0.00000000",
            "isolatedSymbol": "ETHBTC"
        }"#;
        let parsed: IncomingMessage = deserialize_json(json).unwrap();
        let IncomingMessage::ExecutionReport(event) = parsed else {
            panic!("expected ExecutionReport, got {parsed:?}");
        };
        assert_eq!(event.isolated_symbol.as_deref(), Some("ETHBTC"));
        assert_eq!(event.commission_asset.as_deref(), Some("BTC"));
        assert!(event.is_maker);
    }
}
