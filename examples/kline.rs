//! Run with
//!
//! ```not_rust
//! cargo run --example kline
//! ```

use tokio;

use binance::spot::{BASE_URL_API, GetKlineListParams, MarketClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = MarketClient::new(BASE_URL_API.into());

    let params = GetKlineListParams {
        symbol: String::from("BTCUSDT"),
        interval: binance::spot::KlineInterval::Minute1,
        start_time: None,
        end_time: None,
        time_zone: None,
        limit: Some(2),
    };
    let response = client.get_kline_list(params).await?;
    println!("{response:#?}");

    Ok(())
}
