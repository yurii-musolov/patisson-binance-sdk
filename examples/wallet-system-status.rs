//! Run with
//!
//! ```not_rust
//! cargo run --example wallet-system-status
//! ```

use binance::{
    SensitiveString,
    wallet::{
        BASE_URL_API,
        http::{PrivateClient, PrivateConfig},
    },
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // `system_status` is a public endpoint (no signature required), but the
    // wallet module only exposes `PrivateClient` — any non-empty key/secret
    // works here since they're not actually checked by Binance for this call.
    let api_key = SensitiveString::from("unused");
    let api_secret = SensitiveString::from("unused");
    let cfg = PrivateConfig::new(BASE_URL_API, api_key, api_secret);
    let client = PrivateClient::new(cfg)?;

    let response = client.system_status().await?;
    info!(?response, "response");

    Ok(())
}
