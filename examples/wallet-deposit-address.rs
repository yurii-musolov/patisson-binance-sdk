//! Run with
//!
//! ```not_rust
//! cargo run --example wallet-deposit-address
//! ```

mod support;

use binance::wallet::http::{GetDepositAddressParams, PrivateClient, PrivateConfig};
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

    // Pick a network with `.network("BTC")` / `.network("TRX")` etc.; omit to get the default.
    let params = GetDepositAddressParams::new("USDT").network("TRX");
    let response = client.get_deposit_address(params).await?;
    info!(?response, "response");

    Ok(())
}
