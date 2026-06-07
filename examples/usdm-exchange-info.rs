//! Run with
//!
//! ```not_rust
//! cargo run --example usdm-exchange-info
//! ```

use binance::derivatives::usds_margined_futures::{
    BASE_URL_API,
    http::{PublicClient, PublicConfig},
};
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

    let response = client.get_exchange_info().await?;
    info!(
        timezone = %response.result.timezone,
        server_time = response.result.server_time,
        symbol_count = response.result.symbols.len(),
        "exchange info"
    );

    Ok(())
}
