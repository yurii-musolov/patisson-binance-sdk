//! Run with
//!
//! ```not_rust
//! cargo run --example margin-borrow-repay
//! ```

use binance::{
    SensitiveString,
    margin::{
        BASE_URL_API, BorrowRepayType, IsIsolated,
        http::{BorrowRepayParams, PrivateClient, PrivateConfig},
    },
};
use rust_decimal::dec;
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
    let client = PrivateClient::new(cfg)?;

    // Cross-margin: omit `.symbol(...)`. For isolated margin, set it to route
    // the borrow/repay to that symbol's isolated account.
    let params = BorrowRepayParams::new(
        "USDT",
        IsIsolated::False,
        dec!(100),
        BorrowRepayType::Borrow,
    );
    let response = client.borrow_repay(params).await?;
    info!(?response, "response");

    Ok(())
}
