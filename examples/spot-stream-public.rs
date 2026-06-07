//! Run with
//!
//! ```not_rust
//! cargo run --example spot-stream-public
//! ```

use binance::{
    spot::{
        BASE_URL_MARKET_DATA_STREAM1, KlineInterval, Path,
        ws::{IncomingMessage, OutgoingMessage, StreamName},
    },
    ws::{Config, Event, Stream},
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let url = format!("{}{}", BASE_URL_MARKET_DATA_STREAM1, Path::Stream);
    let symbol = String::from("BTCUSDT").to_lowercase(); // All symbols for streams are lowercase
    let mut id = {
        let mut id = id_generator("request-id");
        move || Some(id().into())
    };
    let messages = {
        let kline = StreamName::Kline {
            symbol: symbol.clone(),
            interval: KlineInterval::Minute1,
        };
        let trade = StreamName::Trade {
            symbol: symbol.clone(),
        };

        vec![
            OutgoingMessage::Subscribe {
                id: id(),
                params: vec![kline.clone()],
            },
            OutgoingMessage::Unsubscribe {
                id: id(),
                params: vec![kline],
            },
            OutgoingMessage::Subscribe {
                id: id(),
                params: vec![trade.clone()],
            },
            OutgoingMessage::Unsubscribe {
                id: id(),
                params: vec![trade],
            },
        ]
    };

    let cfg = Config::new(url);
    let (handle, mut events) = Stream::<OutgoingMessage, IncomingMessage>::new(cfg);

    tokio::spawn(async move {
        let _ = handle.connect().await;

        for (i, message) in messages.into_iter().enumerate() {
            info!(?message, "send message");
            let _ = handle.send_command(message).await;
            if i % 2 == 0 {
                sleep(Duration::from_mins(1)).await; // subs
            } else {
                sleep(Duration::from_secs(10)).await; // unsubs
            }
        }

        let _ = handle.disconnect().await;
    });

    while let Some(event) = events.recv().await {
        info!(?event, "receive message");
        if let Event::Disconnected { reason } = event {
            info!(?reason, "disconnected");
            break;
        }
    }

    Ok(())
}

pub fn id_generator(prefix: impl Into<String>) -> impl FnMut() -> String {
    let prefix = prefix.into();
    let mut counter = 0_u64;

    move || {
        counter += 1;
        format!("{prefix}_{counter}")
    }
}
