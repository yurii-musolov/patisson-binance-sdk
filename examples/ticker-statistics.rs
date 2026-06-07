//! Run with
//!
//! ```not_rust
//! cargo run --example ticker-statistics
//! ```

use binance::spot::{
    BASE_URL_API,
    http::{GetTickerPriceChangeStatisticsParams, PublicClient, PublicConfig, SymbolOrSymbols},
};
use tokio;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cfg = PublicConfig::new(BASE_URL_API);
    let client = PublicClient::new(cfg);

    let params = GetTickerPriceChangeStatisticsParams::Full(SymbolOrSymbols {
        symbol: Some(String::from("BTCUSDT")),
        symbols: None,
    });
    let response = client.ticker_price_change_statistics(params).await?;
    info!(?response, "response");

    Ok(())
}
