//! Run with
//!
//! ```not_rust
//! cargo run --example spot-server-time
//! ```

use binance::spot::{
    BASE_URL_API,
    http::{PublicClient, PublicConfig},
};
use std::time::Instant;
use tokio;
use tracing::{Level, debug, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cfg = PublicConfig::new(BASE_URL_API);
    let client = PublicClient::new(cfg);

    let start = Instant::now();
    let response = client.get_server_time().await?;
    let duration = start.elapsed();

    info!(?response, "response");
    debug!(?duration, "duration");

    Ok(())
}
