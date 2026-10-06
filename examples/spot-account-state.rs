//! Spot balances and open orders kept current from the user data stream:
//!
//! 1. Subscribe to the user data stream (WebSocket API); events received
//!    meanwhile wait in the channel
//! 2. Load the REST snapshot (`AccountState::load`), then apply events
//! 3. After every reconnect or `Lagged`, reload the snapshot (events may
//!    have been missed); older events are ignored, so nothing is counted
//!    twice
//!
//! Run with
//!
//! ```not_rust
//! BINANCE_ENV=demo cargo run --example spot-account-state
//! ```
//!
//! Needs `API_KEY` / `API_SECRET` (see examples/README.md). Runs for ~60 s.

mod support;

use std::time::Duration;

use binance::{
    TimeOffset,
    spot::{
        AccountChange, AccountState, Path,
        http::{PrivateClient, PrivateConfig},
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

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::for_env(support::env()?, api_key.clone(), api_secret.clone())?;
    let client = PrivateClient::new(cfg)?;

    let url = format!(
        "{}{}",
        support::ws_api(support::Product::Spot)?,
        Path::WebSocketApiV3
    );
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
        let _ = stopper.disconnect().await;
    });

    let mut state: Option<AccountState> = None;
    while let Some(event) = events.recv().await {
        match event {
            // First connect and every reconnect: (re)load the snapshot.
            Event::Connected => {
                let account = match state.as_mut() {
                    Some(account) => {
                        account.reload(&client).await?;
                        account
                    }
                    None => state.insert(AccountState::load(&client).await?),
                };
                for (asset, balance) in account.balances().filter(|(_, b)| !b.total().is_zero()) {
                    info!(asset, free = %balance.free, locked = %balance.locked, "balance");
                }
                info!(
                    open_orders = account.open_orders().count(),
                    "snapshot loaded"
                );
            }
            Event::Message(IncomingMessage::Event(message)) => {
                let Some(account) = state.as_mut() else {
                    continue;
                };
                for change in account.apply(&message.event) {
                    match change {
                        AccountChange::Balance { asset, balance } => {
                            info!(%asset, free = %balance.free, locked = %balance.locked, "balance changed")
                        }
                        AccountChange::Order(order) => info!(
                            symbol = %order.symbol,
                            client_order_id = %order.client_order_id,
                            status = ?order.status,
                            filled = %order.filled_qty,
                            "order changed"
                        ),
                    }
                }
            }
            // Dropped events are a gap too.
            Event::Lagged { .. } => {
                if let Some(account) = state.as_mut() {
                    account.reload(&client).await?;
                }
            }
            Event::Message(IncomingMessage::Response(response)) => {
                if let Some(error) = &response.error {
                    warn!(code = %error.code, msg = %error.msg, "request failed");
                }
            }
            Event::Disconnected { reason } => {
                info!(?reason, "disconnected");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
