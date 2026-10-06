//! `SymbolRules` / `ExchangeRules` built from live `exchangeInfo` responses
//! (spot BTCUSDT 2026-10-06, USD-M and COIN-M 2026-10-04).

use binance::{
    ExchangeRules, Rounding, RuleViolation,
    derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm},
    spot,
};
use rust_decimal::dec;
use serde::de::DeserializeOwned;

fn fixture<T: DeserializeOwned>(path: &str) -> T {
    let path = format!("{}/tests/fixtures/{path}", env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn spot_btcusdt() {
    let info: spot::http::ExchangeInfo = fixture("spot/exchange_info_live.json");
    let rules = ExchangeRules::from(&info);
    let r = rules.get("BTCUSDT").expect("BTCUSDT");
    assert_eq!(r.tick_size, Some(dec!(0.01)));
    assert_eq!(r.step_size, Some(dec!(0.00001)));
    assert_eq!(r.min_notional, Some(dec!(5)));
    assert_eq!(r.max_notional, Some(dec!(9000000)));
    assert!(r.min_notional_applies_to_market && !r.max_notional_applies_to_market);
    // MARKET_LOT_SIZE stepSize / minQty are 0 (disabled): LOT_SIZE applies.
    assert_eq!(r.market_step_size, None);
    assert_eq!(
        r.round_market_qty(dec!(0.0001234), Rounding::Down),
        dec!(0.00012)
    );

    let price = r.round_price(dec!(65000.019), Rounding::Down);
    let qty = r.round_qty(dec!(0.0001999), Rounding::Down);
    assert_eq!((price, qty), (dec!(65000.01), dec!(0.00019)));
    assert_eq!(r.validate(price, qty), Ok(()));
    assert!(matches!(
        r.validate(price, dec!(0.00001)),
        Err(RuleViolation::NotionalBelowMin { .. })
    ));
}

#[test]
fn usdm_btcusdt() {
    let info: usdm::http::ExchangeInfo = fixture("usdm/exchange_info_live.json");
    let rules = ExchangeRules::from(&info);
    assert_eq!(rules.len(), info.symbols.len());
    let r = rules.get("BTCUSDT").expect("BTCUSDT");
    assert_eq!(r.tick_size, Some(dec!(0.10)));
    assert_eq!(r.market_max_qty, Some(dec!(120)));
    assert!(r.min_notional.is_some() && r.min_notional_applies_to_market);
    assert_eq!(r.validate(dec!(65000.1), dec!(0.002)), Ok(()));
    assert!(matches!(
        r.validate(dec!(65000.15), dec!(0.002)),
        Err(RuleViolation::PriceNotOnTick { .. })
    ));
    assert!(matches!(
        r.validate_market(dec!(121), None),
        Err(RuleViolation::QtyAboveMax { .. })
    ));
}

#[test]
fn coinm_btcusd_perp() {
    let info: coinm::http::ExchangeInfo = fixture("coinm/exchange_info_live.json");
    let rules = ExchangeRules::from(&info);
    let r = rules.get("BTCUSD_PERP").expect("BTCUSD_PERP");
    assert_eq!(r.step_size, Some(dec!(1)));
    assert_eq!(r.min_notional, None, "COIN-M quantities are contracts");
    assert_eq!(r.round_qty(dec!(2.7), Rounding::Nearest), dec!(3));
    assert_eq!(r.validate(dec!(65000.1), dec!(3)), Ok(()));
    assert!(matches!(
        r.validate(dec!(65000.1), dec!(1.5)),
        Err(RuleViolation::QtyNotOnStep { .. })
    ));
}
