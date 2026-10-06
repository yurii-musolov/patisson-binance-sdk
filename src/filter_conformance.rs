//! The `exchangeInfo` filters are internally tagged enums (`filterType`),
//! which `serde_ignored` can't look into, so the strict fixture tests would
//! miss a filter field the SDK doesn't model. Here every documented and
//! live filter is parsed, serialized back, and each input key must survive
//! the round trip.

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
    derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm},
    spot,
};

fn filters(json: &str, pointer: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(json).unwrap();
    match v.pointer(pointer) {
        Some(Value::Array(items)) => items.clone(),
        _ => Vec::new(),
    }
}

fn symbol_filters(json: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(json).unwrap();
    v["symbols"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|s| s["filters"].as_array().cloned().unwrap_or_default())
        .collect()
}

fn assert_round_trip<T: DeserializeOwned + Serialize>(source: &str, items: Vec<Value>) {
    assert!(!items.is_empty(), "{source}: no filters");
    for item in items {
        let parsed: T = serde_json::from_value(item.clone())
            .unwrap_or_else(|e| panic!("{source}: {e}: {item}"));
        let back = serde_json::to_value(&parsed).unwrap();
        let missing: Vec<_> = item
            .as_object()
            .unwrap()
            .keys()
            .filter(|k| back.get(k.as_str()).is_none())
            .collect();
        assert!(missing.is_empty(), "{source}: {item} loses {missing:?}");
    }
}

#[test]
fn spot_filters_keep_every_field() {
    let doc = include_str!("../tests/fixtures/spot/symbol_filters.json");
    assert_round_trip::<spot::http::Filter>("spot symbol_filters", filters(doc, ""));
    let doc = include_str!("../tests/fixtures/spot/exchange_filters.json");
    assert_round_trip::<spot::ExchangeFilter>("spot exchange_filters", filters(doc, ""));
    let live = include_str!("../tests/fixtures/spot/exchange_info_live.json");
    assert_round_trip::<spot::http::Filter>("spot live", symbol_filters(live));
}

// Futures: only the live responses. The specification examples merge the
// fields of every filter into one `PRICE_FILTER` object.

#[test]
fn usdm_filters_keep_every_field() {
    let live = include_str!("../tests/fixtures/usdm/exchange_info_live.json");
    assert_round_trip::<usdm::http::SymbolFilter>("usdm live", symbol_filters(live));
}

#[test]
fn coinm_filters_keep_every_field() {
    let live = include_str!("../tests/fixtures/coinm/exchange_info_live.json");
    assert_round_trip::<coinm::http::SymbolFilter>("coinm live", symbol_filters(live));
}
