//! Run with
//!
//! ```not_rust
//! cargo run --example usdm-stream-public
//! ```

use std::time::Duration;

use binance::{
    derivatives::usds_margined_futures::{
        BASE_URL_STREAM, KlineInterval,
        ws::{IncomingMessage, OutgoingMessage, StreamName},
    },
    ws::{Config, Event, Stream},
};
use tokio::time::sleep;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let url = format!("{BASE_URL_STREAM}/stream");
    let symbol = String::from("BTCUSDT").to_lowercase();
    let messages = {
        let kline = StreamName::Kline {
            symbol: symbol.clone(),
            interval: KlineInterval::Minute1,
        };
        let agg = StreamName::AggTrade {
            symbol: symbol.clone(),
        };

        vec![
            OutgoingMessage::Subscribe {
                id: Some("req-0001".into()),
                params: vec![kline.clone()],
            },
            OutgoingMessage::Unsubscribe {
                id: Some("req-0002".into()),
                params: vec![kline],
            },
            OutgoingMessage::Subscribe {
                id: Some("req-0003".into()),
                params: vec![agg.clone()],
            },
            OutgoingMessage::Unsubscribe {
                id: Some("req-0004".into()),
                params: vec![agg],
            },
        ]
    };

    let cfg = Config::new(url);
    let (handle, mut events) = Stream::<OutgoingMessage, IncomingMessage>::new(cfg);

    tokio::spawn(async move {
        let _ = handle.connect().await;

        for message in messages {
            let _ = handle.send_command(message).await;
            sleep(Duration::from_secs(15)).await;
        }

        let _ = handle.disconnect().await;
    });

    while let Some(event) = events.recv().await {
        info!(?event);
        if matches!(event, Event::Disconnected { reason: _ }) {
            break;
        }
    }

    Ok(())
}
