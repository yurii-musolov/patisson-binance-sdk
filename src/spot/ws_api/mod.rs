//! Spot WebSocket API (`wss://ws-api.binance.com/ws-api/v3`), currently the
//! user data stream.
//!
//! The REST `listenKey` endpoints were retired on 2026-02-20; account events
//! (`executionReport`, `outboundAccountPosition`, `balanceUpdate`, ...) now
//! arrive on a WebSocket API connection after a
//! [`Request::user_data_stream_subscribe_signature`] request. Run it on the
//! shared [`crate::ws::Stream`] driver:
//!
//! ```no_run
//! # async fn run() -> anyhow::Result<()> {
//! use binance::{
//!     SensitiveString, TimeOffset,
//!     spot::{BASE_URL_WEBSOCKET_API3, Path, ws_api::{IncomingMessage, Request}},
//!     ws::{Config, Event, Stream},
//! };
//!
//! let api_key = SensitiveString::from("key");
//! let api_secret = SensitiveString::from("secret");
//! let clock = TimeOffset::new();
//! let url = format!("{BASE_URL_WEBSOCKET_API3}{}", Path::WebSocketApiV3);
//! let (handle, mut events) = Stream::<Request, IncomingMessage>::new(Config::new(url));
//! handle.connect().await?;
//! while let Some(event) = events.recv().await {
//!     match event {
//!         // Subscriptions don't survive a reconnect, and the signed request
//!         // carries a timestamp: sign a fresh one on every connect.
//!         Event::Connected => {
//!             let request = Request::user_data_stream_subscribe_signature(
//!                 "subscribe", &api_key, &api_secret, clock.now(), None,
//!             );
//!             handle.send_command(request).await?;
//!         }
//!         Event::Message(IncomingMessage::Event(event)) => println!("{:?}", event.event),
//!         Event::Disconnected { .. } => break,
//!         _ => {}
//!     }
//! }
//! # Ok(()) }
//! ```

mod events;
mod incoming;
mod request;

pub use events::*;
pub use incoming::*;
pub use request::*;
