//! Run with
//!
//! ```not_rust
//! cargo run --example kline
//! ```

use tokio;

use binance::spot::{BASE_URL_API, Client, ClientConfig, GetKlineListParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = ClientConfig {
        base_url: BASE_URL_API.to_string(),
        api_key: None,
        api_secret: None,
    };
    let client = Client::new(cfg);
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
