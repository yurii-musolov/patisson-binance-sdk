//! Response examples from Binance's Wallet API specification, as shipped in
//! the official `binance-connector-rust` (commit 5ff71b4, 2026-10-01), parsed
//! with the SDK types. Each example must deserialize without error and
//! without silently ignoring any field.
//!
//! Two specification examples are left out because they belong to other
//! endpoints: `GET /sapi/v1/asset/tradeFee` shows the `getUserAsset` payload
//! and `POST /sapi/v1/account/disableFastWithdrawSwitch` shows API key
//! restrictions.

use binance::wallet::http::{
    AccountApiTradingStatus, AccountStatus, AssetDividendRecord, Deposit, DepositAddress,
    GetWithdrawHistoryParams, UniversalTransferHistory, UniversalTransferResult, WalletBalance,
    Withdraw,
};
use serde::de::DeserializeOwned;

fn parse<T: DeserializeOwned>(name: &str) -> T {
    let path = format!(
        "{}/tests/fixtures/wallet/{name}",
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
    parse::<DepositAddress>("get_deposit_address.json");
    parse::<Vec<Deposit>>("get_deposit_history.json");
    parse::<Vec<Withdraw>>("get_withdraw_history.json");
    parse::<AccountStatus>("get_account_status.json");
    parse::<AssetDividendRecord>("get_asset_dividend_record.json");
    parse::<UniversalTransferResult>("user_universal_transfer.json");
    parse::<UniversalTransferHistory>("query_user_universal_transfer_history.json");
    parse::<AccountApiTradingStatus>("account_api_trading_status.json");
}

#[test]
fn wallet_balance_carries_the_optional_asset_breakdown() {
    let balances: Vec<WalletBalance> = parse("query_user_wallet_balance.json");
    let detail = balances[0].asset_balances.as_ref().expect("breakdown");
    assert_eq!(detail[0].asset, "USDT");
}

#[test]
fn withdraw_id_list_is_sent_comma_separated() {
    let params = GetWithdrawHistoryParams::new().id_list(["a1", "b2"]);
    let query = serde_urlencoded::to_string(&params).unwrap();
    assert_eq!(query, "idList=a1%2Cb2");
}
