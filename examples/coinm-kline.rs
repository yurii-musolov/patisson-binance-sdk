//! Run with
//!
//! ```not_rust
//! cargo run --example coinm-kline
//! ```

use binance::derivatives::coin_margined_futures::{
    BASE_URL_API, KlineInterval,
    http::{GetKlineListParams, PublicClient, PublicConfig},
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cfg = PublicConfig::new(BASE_URL_API);
    let client = PublicClient::new(cfg)?;

    // COIN-M symbols carry a contract suffix (e.g. perpetual is "_PERP").
    let params = GetKlineListParams::new("BTCUSD_PERP", KlineInterval::Minute1).limit(2);
    let response = client.get_kline_list(params).await?;
    info!(?response, "response");

    Ok(())
}
