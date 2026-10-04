//! Run with
//!
//! ```not_rust
//! cargo run --example spot-ticker-statistics
//! ```

mod support;

use binance::spot::http::{
    GetTickerPriceChangeStatisticsParams, PublicClient, PublicConfig, SymbolOrSymbols,
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api = support::api(support::Product::Spot)?;

    let cfg = PublicConfig::new(api);
    let client = PublicClient::new(cfg)?;

    let params =
        GetTickerPriceChangeStatisticsParams::Full(SymbolOrSymbols::new().symbol("BTCUSDT"));
    let response = client.ticker_price_change_statistics(params).await?;
    info!(?response, "response");

    Ok(())
}
