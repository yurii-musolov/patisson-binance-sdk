//! Response examples from the official Binance Spot API documentation
//! (`binance/binance-spot-api-docs`, commit 828ca74, 2026-09-18), parsed with
//! the SDK types. Each example must deserialize without error and without
//! silently ignoring any field, so a model that drifts from the docs fails
//! here instead of losing data at runtime.
//!
//! The fixtures in `tests/fixtures/spot/` are the documentation examples with
//! comments stripped.

use binance::spot::http::{TickerPriceChangeStatistic, TickerTradingDay};
use serde::de::DeserializeOwned;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/spot/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Deserialize `name` as `T`, failing on errors and on ignored fields.
fn parse<T: DeserializeOwned>(name: &str) -> T {
    let json = fixture(name);
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
fn ticker_24hr_full_is_not_mistaken_for_mini() {
    assert!(matches!(
        parse::<TickerPriceChangeStatistic>("ticker_24hr_0.json"),
        TickerPriceChangeStatistic::FullElement(_)
    ));
    assert!(matches!(
        parse::<TickerPriceChangeStatistic>("ticker_24hr_1.json"),
        TickerPriceChangeStatistic::FullList(_)
    ));
    assert!(matches!(
        parse::<TickerPriceChangeStatistic>("ticker_24hr_2.json"),
        TickerPriceChangeStatistic::MiniElement(_)
    ));
    assert!(matches!(
        parse::<TickerPriceChangeStatistic>("ticker_24hr_3.json"),
        TickerPriceChangeStatistic::MiniList(_)
    ));
}

#[test]
fn ticker_trading_day_full_is_not_mistaken_for_mini() {
    assert!(matches!(
        parse::<TickerTradingDay>("ticker_trading_day_0.json"),
        TickerTradingDay::FullElement(_)
    ));
    assert!(matches!(
        parse::<TickerTradingDay>("ticker_trading_day_1.json"),
        TickerTradingDay::FullList(_)
    ));
    assert!(matches!(
        parse::<TickerTradingDay>("ticker_trading_day_2.json"),
        TickerTradingDay::MiniElement(_)
    ));
    assert!(matches!(
        parse::<TickerTradingDay>("ticker_trading_day_3.json"),
        TickerTradingDay::MiniList(_)
    ));
}
