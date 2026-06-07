//! Margin user data stream end-to-end:
//!
//! 1. `POST /sapi/v1/userDataStream` to mint a `listenKey`
//! 2. WebSocket-connect to `wss://stream.binance.com:9443/ws/<listenKey>`
//! 3. Consume events for ~60 s
//! 4. `DELETE /sapi/v1/userDataStream` to release the key
//!
//! Run with
//!
//! ```not_rust
//! cargo run --example margin-user-data-stream
//! ```
//!
//! In production also schedule `keepalive_listen_key` every 30 minutes — keys
//! expire after 60 minutes of inactivity.

use std::time::Duration;

use binance::{
    SensitiveString,
    margin::{
        BASE_URL_API, BASE_URL_STREAM,
        http::{PrivateClient, PrivateConfig},
        ws::{IncomingMessage, OutgoingMessage},
    },
    ws::{Config, Event, Stream},
};
use tokio::time::sleep;
use tracing::{Level, info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api_key = std::env::var("API_KEY").expect("environment variable API_KEY is required");
    let api_secret =
        std::env::var("API_SECRET").expect("environment variable API_SECRET is required");
    let http_client = PrivateClient::new(PrivateConfig::new(
        BASE_URL_API,
        SensitiveString::from(api_key),
        SensitiveString::from(api_secret),
    ));

    // 1. Mint a listenKey.
    let listen_key = http_client.create_listen_key().await?.result.listen_key;
    info!(%listen_key, "listen key acquired");

    // 2. Connect the WebSocket. Outgoing is the empty enum — server-push only.
    let url = format!("{BASE_URL_STREAM}/ws/{listen_key}");
    let (handle, mut events) = Stream::<OutgoingMessage, IncomingMessage>::new(Config::new(url));
    let _ = handle.connect().await;

    // 3. Drain events for ~60s, then disconnect.
    let drain = tokio::spawn(async move {
        sleep(Duration::from_secs(60)).await;
        let _ = handle.disconnect().await;
    });

    while let Some(event) = events.recv().await {
        match event {
            Event::Message(msg) => info!(?msg, "user data event"),
            Event::Disconnected { .. } => break,
            other => info!(?other, "stream event"),
        }
    }
    let _ = drain.await;

    // 4. Release the listenKey. Skipping this leaves it valid for up to 60
    //    minutes — usually harmless, but cleaner to close explicitly.
    if let Err(e) = http_client.close_listen_key(&listen_key).await {
        warn!(?e, "close_listen_key failed");
    }

    Ok(())
}
