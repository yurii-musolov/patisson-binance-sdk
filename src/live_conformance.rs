//! Live conformance: public production endpoints and streams parsed with
//! the SDK models, strictly (a field the SDK doesn't model fails the test).
//!
//! Off by default; needs network access, no API keys:
//!
//! ```text
//! cargo test --features live --lib live_conformance -- --nocapture
//! ```
//!
//! Runs weekly in CI (`.github/workflows/live-conformance.yml`), so a
//! response shape Binance changes is noticed before users report it.
//!
//! Spot goes through Binance's public market data mirrors
//! (`data-api.binance.vision`, `data-stream.binance.vision`), reachable from
//! anywhere. The futures hosts answer `451` from restricted locations such
//! as the US-based GitHub runners: those checks print `SKIP` and stop.
//! Hosts can be overridden with `BINANCE_LIVE_{SPOT,USDM,COINM}{,_WS}`.

use std::time::Duration;

use futures_util::StreamExt;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message},
};

use crate::{
    derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm},
    spot,
};

fn host(var: &str, default: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| default.to_owned())
}

fn restricted(status: u16) -> bool {
    status == 451 || status == 403
}

/// Body of `url`; `None` (after printing `SKIP`) when the host refuses this
/// location.
async fn get(url: &str) -> Option<String> {
    let response = reqwest::Client::new()
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .unwrap_or_else(|e| panic!("{url}: {e}"));
    let status = response.status().as_u16();
    if restricted(status) {
        eprintln!("SKIP {url}: HTTP {status}, restricted location");
        return None;
    }
    assert!(response.status().is_success(), "{url}: HTTP {status}");
    Some(response.text().await.unwrap())
}

/// Parse `json` as `T`, failing on fields `T` doesn't model (except
/// `allowed`, e.g. an enum tag).
fn strict<T: DeserializeOwned>(source: &str, json: &str, allowed: &[&str]) {
    let mut ignored = Vec::new();
    let mut track = |path: serde_ignored::Path| ignored.push(path.to_string());
    let de = &mut serde_json::Deserializer::from_str(json);
    let _: T =
        serde_ignored::deserialize(de, &mut track).unwrap_or_else(|e| panic!("{source}: {e}"));
    ignored.retain(|p| !allowed.contains(&p.as_str()));
    assert!(
        ignored.is_empty(),
        "{source}: fields not modelled: {ignored:?}"
    );
}

/// GET `url` and parse it strictly as `T`. `false` when skipped.
async fn rest<T: DeserializeOwned>(source: &str, url: &str) -> bool {
    match get(url).await {
        Some(json) => {
            strict::<T>(source, &json, &[]);
            true
        }
        None => false,
    }
}

/// `exchangeInfo`: strict parse, plus a round trip of every filter (tagged
/// enums `serde_ignored` can't look into). `false` when skipped.
async fn exchange_info<T: DeserializeOwned, F: DeserializeOwned + Serialize>(
    source: &str,
    url: &str,
) -> bool {
    let Some(json) = get(url).await else {
        return false;
    };
    strict::<T>(source, &json, &[]);
    let v: Value = serde_json::from_str(&json).unwrap();
    for symbol in v["symbols"].as_array().unwrap() {
        for filter in symbol["filters"].as_array().unwrap() {
            let parsed: F = serde_json::from_value(filter.clone())
                .unwrap_or_else(|e| panic!("{source}: {e}: {filter}"));
            let back = serde_json::to_value(&parsed).unwrap();
            for key in filter.as_object().unwrap().keys() {
                assert!(back.get(key).is_some(), "{source}: {filter} loses {key}");
            }
        }
    }
    true
}

/// First `n` frames of a stream, parsed strictly as the event struct `T`
/// (the `e` tag is allowed). `false` when skipped.
async fn stream<T: DeserializeOwned>(source: &str, url: &str, n: usize) -> bool {
    let connected = tokio::time::timeout(Duration::from_secs(20), connect_async(url))
        .await
        .unwrap_or_else(|_| panic!("{url}: connect timed out"));
    let mut ws = match connected {
        Ok((ws, _)) => ws,
        Err(WsError::Http(response)) if restricted(response.status().as_u16()) => {
            eprintln!(
                "SKIP {url}: HTTP {}, restricted location",
                response.status()
            );
            return false;
        }
        Err(e) => panic!("{url}: {e}"),
    };
    let mut seen = 0;
    while seen < n {
        let msg = tokio::time::timeout(Duration::from_secs(60), ws.next())
            .await
            .unwrap_or_else(|_| panic!("{url}: no message in 60 s"))
            .unwrap_or_else(|| panic!("{url}: closed"))
            .unwrap_or_else(|e| panic!("{url}: {e}"));
        if let Message::Text(text) = msg {
            strict::<T>(source, &text, &["e"]);
            seen += 1;
        }
    }
    true
}

