//! Run with
//!
//! ```not_rust
//! cargo run --example coinm-order-book
//! ```
//!
//! Maintains a local COIN-M Futures order book. The synchronization
//! protocol (snapshot + diff stream, `pu`/`u` chain verification, bridging,
//! resync on gaps) is fully encapsulated by [`OrderBookState`]. This example
//! just forwards diffs and snapshots to it and reacts to the returned
//! [`ApplyOutcome`].

use anyhow::Ok;
use binance::{
    derivatives::coin_margined_futures::{
        ApplyOutcome, BASE_URL_API, BASE_URL_STREAM, OrderBookState, Path,
        http::{GetOrderBookParams, OrderBook, PublicClient, PublicConfig, Response},
        ws::{DepthUpdateMsg, IncomingMessage, OutgoingMessage, StreamMessage, StreamName},
    },
    ws::{Config, Event, Stream},
};
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};
use tracing::{Level, info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish(),
    )?;

    let symbol_rest = "BTCUSD_PERP";
    let symbol_stream = symbol_rest.to_lowercase();

    // ----- WS diff stream -----
    let url = format!("{BASE_URL_STREAM}{}", Path::Stream);
    let cfg = Config::new(url)
        .ping_interval(Duration::from_secs(180))
        .pong_timeout(Duration::from_secs(600));
    let (handle, mut events) = Stream::<OutgoingMessage, IncomingMessage>::new(cfg);

    let stream = StreamName::Depth {
        symbol: symbol_stream,
    };
    tokio::spawn(async move {
        let _ = handle.connect().await;
        let _ = handle
            .send_command(OutgoingMessage::Subscribe {
                id: Some("sub".into()),
                params: vec![stream],
            })
            .await;
        sleep(Duration::from_secs(60)).await;
        let _ = handle.disconnect().await;
    });

    // ----- Snapshot fetcher task: refetch on demand -----
    let (snap_tx, mut snap_rx) = mpsc::channel::<OrderBook>(4);
    let (refetch_tx, mut refetch_rx) = mpsc::channel::<()>(4);
    let client = PublicClient::new(PublicConfig::new(BASE_URL_API));
    let symbol = symbol_rest.to_string();
    tokio::spawn(async move {
        while refetch_rx.recv().await.is_some() {
            let params = GetOrderBookParams::new(symbol.clone()).limit(1000);
            match client.get_order_book(params).await {
                std::result::Result::Ok(Response { result, .. }) => {
                    info!(last_update_id = result.last_update_id, "snapshot fetched");
                    if snap_tx.send(result).await.is_err() {
                        break;
                    }
                }
                Err(e) => warn!(?e, "snapshot fetch failed"),
            }
        }
    });
    refetch_tx.send(()).await.ok();

    // ----- State machine + event loop -----
    let mut book = OrderBookState::new();
    loop {
        tokio::select! {
            event = events.recv() => {
                let Some(event) = event else { break; };
                match event {
                    Event::Message(msg) => {
                        if let Some(diff) = extract_depth_update(msg) {
                            handle_outcome(book.apply_diff(diff), &book, &refetch_tx).await;
                        }
                    }
                    Event::Disconnected { .. } => break,
                    _ => {}
                }
            }
            snapshot = snap_rx.recv() => {
                let Some(snapshot) = snapshot else { break; };
                handle_outcome(book.apply_snapshot(snapshot), &book, &refetch_tx).await;
            }
        }
    }

    Ok(())
}

async fn handle_outcome(
    outcome: ApplyOutcome,
    book: &OrderBookState,
    refetch_tx: &mpsc::Sender<()>,
) {
    match outcome {
        ApplyOutcome::Synced => {
            info!(
                last_update_id = book.last_update_id(),
                bids = book.bids().map(|b| b.len()),
                asks = book.asks().map(|a| a.len()),
                best_bid = ?book.best_bid(),
                best_ask = ?book.best_ask(),
                "synchronized"
            );
        }
        ApplyOutcome::Applied => {
            info!(
                last_update_id = book.last_update_id(),
                best_bid = ?book.best_bid(),
                best_ask = ?book.best_ask(),
                "applied"
            );
        }
        ApplyOutcome::ResyncRequired => {
            warn!("resync required; requesting new snapshot");
            refetch_tx.send(()).await.ok();
        }
        ApplyOutcome::Buffered | ApplyOutcome::Ignored => {}
    }
}

fn extract_depth_update(msg: IncomingMessage) -> Option<DepthUpdateMsg> {
    match msg {
        IncomingMessage::Stream(StreamMessage::DepthUpdate(msg)) => Some(msg),
        IncomingMessage::CombinedStream(msg) => {
            if let StreamMessage::DepthUpdate(msg) = msg.data {
                Some(msg)
            } else {
                None
            }
        }
        _ => None,
    }
}
