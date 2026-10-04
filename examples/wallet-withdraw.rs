//! Run with
//!
//! ```not_rust
//! cargo run --example wallet-withdraw
//! ```

mod support;

use binance::wallet::http::{PrivateClient, PrivateConfig, WithdrawRequest};
use rust_decimal::dec;
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

    // This is a real, irreversible withdrawal once submitted. Double-check
    // `coin` / `network` / `address` / `amount` before running against mainnet.
    let params =
        WithdrawRequest::new("USDT", "TXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX", dec!(1)).network("TRX");

    let response = client.withdraw(params).await?;
    info!(?response, "response");

    Ok(())
}
