//! Run with
//!
//! ```not_rust
//! cargo run --example margin-max-borrowable
//! ```

mod support;

use binance::margin::http::{GetMaxBorrowableParams, PrivateClient, PrivateConfig};
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

    // Cross-margin: omit isolated_symbol. For isolated margin, set it via `.isolated_symbol("BTCUSDT")`.
    let params = GetMaxBorrowableParams::new("USDT");
    let response = client.max_borrowable(params).await?;
    info!(?response, "response");

    Ok(())
}
