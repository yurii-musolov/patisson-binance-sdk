//! Run with
//!
//! ```not_rust
//! cargo run --example wallet-account-status
//! ```

mod support;

use binance::wallet::http::{GetAccountStatusParams, PrivateClient, PrivateConfig};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api = support::api(support::Product::Wallet)?;

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::new(api, api_key, api_secret);
    let client = PrivateClient::new(cfg)?;

    let params = GetAccountStatusParams::new();
    let response = client.get_account_status(params).await?;
    info!(?response, "response");

    Ok(())
}
