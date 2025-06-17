//! Run with
//!
//! ```not_rust
//! cargo run --example ticker-statistics
//! ```

use tokio;

use binance::spot::{BASE_URL_API, Client, ClientConfig, SymbolOrSymbols};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = ClientConfig {
        base_url: BASE_URL_API.to_string(),
        api_key: None,
        api_secret: None,
    };
    let client = Client::new(cfg);

    let params = binance::spot::GetTickerPriceChangeStatisticsParams::Full(SymbolOrSymbols {
        symbol: Some(String::from("BTCUSDT")),
        symbols: None,
    });
    let response = client.ticker_price_change_statistics(params).await?;
    println!("{response:#?}");

    Ok(())
}
