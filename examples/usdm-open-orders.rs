//! Run with
//!
//! ```not_rust
//! cargo run --example usdm-open-orders
//! ```

use binance::{
    SensitiveString,
    derivatives::usds_margined_futures::{
        BASE_URL_API,
        http::{GetOpenOrdersParams, PrivateClient, PrivateConfig},
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

    let api_key = std::env::var("API_KEY").expect("environment variable API_KEY is required");
    let api_key = SensitiveString::from(api_key);
    let api_secret =
        std::env::var("API_SECRET").expect("environment variable API_SECRET is required");
    let api_secret = SensitiveString::from(api_secret);
    let cfg = PrivateConfig::new(BASE_URL_API, api_key, api_secret);
    let client = PrivateClient::new(cfg);

    let params = GetOpenOrdersParams::new().symbol("BTCUSDT");

    let response = client.get_open_orders(params).await?;
    info!(?response, "response");

    Ok(())
}
