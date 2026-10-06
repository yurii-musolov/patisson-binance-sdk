//! `OrderState` / `Orders` fed with order events built from the official
//! documentation examples (`tests/fixtures/*user_data*`).

use binance::{
    OrderStatus, Orders,
    derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm},
    spot::ws_api::ExecutionReport,
};
use rust_decimal::dec;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn fixture(path: &str) -> Value {
    let path = format!("{}/tests/fixtures/{path}", env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&json).unwrap()
}

/// `base` with `fields` overwritten, parsed as `T`.
fn with<T: DeserializeOwned>(base: &Value, fields: Value) -> T {
    let mut v = base.clone();
    for (k, val) in fields.as_object().unwrap() {
        v[k] = val.clone();
    }
    serde_json::from_value(v).unwrap()
}

#[test]
fn spot_new_partial_fill_cancel() {
    let base = fixture("spot_user_data/event_executionReport.json")["event"].clone();
    let new: ExecutionReport = with(&base, json!({}));
    let trade: ExecutionReport = with(
        &base,
        json!({"x": "TRADE", "X": "PARTIALLY_FILLED", "l": "0.4", "z": "0.4",
               "L": "0.1", "Z": "0.04", "n": "0.0004", "N": "BNB", "t": 1001, "T": 1499405658700u64}),
    );
    // The cancel carries the cancel request's id in `c`, the order's in `C`.
    let cancel: ExecutionReport = with(
        &base,
        json!({"x": "CANCELED", "X": "CANCELED", "c": "cancel-request", "C": "mUvoqJxFIILMdfAW5iGSOW",
               "z": "0.4", "Z": "0.04", "T": 1499405658800u64}),
    );

    let mut orders = Orders::new();
    orders.apply(&new);
    orders.apply(&trade);
    orders.apply(&trade); // repeated after a reconnect
    let o = orders.apply(&cancel);
    assert_eq!(o.status, OrderStatus::Canceled);
    assert_eq!(o.client_order_id, "mUvoqJxFIILMdfAW5iGSOW");
    assert_eq!(o.filled_qty, dec!(0.4));
    assert_eq!(o.average_price, Some(dec!(0.1)));
    assert_eq!(o.fills.len(), 1);
    assert_eq!(o.commissions["BNB"], dec!(0.0004));
    assert!(orders.get_by_client_id("mUvoqJxFIILMdfAW5iGSOW").is_some());
    assert!(orders.get_by_client_id("cancel-request").is_none());
}

#[test]
fn usdm_order_trade_update() {
    let base = fixture("usdm_user_data/order_trade_update.json");
    let fill = |status: &str, last: &str, cum: &str, trade: u64| -> usdm::ws::UserDataMessage {
        let mut v = base.clone();
        let o = &mut v["o"];
        o["x"] = json!("TRADE");
        o["X"] = json!(status);
        o["q"] = json!("0.003");
        o["l"] = json!(last);
        o["z"] = json!(cum);
        o["L"] = json!("65000");
        o["ap"] = json!("65000");
        o["n"] = json!("0.01");
        o["t"] = json!(trade);
        serde_json::from_value(v).unwrap()
    };
    let events = [
        serde_json::from_value(base.clone()).unwrap(),
        fill("PARTIALLY_FILLED", "0.001", "0.001", 1),
        fill("FILLED", "0.002", "0.003", 2),
        fill("PARTIALLY_FILLED", "0.001", "0.001", 1), // late repeat
    ];
    let mut orders = Orders::new();
    for event in &events {
        if let usdm::ws::UserDataMessage::OrderTradeUpdate(e) = event {
            orders.apply(e);
        }
    }
    let o = orders.get("BTCUSDT", 8886774).unwrap();
    assert_eq!(o.status, OrderStatus::Filled);
    assert_eq!(o.filled_qty, dec!(0.003));
    assert_eq!(o.average_price, Some(dec!(65000)));
    assert_eq!(o.commissions["USDT"], dec!(0.02));
}

#[test]
fn coinm_order_trade_update() {
    let msg: coinm::ws::UserDataMessage =
        serde_json::from_value(fixture("coinm_user_data/order_trade_update.json")).unwrap();
    let coinm::ws::UserDataMessage::OrderTradeUpdate(e) = msg else {
        panic!()
    };
    let mut orders = Orders::new();
    let o = orders.apply(&e);
    assert_eq!((o.symbol.as_str(), o.order_id), ("BTCUSD_200925", 8888888));
    assert_eq!(o.status, OrderStatus::New);
    assert_eq!(o.average_price, None, "ap 0 means nothing filled");
    assert!(o.fills.is_empty());
}
