//! Run with
//!
//! ```not_rust
//! cargo run --example ticker-statistics
//! ```

use tokio;

use binance::spot::{BASE_URL_API, MarketClient, SymbolOrSymbols};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = MarketClient::new(BASE_URL_API.into());

    let params = binance::spot::GetTickerPriceChangeStatisticsParams::Full(SymbolOrSymbols {
        symbol: Some(String::from("BTCUSDT")),
        symbols: None,
    });
    let response = client.ticker_price_change_statistics(params).await?;
    println!("{response:#?}");

    Ok(())
}
