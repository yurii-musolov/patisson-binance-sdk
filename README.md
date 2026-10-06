# Binance SDK

[![Crates.io](https://img.shields.io/crates/v/patisson-binance-sdk.svg)](https://crates.io/crates/patisson-binance-sdk)
[![Documentation](https://docs.rs/patisson-binance-sdk/badge.svg)](https://docs.rs/patisson-binance-sdk)
[![MIT licensed](https://img.shields.io/crates/l/patisson-binance-sdk.svg)](LICENSE)
[![CI](https://github.com/yurii-musolov/patisson-binance-sdk/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/yurii-musolov/patisson-binance-sdk/actions/workflows/ci.yml)

Unofficial Rust SDK for the [Binance exchange API](https://developers.binance.com/en).

## Disclaimer

### Stability & Versioning Policy

No stability or backward-compatibility guarantees are provided.

Every version change must be treated as a breaking change, including minor and patch releases.

Users are strongly advised to pin an exact version, for example:

```rs
patisson-binance-sdk = "=0.1.12"
```

### Maintenance Policy

This package is developed only in the author’s free time.

Releases are best-effort and not planned in advance.

The scope of the package is intentionally limited to the most commonly used functionality.

## Features

- **Spot Trading** — public market data + signed trading via `PublicClient` / `PrivateClient`
- **Margin Trading** — cross-margin and isolated-margin trading, borrow/repay, user data stream
- **Derivatives** — USDⓈ-M and COIN-M Futures
- **Wallet** — coin/network metadata, deposits, withdrawals, account status
- **Shared WebSocket driver** (`binance::ws::Stream<C, M>`) with protocol-level heartbeat,
  automatic reconnect with exponential back-off, and a connection-TTL refresh that
  pre-empts Binance's 24-hour disconnect
- **Typed error classification** — `ErrorCode` is a transparent newtype with named
  constants and predicates (`is_auth`, `is_rate_limited`, `is_transient`, …) instead of
  a closed enum that goes stale every time Binance ships a new code
- **Builder pattern** on every `*Request` / `*Params` type with private fields, so new
  optional parameters can be added without breaking callers

> **Scope:** every product module ships a representative slice of endpoints — enough
> to use the SDK and copy the pattern for the rest of Binance's API. Coverage is not
> exhaustive; extensions are straightforward to add following the existing layout.

## Installation

```toml
[dependencies]
patisson-binance-sdk = "=0.1.12"
```

> The package is `patisson-binance-sdk` (Cargo.toml); the library you import is `binance`:
>
> ```rust
> use binance::spot::http::PublicClient;
> ```

### TLS

REST and WebSocket share one TLS implementation, chosen by feature:

| Feature | TLS | Notes |
|---|---|---|
| `rustls-tls` (default) | rustls (aws-lc-rs) | no system OpenSSL needed |
| `native-tls` | platform library (OpenSSL on Linux) | `default-features = false, features = ["native-tls"]` |

## Quick start

### Environments

`binance::Environment` (`Production`, `Testnet`, `Demo`, `Custom`) knows the
base URLs of every product, so configs can be built without picking
constants by hand:

```rust
use binance::{Environment, Product, spot::http::{PublicClient, PublicConfig}};

let client = PublicClient::new(PublicConfig::for_env(Environment::Demo)?)?;
let stream = Environment::Demo.stream_url(Product::Spot)?; // wss://demo-stream.binance.com:9443
```

Products an environment doesn't offer (margin and wallet are production
only) return an `Unsupported` error instead of silently using production.

### Public REST — no API key

```rust
use binance::spot::{
    BASE_URL_API,
    http::{GetExchangeInfoParams, PublicClient, PublicConfig},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = PublicClient::new(PublicConfig::new(BASE_URL_API))?;

    let time = client.get_server_time().await?;
    println!("server time: {}", time.result.server_time);

    let info = client
        .get_exchange_info(GetExchangeInfoParams::new().symbol("BTCUSDT"))
        .await?;
    println!("symbols: {}", info.result.symbols.len());

    Ok(())
}
```

### Private REST — signed

`PrivateClient` requires API key + secret. The timestamp and HMAC-SHA256
signature are appended automatically.

```rust
use binance::{
    SensitiveString,
    spot::{
        BASE_URL_API,
        http::{GetAccountInformationParams, PrivateClient, PrivateConfig},
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = SensitiveString::from(std::env::var("API_KEY")?);
    let api_secret = SensitiveString::from(std::env::var("API_SECRET")?);
    let client = PrivateClient::new(PrivateConfig::new(BASE_URL_API, api_key, api_secret))?;

    let account = client
        .account_information(GetAccountInformationParams::new())
        .await?;
    println!("balances: {}", account.result.balances.len());
    Ok(())
}
```

### Trading rules

Round prices and quantities to what a symbol accepts and check an order
before sending it, instead of getting `-1013 Filter failure`:

```rust
use binance::{ExchangeRules, Rounding};

let info = client.exchange_info(/* ... */).await?.result;
let rules = ExchangeRules::from(&info);
let btc = rules.get("BTCUSDT").unwrap();
let price = btc.round_price(price, Rounding::Down);
let qty = btc.round_qty(qty, Rounding::Down);
btc.validate(price, qty)?; // tick, step, min/max qty and price, notional
```

Works with spot, USD-M and COIN-M `exchangeInfo`.

### Clock drift

Signed requests fail with `-1021` when the local clock drifts from
Binance's. Correct it once and let the client resync on its own:

```rust
let cfg = PrivateConfig::for_env(Environment::Production, api_key, api_secret)?
    .resync_on_invalid_timestamp(true); // on -1021: resync and resend once
let client = PrivateClient::new(cfg)?;
client.sync_time().await?; // offset shared by every clone of the client
```

### WebSocket — market data

The shared `Stream<C, M>` driver is generic over the outgoing command and
incoming message types — every product module plugs into it with its own
`OutgoingMessage` / `IncomingMessage`.

```rust
use binance::{
    spot::{
        BASE_URL_MARKET_DATA_STREAM1, Path,
        ws::{IncomingMessage, OutgoingMessage, StreamName},
    },
    ws::{Config, Event, Stream},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = format!("{BASE_URL_MARKET_DATA_STREAM1}{}", Path::Stream);
    let (handle, mut events) = Stream::<OutgoingMessage, IncomingMessage>::new(Config::new(url));

    handle.connect().await?;

    while let Some(event) = events.recv().await {
        match event {
            // Subscriptions don't survive a reconnect: (re)subscribe on every connect.
            Event::Connected => {
                handle
                    .send_command(OutgoingMessage::Subscribe {
                        id: Some("req-1".into()),
                        params: vec![StreamName::Trade { symbol: "btcusdt".into() }],
                    })
                    .await?;
            }
            Event::Message(msg) => println!("{msg:?}"),
            Event::Disconnected { .. } => break,
            other => println!("{other:?}"),
        }
    }
    Ok(())
}
```

Use `Config::futures(url)` for USD-M / COIN-M streams: their servers ping only
every 3 minutes, so the Spot heartbeat defaults of `Config::new` would declare
the connection dead.

### Error handling

API errors round-trip through `ErrorCode`, a transparent newtype around `i64`
with named constants and classification predicates — match by intent rather
than by remembering numbers.

```rust
use binance::{ErrorCode, spot::Error};

match client.new_order(req).await {
    Ok(resp) => { /* … */ }
    Err(Error::Api(e)) if e.code.is_rate_limited() => {
        // Binance is throttling — back off and retry.
    }
    Err(Error::Api(e)) if e.code.is_auth() => {
        // API key / signature / timestamp problem; not transient.
        eprintln!("auth error {}: {}", e.code, e.msg);
    }
    Err(Error::Api(e)) if e.code == ErrorCode::NO_SUCH_ORDER => {
        // Order already filled / canceled / never placed.
    }
    Err(e) if e.is_execution_status_unknown() => {
        // -1006/-1007, a 5xx without an error body, or a timeout after the
        // request was sent: the order may exist. Query it before retrying.
    }
    Err(e) => return Err(e.into()),
}
```

Available predicates: `is_auth`, `is_invalid_timestamp`, `is_invalid_signature`,
`is_bad_api_key_format`, `is_api_key_rejected`, `is_wrong_permissions`,
`is_bad_request`, `is_rate_limited`, `is_server_error`, `is_transient`,
`is_order_rejected`, `is_no_such_order`. Plus `ErrorCode::raw() -> i64` as the
escape hatch for product-specific codes.

A non-2xx response whose body is not a Binance error (e.g. an HTML page from
a proxy on 502/503) surfaces as `Error::Http { status, body }`, so the HTTP
status is never lost.

## Modules

| Path | Purpose | HTTP | WS |
|---|---|---|---|
| `binance::spot` | Spot trading | `PublicClient` + `PrivateClient` | market streams; user data stream via the WebSocket API (`spot::ws_api`) |
| `binance::margin` | Margin (cross + isolated) | `PrivateClient` only — use spot's `PublicClient` for market data | user data stream (listenKey-based) |
| `binance::derivatives::usds_margined_futures` | USDⓈ-M Futures | `PublicClient` + `PrivateClient` | market streams |
| `binance::derivatives::coin_margined_futures` | COIN-M Futures | `PublicClient` + `PrivateClient` | market streams |
| `binance::wallet` | Deposits / withdrawals / account status | `PrivateClient` only | — |
| `binance::ws` | Shared WebSocket driver (`Stream<C, M>`) | — | — |
| `binance::ErrorCode` | Crate-wide error code newtype + predicates | — | — |

## Examples

Runnable examples live in [`examples/`](examples/), grouped by product prefix:

| Prefix | Products |
|---|---|
| `spot-*` | Spot |
| `usdm-*` | USDⓈ-M Futures |
| `coinm-*` | COIN-M Futures |
| `margin-*` | Margin |
| `wallet-*` | Wallet |

Run any of them with cargo:

```sh
cargo run --example spot-server-time
cargo run --example spot-stream-public
cargo run --example margin-user-data-stream
```

Examples that hit private endpoints expect `API_KEY` and `API_SECRET` in the
environment. See [`examples/README.md`](examples/README.md) for the full list.

## Development

Quality-check hooks live in [`.githooks/`](.githooks/) — install once per clone:

```sh
./.githooks/install.sh
```

This wires `core.hooksPath` to `.githooks/`, which adds:

| Hook | Runs |
|---|---|
| `pre-commit` | `cargo fmt --check`, `cargo check --all-targets` |
| `pre-push` | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` |

The pre-push hook is strict — any new clippy warning fails the push. Skip a
single run with `--no-verify` when you have a good reason (WIP, hotfix).

## License

This project is licensed under the [MIT license](LICENSE).
