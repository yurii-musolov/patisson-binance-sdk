//! Response examples from Binance's Margin Trading API specification, as
//! shipped in the official `binance-connector-rust` (commit 5ff71b4,
//! 2026-10-01), parsed with the SDK types. Each example must deserialize
//! without error and without silently ignoring any field.
//!
//! Two specification examples are left out because they are wrong, not the
//! SDK: `DELETE /sapi/v1/margin/order` shows `"orderId": "28"` as a string
//! (the API returns a number) and `GET /sapi/v1/margin/priceIndex` shows the
//! collateral-ratio payload of another endpoint.

use binance::margin::http::{
    BorrowRepayRecords, BorrowRepayResult, CanceledOrderOrList, InterestRateRecord,
    IsolatedMarginAccount, MarginAccount, MarginAsset, MaxBorrowable, MaxTransferable,
    NewOrderResponse, Order, Trade,
};
use serde::de::DeserializeOwned;

fn parse<T: DeserializeOwned>(name: &str) -> T {
    let path = format!(
        "{}/tests/fixtures/margin/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
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
    parse::<Vec<MarginAsset>>("get_all_assets.json");
    parse::<MarginAccount>("margin_account.json");
    parse::<NewOrderResponse>("new_order.json");
    parse::<Order>("query_order.json");
    parse::<Vec<CanceledOrderOrList>>("cancel_all_open_orders.json");
    parse::<Vec<Order>>("get_open_orders.json");
    parse::<Vec<Order>>("get_all_orders.json");
    parse::<Vec<Trade>>("get_account_trade_list.json");
    parse::<MaxBorrowable>("max_borrowable.json");
    parse::<BorrowRepayRecords>("get_borrow_repay_records.json");
    parse::<Vec<InterestRateRecord>>("get_margin_interest_rate_history.json");
    parse::<MaxTransferable>("get_max_transfer_out_amount.json");
    parse::<IsolatedMarginAccount>("get_isolated_margin_account.json");
}

#[test]
fn borrow_repay_reads_the_camel_case_tran_id() {
    let result: BorrowRepayResult = parse("borrow_repay.json");
    assert_eq!(result.tran_id, 100000001);
}

#[test]
fn margin_account_reads_the_upper_case_collateral_key() {
    let account: MarginAccount = parse("margin_account.json");
    assert!(account.margin_level_status.is_none());
    // Binance spells it `TotalCollateralValueInUSDT`; the documented camel
    // case spelling is accepted as an alias.
    let json = std::fs::read_to_string(format!(
        "{}/tests/fixtures/margin/margin_account.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("TotalCollateralValueInUSDT", "totalCollateralValueInUsdt");
    let alias: MarginAccount = serde_json::from_str(&json).unwrap();
    assert_eq!(
        alias.total_collateral_value_in_usdt,
        account.total_collateral_value_in_usdt
    );
}
