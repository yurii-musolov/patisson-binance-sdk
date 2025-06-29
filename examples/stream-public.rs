//! Run with
//!
//! ```not_rust
//! cargo run --example stream-public
//! ```

use std::time::Duration;

use tokio::{self, time::sleep};

use binance::spot::{
    BASE_URL_MARKET_DATA_STREAM1, KlineInterval, OutgoingMessage, Path, StreamName, stream,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = format!("{}{}", BASE_URL_MARKET_DATA_STREAM1, Path::Stream);
    let symbol = String::from("BTCUSDT").to_lowercase(); // All symbols for streams are lowercase
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
                id: String::from("req-0001"),
                params: vec![kline.clone()],
            },
            OutgoingMessage::Unsubscribe {
                id: String::from("req-0002"),
                params: vec![kline],
            },
            OutgoingMessage::Subscribe {
                id: String::from("req-0003"),
                params: vec![trade.clone()],
            },
            OutgoingMessage::Unsubscribe {
                id: String::from("req-0004"),
                params: vec![trade],
            },
        ]
    };

    let (tx, mut rx, response) = stream(&url).await?;
    println!("{response:#?}");

    tokio::spawn(async move {
        for message in messages {
            let _ = tx.send(message).await;
            sleep(Duration::from_secs(4)).await;
        }
    });

    while let Some(message) = rx.recv().await {
        println!("{message:#?}");
    }

    Ok(())
}
