//! Run with
//!
//! ```not_rust
//! cargo run --example spot-test-new-order
//! ```

use anyhow::bail;
use binance::{
    SensitiveString,
    spot::{
        BASE_URL_API, OrderResponseType, OrderSide, OrderType,
        http::{NewOrderRequest, PrivateClient, PrivateConfig},
    },
    timestamp,
};
use rust_decimal::dec;
use tokio;
use tracing::{Level, error, info};
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

    let params = NewOrderRequest::new(
        "BTCUSDT",
        OrderSide::BUY,
        OrderType::Market,
        OrderResponseType::FULL,
        timestamp(),
    )
    .quantity(dec!(0.0002))
    .compute_commission_rates(true);
    if !params.is_valid() {
        error!(?params, "not valid params");
        bail!("not valid params")
    }

    let response = client.test_new_order(params).await?;
    info!(?response, "response");

    Ok(())
}
