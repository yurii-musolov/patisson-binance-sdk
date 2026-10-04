//! Run with
//!
//! ```not_rust
//! cargo run --example usdm-change-leverage
//! ```

mod support;

use binance::derivatives::usds_margined_futures::http::{
    ChangeInitialLeverageParams, PrivateClient, PrivateConfig,
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api = support::api(support::Product::UsdmFutures)?;

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::new(api, api_key, api_secret);
    let client = PrivateClient::new(cfg)?;

    let params = ChangeInitialLeverageParams::new("BTCUSDT", 10);

    let response = client.change_initial_leverage(params).await?;
    info!(?response, "response");

    Ok(())
}
