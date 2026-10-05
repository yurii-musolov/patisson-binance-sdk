//! USDⓈ-M Futures user data stream:
//!
//! 1. Create a listenKey and keep it alive in the background
//!    (`keep_listen_key_alive`, a keepalive every 30 minutes)
//! 2. Connect to `<stream>/private/ws/<listenKey>`
//! 3. Log account events for ~60 s, then disconnect and close the listenKey
//!
//! Run with
//!
//! ```not_rust
//! BINANCE_ENV=demo cargo run --example usdm-user-data-stream
//! ```
//!
//! Needs `API_KEY` / `API_SECRET` (see examples/README.md). The listenKey
//! grants read access to the account's events, so it is never logged.

mod support;

use std::time::Duration;

use binance::{
    derivatives::usds_margined_futures::{
        Path,
        http::{PrivateClient, PrivateConfig},
        ws::{OutgoingMessage, UserDataMessage},
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

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::for_env(support::env()?, api_key, api_secret)?;
    let client = PrivateClient::new(cfg)?;

    let listen_key = client.create_listen_key().await?.result.listen_key;
    info!("listen key acquired");
    let mut keeper = client.keep_listen_key_alive();

    let stream = support::stream(support::Product::UsdmFutures)?;
    let url = format!("{stream}{}/ws/{listen_key}", Path::Private);
    let (handle, mut events) =
        Stream::<OutgoingMessage, UserDataMessage>::new(Config::futures(url));
    handle.connect().await?;

    let stop = tokio::spawn(async move {
        sleep(Duration::from_secs(60)).await;
        let _ = handle.disconnect().await;
    });

    loop {
        tokio::select! {
            event = events.recv() => match event {
                Some(Event::Message(UserDataMessage::OrderTradeUpdate(e))) => info!(
                    symbol = %e.order.symbol,
                    status = ?e.order.order_status,
                    filled = %e.order.cumulative_filled_quantity,
                    "order update"
                ),
                Some(Event::Message(UserDataMessage::AccountUpdate(e))) => {
                    info!(reason = ?e.update.reason, "account update");
                    for p in &e.update.positions {
                        info!(symbol = %p.symbol, side = ?p.position_side, amount = %p.position_amount, "position");
                    }
                }
                Some(Event::Message(UserDataMessage::ListenKeyExpired(_))) => {
                    warn!("listen key expired");
                    break;
                }
                Some(Event::Message(msg)) => info!(?msg, "user data event"),
                Some(Event::Disconnected { .. }) | None => break,
                Some(other) => info!(?other, "stream event"),
            },
            Some(error) = keeper.next_error() => warn!(%error, "listen key keepalive failed"),
        }
    }
    let _ = stop.await;
    keeper.stop();

    if let Err(e) = client.close_listen_key().await {
        warn!(%e, "close_listen_key failed");
    }
    Ok(())
}
