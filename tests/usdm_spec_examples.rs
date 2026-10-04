//! Response examples from Binance's USD-M Futures API specification, as
//! shipped in the official `binance-connector-rust` (commit 5ff71b4,
//! 2026-10-01), parsed with the SDK types. Each example must deserialize
//! without error and without silently ignoring any field.
//!
//! Left out: the `userTrades` example carries COIN-M-only fields (`pair`,
//! `baseQty`) and the listenKey keepalive example echoes the key, which the
//! SDK deliberately discards.

use binance::derivatives::usds_margined_futures::http::{
    AccountBalance, AccountInformation, ActionResult, ExchangeInfo, ListenKey, MarkPrice,
    NewOrderResponse, Order, OrderBook, Position, PositionMode, ServerTime, SymbolOrderBookTicker,
};
use serde::de::DeserializeOwned;

fn parse<T: DeserializeOwned>(name: &str) -> T {
    let path = format!("{}/tests/fixtures/usdm/{name}", env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut ignored = Vec::new();
    let mut track = |path: serde_ignored::Path| ignored.push(path.to_string());
    let de = &mut serde_json::Deserializer::from_str(&json);
    let value =
        serde_ignored::deserialize(de, &mut track).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert!(
        ignored.is_empty(),
        "{name}: fields not modelled: {ignored:?}"
    );
    value
}

#[test]
fn responses_match_the_specification() {
    parse::<ServerTime>("get_server_time.json");
    parse::<ExchangeInfo>("get_exchange_info.json");
    parse::<OrderBook>("get_order_book.json");
    parse::<SymbolOrderBookTicker>("symbol_order_book_ticker.json");
    parse::<MarkPrice>("mark_price.json");
    parse::<NewOrderResponse>("new_order.json");
    parse::<Order>("query_order.json");
    parse::<Order>("cancel_order.json");
    parse::<Vec<Order>>("get_open_orders.json");
    parse::<Vec<Order>>("get_all_orders.json");
    parse::<ActionResult>("change_margin_type.json");
    parse::<ActionResult>("change_position_mode.json");
    parse::<PositionMode>("get_current_position_mode.json");
    parse::<Vec<AccountBalance>>("futures_account_balance.json");
    parse::<ListenKey>("create_listen_key.json");
}

#[test]
fn v3_account_and_position_risk_parse() {
    // The v3 endpoints dropped leverage / isolated / entryPrice / maxNotional
    // from positions; the v2-shaped models failed on every response.
    let account: AccountInformation = parse("account_information.json");
    assert!(!account.positions.is_empty());
    let positions: Vec<Position> = parse("position_information.json");
    assert!(!positions.is_empty());
}
