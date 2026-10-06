//! USDⓈ-M Futures balances, positions and open orders kept current from
//! the user data stream:
//!
//! 1. Create a listenKey, keep it alive, connect to the user data stream;
//!    events received meanwhile wait in the channel
//! 2. Load the REST snapshot (`AccountState::load`), then apply events
//! 3. After every reconnect or `Lagged`, reload the snapshot; older
//!    updates are ignored
//!
//! Run with
//!
//! ```not_rust
//! BINANCE_ENV=demo cargo run --example usdm-account-state
//! ```
//!
//! Needs `API_KEY` / `API_SECRET` (see examples/README.md). Runs for ~60 s.
//! The listenKey is never logged.

mod support;

use std::time::Duration;

use binance::{
    derivatives::usds_margined_futures::{
        AccountChange, AccountState, Path,
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
    let mut keeper = client.keep_listen_key_alive();
    let stream = support::stream(support::Product::UsdmFutures)?;
    let url = format!("{stream}{}/ws/{listen_key}", Path::Private);
    let (handle, mut events) =
        Stream::<OutgoingMessage, UserDataMessage>::new(Config::futures(url));
    handle.connect().await?;

    let stopper = handle.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(60)).await;
        let _ = stopper.disconnect().await;
    });

    let mut state: Option<AccountState> = None;
    loop {
        let event = tokio::select! {
            event = events.recv() => event,
            Some(error) = keeper.next_error() => {
                warn!(%error, "listen key keepalive failed");
                continue;
            }
        };
        match event {
            Some(Event::Connected) => {
                let account = match state.as_mut() {
                    Some(account) => {
                        account.reload(&client).await?;
                        account
                    }
                    None => state.insert(AccountState::load(&client).await?),
                };
                for (asset, b) in account
                    .balances()
                    .filter(|(_, b)| !b.wallet_balance.is_zero())
                {
                    info!(asset, wallet = %b.wallet_balance, "balance");
                }
                for p in account.positions() {
                    info!(symbol = %p.symbol, side = ?p.position_side, amount = %p.amount, entry = %p.entry_price, "position");
                }
                info!(
                    open_orders = account.open_orders().count(),
                    open_algo_orders = account.open_algo_orders().count(),
                    "snapshot loaded"
                );
            }
            Some(Event::Lagged { .. }) => {
                if let Some(account) = state.as_mut() {
                    account.reload(&client).await?;
                }
            }
            Some(Event::Message(UserDataMessage::ListenKeyExpired(_))) => {
                warn!("listen key expired");
                break;
            }
            Some(Event::Message(message)) => {
                let Some(account) = state.as_mut() else {
                    continue;
                };
                for change in account.apply(&message) {
                    match change {
                        AccountChange::Balance { asset, balance } => {
                            info!(%asset, wallet = %balance.wallet_balance, "balance changed")
                        }
                        AccountChange::Position(p) => {
                            info!(symbol = %p.symbol, side = ?p.position_side, amount = %p.amount, "position changed")
                        }
                        AccountChange::Order(o) => {
                            info!(symbol = %o.symbol, status = ?o.status, filled = %o.filled_qty, "order changed")
                        }
                        AccountChange::AlgoOrder(a) => {
                            info!(symbol = %a.symbol, status = %a.status, "algo order changed")
                        }
                    }
                }
            }
            Some(Event::Disconnected { reason }) => {
                info!(?reason, "disconnected");
                break;
            }
            None => break,
            Some(_) => {}
        }
    }
    keeper.stop();
    if let Err(e) = client.close_listen_key().await {
        warn!(%e, "close_listen_key failed");
    }
    Ok(())
}
