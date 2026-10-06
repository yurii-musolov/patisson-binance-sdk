//! Binance API SDK: REST clients, WebSocket streams, and the building blocks
//! every trading program ends up writing (account and order state, symbol
//! rules, clock sync, listenKey keepalive, pagination).
//!
//! Pick what you need; nothing runs in the background unless you start it.
//!
//! | Module | Product |
//! |---|---|
//! | [`spot`] | Spot: REST, market streams, user data over the WebSocket API |
//! | [`margin`] | Cross and isolated margin |
//! | [`derivatives::usds_margined_futures`] | USD-M Futures |
//! | [`derivatives::coin_margined_futures`] | COIN-M Futures |
//! | [`wallet`] | Deposits, withdrawals, account status |
//! | [`ws`] | WebSocket driver shared by every product |
//!
//! # Environments
//!
//! [`Environment`] gives the base URLs of production, testnet and demo mode
//! for every [`Product`]; product configs take it through `for_env`.
//!
//! # REST
//!
//! ```no_run
//! use binance::{
//!     Environment,
//!     spot::{KlineInterval, http::{GetKlineListParams, PublicClient, PublicConfig}},
//! };
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = PublicClient::new(PublicConfig::for_env(Environment::Production)?)?;
//! let params = GetKlineListParams::new("BTCUSDT", KlineInterval::Minute1).limit(10);
//! for kline in client.get_kline_list(params).await?.result {
//!     println!("{kline:?}");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Signed endpoints use `PrivateClient` with `PrivateConfig::for_env(env,
//! api_key, api_secret)`; keys are [`SensitiveString`]s and never appear in
//! `Debug` output. Clients are cheap to clone. Configs accept a proxy, a
//! shared [`RateLimiter`] and timeouts; `sync_time` and
//! `resync_on_invalid_timestamp` deal with clock drift (`-1021`).
//!
//! # Streams
//!
//! [`ws::Stream`] connects, reconnects with backoff, answers pings and
//! restores subscriptions registered with [`ws::Handle::on_connect`].
//!
//! ```no_run
//! use binance::{
//!     Environment, Product,
//!     spot::{KlineInterval, Path, ws::{IncomingMessage, OutgoingMessage, StreamName}},
//!     ws::{Config, Event, Stream},
//! };
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let url = format!("{}{}", Environment::Production.stream_url(Product::Spot)?, Path::Stream);
//! let (handle, mut events) = Stream::<OutgoingMessage, IncomingMessage>::new(Config::new(url));
//! handle
//!     .on_connect(|| {
//!         vec![OutgoingMessage::Subscribe {
//!             id: Some(1.into()),
//!             params: vec![StreamName::Kline {
//!                 symbol: "btcusdt".into(),
//!                 interval: KlineInterval::Minute1,
//!             }],
//!         }]
//!     })
//!     .await?;
//! handle.connect().await?;
//! while let Some(event) = events.recv().await {
//!     if let Event::Message(message) = event {
//!         println!("{message:?}");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # User data and state
//!
//! - Spot user data comes over the WebSocket API
//!   ([`spot::ws_api::user_data_stream_on_connect`]); margin and futures use
//!   a listenKey kept alive by [`ListenKeyKeeper`].
//! - [`spot::AccountState`], [`derivatives::usds_margined_futures::AccountState`]
//!   and [`derivatives::coin_margined_futures::AccountState`] load a REST
//!   snapshot and apply user data events, ignoring stale and repeated ones.
//! - [`Orders`] / [`OrderState`] track orders from order update events of any
//!   product.
//! - [`SymbolRules`] / [`ExchangeRules`] round prices and quantities and
//!   check orders against the symbol filters before sending.
//! - `*_all` methods walk `allOrders` and trade lists page by page
//!   ([`AllPages`]).
//!
//! # Errors
//!
//! Each product has its own `Error` with `api_code()`, `is_retryable()`,
//! `retry_after()` and `is_execution_status_unknown()`; [`ErrorCode`] names
//! Binance's codes.
//!
//! The `examples/` directory has a runnable program for each of these.

#![warn(missing_docs)]

mod common;
mod crypto;
mod environment;
mod error_code;
#[cfg(test)]
mod filter_conformance;
mod http;
mod listen_key;
#[cfg(all(test, feature = "live"))]
mod live_conformance;
mod order_state;
mod pagination;
mod rules;
mod serde;

/// USD-M and COIN-M futures.
pub mod derivatives;
/// Cross and isolated margin: REST and user data stream.
pub mod margin;
pub mod rate_limit;
/// Spot trading: REST, market streams, WebSocket API user data stream, account state.
pub mod spot;
/// Wallet: deposits, withdrawals, account status.
pub mod wallet;
/// WebSocket streams.
pub mod ws;

pub use common::*;
pub use crypto::{SensitiveString, TimeOffset, timestamp};
pub use environment::{Environment, Product, Unsupported};
pub use error_code::ErrorCode;
pub use http::{DEFAULT_HTTP_CONNECT_TIMEOUT, DEFAULT_HTTP_TIMEOUT, Timeouts};
pub use listen_key::{KEEPALIVE_INTERVAL, ListenKeyKeeper};
pub use order_state::{Fill, OrderEvent, OrderState, OrderStatus, Orders};
pub use pagination::AllPages;
pub use rate_limit::{BucketKind, BucketSpec, Cost, RateLimitSource, RateLimited, RateLimiter};
pub use rules::{ExchangeRules, Rounding, RuleViolation, SymbolRules};
