//! Spot user data stream over the WebSocket API:
//!
//! 1. Connect to `wss://ws-api.binance.com:9443/ws-api/v3`
//! 2. Register `user_data_stream_on_connect`: the driver sends a freshly
//!    signed `userDataStream.subscribe.signature` on every (re)connect
//! 3. Log account events for ~60 s, then unsubscribe and disconnect
//!
//! Run with
//!
//! ```not_rust
//! BINANCE_ENV=demo cargo run --example spot-user-data-stream
//! ```
//!
//! Needs `API_KEY` / `API_SECRET` (see examples/README.md). They are never
//! logged: `Request`'s `Debug` redacts `apiKey` and `signature`.

mod support;

use std::time::Duration;

use binance::{
    TimeOffset,
    spot::{
        Path,
        ws_api::{IncomingMessage, Request, user_data_stream_on_connect},
    },
    ws::{Config, Event, Stream},
};
use tokio::time::sleep;
use tracing::{Level, info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let ws_api = support::ws_api(support::Product::Spot)?;
    let (api_key, api_secret) = support::credentials()?;

    let url = format!("{ws_api}{}", Path::WebSocketApiV3);
    let (handle, mut events) = Stream::<Request, IncomingMessage>::new(Config::new(url));
    handle
        .on_connect(user_data_stream_on_connect(
            api_key,
            api_secret,
            TimeOffset::new(),
        ))
        .await?;
    handle.connect().await?;

    let stopper = handle.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(60)).await;
        let _ = stopper
            .send_command(Request::user_data_stream_unsubscribe("unsubscribe", None))
            .await;
        sleep(Duration::from_secs(1)).await;
        let _ = stopper.disconnect().await;
    });

    while let Some(event) = events.recv().await {
        match event {
            Event::Message(IncomingMessage::Response(response)) => match &response.error {
                None => info!(
                    id = ?response.id,
                    subscription_id = ?response.subscription_id(),
                    "request accepted"
                ),
                Some(error) => {
                    warn!(id = ?response.id, code = %error.code, msg = %error.msg, "request failed")
                }
            },
            Event::Message(IncomingMessage::Event(message)) => {
                info!(subscription = ?message.subscription_id, event = ?message.event, "account event");
            }
            Event::Disconnected { reason } => {
                info!(?reason, "disconnected");
                break;
            }
            other => info!(?other, "stream event"),
        }
    }
    Ok(())
}
