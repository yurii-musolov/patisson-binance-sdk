//! Run with
//!
//! ```not_rust
//! cargo run --example spot-open-orders
//! ```

mod support;

use binance::spot::http::{GetOpenOrdersParams, PrivateClient, PrivateConfig};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api = support::api(support::Product::Spot)?;

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::new(api, api_key, api_secret);
    let client = PrivateClient::new(cfg)?;

    let params = GetOpenOrdersParams::new().symbol("BTCUSDT");

    let response = client.get_open_orders(params).await?;
    info!(?response, "response");

    Ok(())
}
