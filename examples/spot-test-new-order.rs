//! Run with
//!
//! ```not_rust
//! cargo run --example spot-test-new-order
//! ```

mod support;

use anyhow::bail;
use binance::spot::{
    OrderResponseType, OrderSide, OrderType,
    http::{NewOrderRequest, PrivateClient, PrivateConfig},
};
use rust_decimal::dec;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api = support::api(support::Product::Spot)?;

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::new(api, api_key, api_secret);
    let client = PrivateClient::new(cfg)?;

    let params = NewOrderRequest::new(
        "BTCUSDT",
        OrderSide::BUY,
        OrderType::Market,
        OrderResponseType::FULL,
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
