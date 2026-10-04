//! Algo Order API (`/fapi/v1/algoOrder` and friends): conditional orders
//! (`STOP`, `TAKE_PROFIT`, `STOP_MARKET`, `TAKE_PROFIT_MARKET`,
//! `TRAILING_STOP_MARKET`), which `POST /fapi/v1/order` no longer documents.
//!
//! Modelled after Binance's API specification as shipped in the official
//! `binance-connector-rust` (5ff71b4, 2026-10-01).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    derivatives::usds_margined_futures::{
        AlgoType, OrderResponseType, OrderSide, OrderType, PositionSide, PriceMatch, STPMode,
        TimeInForce, WorkingType,
    },
    serde::{decimal_opt_lenient, string_opt_lenient},
};

/// `POST /fapi/v1/algoOrder`.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewAlgoOrderRequest {
    algo_type: AlgoType,
    symbol: String,
    side: OrderSide,
    #[serde(rename = "type")]
    order_type: OrderType,
    position_side: Option<PositionSide>,
    time_in_force: Option<TimeInForce>,
    /// Not with `close_position`.
    quantity: Option<Decimal>,
    price: Option<Decimal>,
    trigger_price: Option<Decimal>,
    working_type: Option<WorkingType>,
    price_match: Option<PriceMatch>,
    /// Close the whole position when triggered (`STOP_MARKET`,
    /// `TAKE_PROFIT_MARKET`); not with `quantity` or `reduce_only`.
    close_position: Option<bool>,
    price_protect: Option<bool>,
    reduce_only: Option<bool>,
    /// `TRAILING_STOP_MARKET` activation price.
    activate_price: Option<Decimal>,
    /// `TRAILING_STOP_MARKET` callback rate in percent.
    callback_rate: Option<Decimal>,
    client_algo_id: Option<String>,
    new_order_resp_type: Option<OrderResponseType>,
    self_trade_prevention_mode: Option<STPMode>,
    good_till_date: Option<Timestamp>,
    recv_window: Option<u64>,
}

