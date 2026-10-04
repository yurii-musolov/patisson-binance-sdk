//! Run with
//!
//! ```not_rust
//! cargo run --example margin-cancel-order
//! ```

mod support;

use binance::margin::http::{CancelOrderParams, PrivateClient, PrivateConfig};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api = support::api(support::Product::Margin)?;

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::new(api, api_key, api_secret);
    let client = PrivateClient::new(cfg)?;

    let params = CancelOrderParams::new("BTCUSDT").order_id(123456789);
    let response = client.cancel_order(params).await?;
    info!(?response, "response");

    Ok(())
}
