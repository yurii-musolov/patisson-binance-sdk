//! Run with
//!
//! ```not_rust
//! cargo run --example exchange-info
//! ```

use binance::spot::{
    BASE_URL_API,
    http::{GetExchangeInfoParams, PublicClient, PublicConfig},
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

    let cfg = PublicConfig::new(BASE_URL_API);
    let client = PublicClient::new(cfg);

    let params = GetExchangeInfoParams {
        symbol: Some(String::from("BTCUSDT")),
        symbols: None,
        permissions: None,
        show_permission_sets: None,
        symbol_status: None,
    };
    let response = client.get_exchange_info(params).await?;
    info!(?response, "response");

    Ok(())
}
