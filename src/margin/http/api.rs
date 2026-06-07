use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    margin::{
        IsIsolated, MarginLevelStatus, OrderResponseType, OrderSide, OrderStatus, OrderType,
        STPMode, SideEffectType, TimeInForce,
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

// ===== Margin metadata =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAllMarginAssetsParams {
    asset: Option<String>,
    recv_window: Option<i64>,
}

impl GetAllMarginAssetsParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn asset(mut self, value: impl Into<String>) -> Self {
        self.asset = Some(value.into());
        self
    }

    pub fn recv_window(mut self, value: i64) -> Self {
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
    recv_window: Option<i64>,
}

impl GetMarginAccountParams {
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
    recv_window: Option<i64>,
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
    pub fn recv_window(mut self, value: i64) -> Self {
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
    recv_window: Option<i64>,
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
    pub fn recv_window(mut self, value: i64) -> Self {
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
    recv_window: Option<i64>,
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
    pub fn recv_window(mut self, value: i64) -> Self {
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
        let json = r#"{"listenKey":"pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"}"#;
        let parsed: ListenKey = deserialize_json(json).unwrap();
        assert_eq!(parsed.listen_key.len(), 64);
    }

    #[test]
    fn deserialize_empty_response() {
        let json = r#"{}"#;
        let parsed: EmptyResponse = deserialize_json(json).unwrap();
        assert_eq!(parsed, EmptyResponse {});
    }
}