#[tokio::test]
async fn spot_rest() {
    let api = host("BINANCE_LIVE_SPOT", "https://data-api.binance.vision");
    let url = format!("{api}/api/v3/exchangeInfo?symbol=BTCUSDT");
    if !exchange_info::<spot::http::ExchangeInfo, spot::http::Filter>("spot exchangeInfo", &url)
        .await
    {
        return;
    }
    rest::<spot::http::OrderBook>(
        "spot depth",
        &format!("{api}/api/v3/depth?symbol=BTCUSDT&limit=5"),
    )
    .await;
    rest::<Vec<spot::http::Kline>>(
        "spot klines",
        &format!("{api}/api/v3/klines?symbol=BTCUSDT&interval=1m&limit=3"),
    )
    .await;
    rest::<spot::http::TickerPriceChangeStatisticFull>(
        "spot ticker/24hr",
        &format!("{api}/api/v3/ticker/24hr?symbol=BTCUSDT"),
    )
    .await;
    rest::<spot::http::SymbolOrderBookTicker>(
        "spot ticker/bookTicker",
        &format!("{api}/api/v3/ticker/bookTicker?symbol=BTCUSDT"),
    )
    .await;
    rest::<spot::http::CurrentAveragePrice>(
        "spot avgPrice",
        &format!("{api}/api/v3/avgPrice?symbol=BTCUSDT"),
    )
    .await;
}

#[tokio::test]
async fn spot_streams() {
    let ws = host("BINANCE_LIVE_SPOT_WS", "wss://data-stream.binance.vision");
    if !stream::<spot::ws::TradeMsg>("spot trade", &format!("{ws}/ws/btcusdt@trade"), 2).await {
        return;
    }
    stream::<spot::ws::KlineMsg>("spot kline", &format!("{ws}/ws/btcusdt@kline_1s"), 1).await;
}

#[tokio::test]
async fn usdm_rest() {
    let api = host("BINANCE_LIVE_USDM", "https://fapi.binance.com");
    let url = format!("{api}/fapi/v1/exchangeInfo");
    if !exchange_info::<usdm::http::ExchangeInfo, usdm::http::SymbolFilter>(
        "usdm exchangeInfo",
        &url,
    )
    .await
    {
        return;
    }
    rest::<usdm::http::OrderBook>(
        "usdm depth",
        &format!("{api}/fapi/v1/depth?symbol=BTCUSDT&limit=5"),
    )
    .await;
    rest::<Vec<usdm::http::Kline>>(
        "usdm klines",
        &format!("{api}/fapi/v1/klines?symbol=BTCUSDT&interval=1m&limit=3"),
    )
    .await;
    rest::<usdm::http::MarkPrice>(
        "usdm premiumIndex",
        &format!("{api}/fapi/v1/premiumIndex?symbol=BTCUSDT"),
    )
    .await;
}

#[tokio::test]
async fn usdm_streams() {
    let ws = host("BINANCE_LIVE_USDM_WS", "wss://fstream.binance.com");
    let mark_price = format!("{ws}/market/ws/btcusdt@markPrice@1s");
    if !stream::<usdm::ws::MarkPriceUpdateMsg>("usdm markPrice", &mark_price, 1).await {
        return;
    }
    let agg_trade = format!("{ws}/market/ws/btcusdt@aggTrade");
    stream::<usdm::ws::AggTradeMsg>("usdm aggTrade", &agg_trade, 2).await;
    let kline = format!("{ws}/market/ws/btcusdt@kline_1m");
    stream::<usdm::ws::KlineMsg>("usdm kline", &kline, 1).await;
    let depth = format!("{ws}/public/ws/btcusdt@depth@100ms");
    stream::<usdm::ws::DepthUpdateMsg>("usdm depthUpdate", &depth, 1).await;
}

#[tokio::test]
async fn coinm_rest() {
    let api = host("BINANCE_LIVE_COINM", "https://dapi.binance.com");
    let url = format!("{api}/dapi/v1/exchangeInfo");
    if !exchange_info::<coinm::http::ExchangeInfo, coinm::http::SymbolFilter>(
        "coinm exchangeInfo",
        &url,
    )
    .await
    {
        return;
    }
    rest::<coinm::http::OrderBook>(
        "coinm depth",
        &format!("{api}/dapi/v1/depth?symbol=BTCUSD_PERP&limit=5"),
    )
    .await;
    rest::<Vec<coinm::http::Kline>>(
        "coinm klines",
        &format!("{api}/dapi/v1/klines?symbol=BTCUSD_PERP&interval=1m&limit=3"),
    )
    .await;
}

#[tokio::test]
async fn coinm_streams() {
    let ws = host("BINANCE_LIVE_COINM_WS", "wss://dstream.binance.com");
    let mark_price = format!("{ws}/ws/btcusd_perp@markPrice@1s");
    if !stream::<coinm::ws::MarkPriceUpdateMsg>("coinm markPrice", &mark_price, 1).await {
        return;
    }
    let agg_trade = format!("{ws}/ws/btcusd_perp@aggTrade");
    stream::<coinm::ws::AggTradeMsg>("coinm aggTrade", &agg_trade, 2).await;
    let kline = format!("{ws}/ws/btcusd_perp@kline_1m");
    stream::<coinm::ws::KlineMsg>("coinm kline", &kline, 1).await;
    let depth = format!("{ws}/ws/btcusd_perp@depth@100ms");
    stream::<coinm::ws::DepthUpdateMsg>("coinm depthUpdate", &depth, 1).await;
}
