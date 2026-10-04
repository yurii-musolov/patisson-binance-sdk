//! Run with
//!
//! ```not_rust
//! cargo run --example margin-borrow-repay
//! ```

mod support;

use binance::margin::{
    BorrowRepayType, IsIsolated,
    http::{BorrowRepayParams, PrivateClient, PrivateConfig},
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

    let api = support::api(support::Product::Margin)?;

    let (api_key, api_secret) = support::credentials()?;
    let cfg = PrivateConfig::new(api, api_key, api_secret);
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
