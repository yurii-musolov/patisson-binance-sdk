//! Run with
//!
//! ```not_rust
//! cargo run --example spot-server-time
//! ```

mod support;

use binance::spot::http::{PublicClient, PublicConfig};
use std::time::Instant;
use tracing::{Level, debug, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let api = support::api(support::Product::Spot)?;

    let cfg = PublicConfig::new(api);
    let client = PublicClient::new(cfg)?;

    let start = Instant::now();
    let response = client.get_server_time().await?;
    let duration = start.elapsed();

    info!(?response, "response");
    debug!(?duration, "duration");

    Ok(())
}
