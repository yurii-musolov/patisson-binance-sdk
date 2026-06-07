//! Run with
//!
//! ```not_rust
//! cargo run --example account-information
//! ```

use binance::{
    SensitiveString,
    spot::{
        BASE_URL_API,
        http::{GetAccountInformationParams, PrivateClient, PrivateConfig},
    },
    timestamp,
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

    let api_key = std::env::var("API_KEY").expect("environment variable API_KEY is required");
    let api_key = SensitiveString::from(api_key);
    let api_secret =
        std::env::var("API_SECRET").expect("environment variable API_SECRET is required");
    let api_secret = SensitiveString::from(api_secret);
    let cfg = PrivateConfig::new(BASE_URL_API, api_key, api_secret);
    let client = PrivateClient::new(cfg);

    let params = GetAccountInformationParams {
        omit_zero_balances: Some(true),
        recv_window: None,
        timestamp: timestamp(),
    };

    let response = client.account_information(params).await?;
    info!(?response, "response");

    Ok(())
}
