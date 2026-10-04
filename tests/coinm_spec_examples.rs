//! Response examples from Binance's COIN-M Futures API specification, as
//! shipped in the official `binance-connector-rust` (commit 5ff71b4,
//! 2026-10-01), parsed with the SDK types. Each example must deserialize
//! without error and without silently ignoring any field.
//!
//! Left out: the listenKey keepalive example echoes the key, which the SDK
//! deliberately discards.

use binance::derivatives::coin_margined_futures::http::{
    AccountInformation, ActionResult, Balance, CancelOrderResponse, ChangeInitialLeverageResponse,
    CurrentPositionMode, ExchangeInfo, ListenKey, NewOrderResponse, Order, OrderBook,
    PositionInformation, PremiumIndex, ServerTime, SymbolOrderBookTicker, SymbolPriceTicker, Trade,
};
use serde::de::DeserializeOwned;

fn parse<T: DeserializeOwned>(name: &str) -> T {
    let path = format!("{}/tests/fixtures/coinm/{name}", env!("CARGO_MANIFEST_DIR"));
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
    parse::<Vec<SymbolPriceTicker>>("symbol_price_ticker.json");
    parse::<Vec<SymbolOrderBookTicker>>("symbol_order_book_ticker.json");
    parse::<Vec<PremiumIndex>>("get_premium_index.json");
    parse::<NewOrderResponse>("new_order.json");
    parse::<Order>("query_order.json");
    parse::<CancelOrderResponse>("cancel_order.json");
    parse::<Vec<Order>>("get_open_orders.json");
    parse::<Vec<Order>>("get_all_orders.json");
    parse::<Vec<Trade>>("account_trade_list.json");
    parse::<Vec<PositionInformation>>("position_information.json");
    parse::<ChangeInitialLeverageResponse>("change_initial_leverage.json");
    parse::<ActionResult>("change_margin_type.json");
    parse::<ActionResult>("change_position_mode.json");
    parse::<CurrentPositionMode>("get_current_position_mode.json");
    parse::<AccountInformation>("account_information.json");
    parse::<Vec<Balance>>("futures_account_balance.json");
    parse::<ListenKey>("create_listen_key.json");
}

#[test]
fn tickers_are_arrays_even_for_one_symbol() {
    // COIN-M returns every contract of the pair, so the response is always a
    // list; the single-object model failed on every call.
    let prices: Vec<SymbolPriceTicker> = parse("symbol_price_ticker.json");
    assert!(!prices.is_empty());
    let books: Vec<SymbolOrderBookTicker> = parse("symbol_order_book_ticker.json");
    assert!(!books.is_empty());
}
