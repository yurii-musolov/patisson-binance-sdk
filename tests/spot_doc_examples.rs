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

mod ws {
    use super::{fixture, parse};
    use binance::spot::ws::{IncomingMessage, StreamMessage};

    #[test]
    fn documented_stream_events_parse() {
        for name in [
            "ws_agg_trade.json",
            "ws_trade.json",
            "ws_kline.json",
            "ws_mini_ticker.json",
            "ws_depth_update.json",
            "ws_server_shutdown.json",
        ] {
            assert!(
                matches!(parse::<IncomingMessage>(name), IncomingMessage::Stream(_)),
                "{name}"
            );
        }
        assert!(matches!(
            parse::<IncomingMessage>("ws_combined_server_shutdown.json"),
            IncomingMessage::CombinedStream(msg) if matches!(msg.data, StreamMessage::ServerShutdown(_))
        ));
    }

    #[test]
    fn partial_book_depth_is_not_an_empty_response() {
        match parse::<IncomingMessage>("ws_partial_depth.json") {
            IncomingMessage::PartialBookDepth(depth) => {
                assert_eq!(depth.last_update_id, 160);
                assert!(!depth.bids.is_empty() && !depth.asks.is_empty());
            }
            other => panic!("unexpected {other:?}"),
        }
        assert!(matches!(
            parse::<IncomingMessage>("ws_combined_partial_depth.json"),
            IncomingMessage::CombinedPartialBookDepth(_)
        ));
    }

    #[test]
    fn unmodelled_event_is_a_parse_error_not_an_empty_response() {
        // `bookTicker` is not modelled by the SDK.
        let json = fixture("ws_book_ticker.json");
        assert!(serde_json::from_str::<IncomingMessage>(&json).is_err());
    }

    #[test]
    fn subscription_replies_still_parse_as_responses() {
        for json in [
            r#"{"result":null,"id":1}"#,
            r#"{"result":["btcusdt@aggTrade"],"id":3}"#,
            r#"{"id":"a","status":200,"result":{},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT"}]}"#,
        ] {
            let msg: IncomingMessage = serde_json::from_str(json).unwrap();
            assert!(matches!(msg, IncomingMessage::Response(_)), "{json}");
        }
    }
}