impl NewAlgoOrderRequest {
    /// A conditional order (`algoType=CONDITIONAL`).
    pub fn conditional(symbol: impl Into<String>, side: OrderSide, order_type: OrderType) -> Self {
        Self {
            algo_type: AlgoType::Conditional,
            symbol: symbol.into(),
            side,
            order_type,
            position_side: None,
            time_in_force: None,
            quantity: None,
            price: None,
            trigger_price: None,
            working_type: None,
            price_match: None,
            close_position: None,
            price_protect: None,
            reduce_only: None,
            activate_price: None,
            callback_rate: None,
            client_algo_id: None,
            new_order_resp_type: None,
            self_trade_prevention_mode: None,
            good_till_date: None,
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
    pub fn price(mut self, value: Decimal) -> Self {
        self.price = Some(value);
        self
    }
    pub fn trigger_price(mut self, value: Decimal) -> Self {
        self.trigger_price = Some(value);
        self
    }
    pub fn working_type(mut self, value: WorkingType) -> Self {
        self.working_type = Some(value);
        self
    }
    pub fn price_match(mut self, value: PriceMatch) -> Self {
        self.price_match = Some(value);
        self
    }
    pub fn close_position(mut self, value: bool) -> Self {
        self.close_position = Some(value);
        self
    }
    pub fn price_protect(mut self, value: bool) -> Self {
        self.price_protect = Some(value);
        self
    }
    pub fn reduce_only(mut self, value: bool) -> Self {
        self.reduce_only = Some(value);
        self
    }
    pub fn activate_price(mut self, value: Decimal) -> Self {
        self.activate_price = Some(value);
        self
    }
    pub fn callback_rate(mut self, value: Decimal) -> Self {
        self.callback_rate = Some(value);
        self
    }
    pub fn client_algo_id(mut self, value: impl Into<String>) -> Self {
        self.client_algo_id = Some(value.into());
        self
    }
    pub fn new_order_resp_type(mut self, value: OrderResponseType) -> Self {
        self.new_order_resp_type = Some(value);
        self
    }
    pub fn self_trade_prevention_mode(mut self, value: STPMode) -> Self {
        self.self_trade_prevention_mode = Some(value);
        self
    }
    pub fn good_till_date(mut self, value: Timestamp) -> Self {
        self.good_till_date = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

/// Identifies one algo order: by `algoId` or by `clientAlgoId`.
/// Used by `DELETE` and `GET /fapi/v1/algoOrder`.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlgoOrderIdParams {
    algo_id: Option<i64>,
    client_algo_id: Option<String>,
    recv_window: Option<u64>,
}

impl AlgoOrderIdParams {
    pub fn by_algo_id(algo_id: i64) -> Self {
        Self {
            algo_id: Some(algo_id),
            client_algo_id: None,
            recv_window: None,
        }
    }
    pub fn by_client_algo_id(client_algo_id: impl Into<String>) -> Self {
        Self {
            algo_id: None,
            client_algo_id: Some(client_algo_id.into()),
            recv_window: None,
        }
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

/// `GET /fapi/v1/openAlgoOrders`.
#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetOpenAlgoOrdersParams {
    algo_type: Option<AlgoType>,
    pub(crate) symbol: Option<String>,
    algo_id: Option<i64>,
    recv_window: Option<u64>,
}

impl GetOpenAlgoOrdersParams {
    /// Every symbol (weight 40); narrow it with [`Self::symbol`] (weight 1).
    pub fn new() -> Self {
        Self::default()
    }
    pub fn algo_type(mut self, value: AlgoType) -> Self {
        self.algo_type = Some(value);
        self
    }
    pub fn symbol(mut self, value: impl Into<String>) -> Self {
        self.symbol = Some(value.into());
        self
    }
    pub fn algo_id(mut self, value: i64) -> Self {
        self.algo_id = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

/// `GET /fapi/v1/allAlgoOrders`.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetAllAlgoOrdersParams {
    symbol: String,
    algo_id: Option<i64>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    limit: Option<u32>,
    recv_window: Option<u64>,
}

impl GetAllAlgoOrdersParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            algo_id: None,
            start_time: None,
            end_time: None,
            limit: None,
            recv_window: None,
        }
    }
    pub fn algo_id(mut self, value: i64) -> Self {
        self.algo_id = Some(value);
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
    pub fn limit(mut self, value: u32) -> Self {
        self.limit = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

/// `DELETE /fapi/v1/algoOpenOrders`.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelAllAlgoOpenOrdersParams {
    symbol: String,
    recv_window: Option<u64>,
}

impl CancelAllAlgoOpenOrdersParams {
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

/// An algo order, as returned by every Algo Order endpoint.
///
/// Placeholder values Binance uses for "not set" (`""`, `"null"`) are read
/// as `None`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlgoOrder {
    pub algo_id: i64,
    pub client_algo_id: String,
    pub algo_type: AlgoType,
    pub order_type: OrderType,
    pub symbol: String,
    pub side: OrderSide,
    pub position_side: PositionSide,
    pub time_in_force: TimeInForce,
    pub quantity: Decimal,
    /// e.g. `NEW`, `CANCELED` (Binance doesn't specify the full list).
    pub algo_status: String,
    pub trigger_price: Decimal,
    pub price: Decimal,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub iceberg_quantity: Option<Decimal>,
    pub self_trade_prevention_mode: STPMode,
    pub working_type: WorkingType,
    pub price_match: PriceMatch,
    pub close_position: bool,
    pub price_protect: bool,
    pub reduce_only: bool,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub activate_price: Option<Decimal>,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub callback_rate: Option<Decimal>,
    pub create_time: Timestamp,
    pub update_time: Timestamp,
    /// `0` until triggered.
    pub trigger_time: Timestamp,
    /// `0` unless `time_in_force` is `GTD`.
    pub good_till_date: Timestamp,
    /// Id of the order placed when the algo order triggered.
    #[serde(default, deserialize_with = "string_opt_lenient")]
    pub actual_order_id: Option<String>,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub actual_price: Option<Decimal>,
    #[serde(default, deserialize_with = "string_opt_lenient")]
    pub actual_type: Option<String>,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub actual_qty: Option<Decimal>,
    #[serde(default, deserialize_with = "string_opt_lenient")]
    pub tp_order_type: Option<String>,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub tp_trigger_price: Option<Decimal>,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub tp_price: Option<Decimal>,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub sl_trigger_price: Option<Decimal>,
    #[serde(default, deserialize_with = "decimal_opt_lenient")]
    pub sl_price: Option<Decimal>,
}

/// Result of `DELETE /fapi/v1/algoOrder` (`code` is `"200"` on success).
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanceledAlgoOrder {
    pub algo_id: i64,
    pub client_algo_id: String,
    #[serde(deserialize_with = "crate::serde::u64_from_number_or_string")]
    pub code: u64,
    pub msg: String,
}

/// Result of `DELETE /fapi/v1/algoOpenOrders`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct AlgoActionResult {
    #[serde(deserialize_with = "crate::serde::u64_from_number_or_string")]
    pub code: u64,
    pub msg: String,
}
