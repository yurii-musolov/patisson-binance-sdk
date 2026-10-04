//! Run with
//!
//! ```not_rust
//! cargo run --example spot-exchange-info
//! ```

mod support;

use binance::spot::http::{GetExchangeInfoParams, PublicClient, PublicConfig};
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

    let params = GetExchangeInfoParams::new().symbol("BTCUSDT");
    let response = client.get_exchange_info(params).await?;
    info!(?response, "response");

    Ok(())
}
