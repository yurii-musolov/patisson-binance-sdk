//! Run with
//!
//! ```not_rust
//! cargo run --example usdm-kline
//! ```

mod support;

use binance::derivatives::usds_margined_futures::{
    KlineInterval,
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

    let api = support::api(support::Product::UsdmFutures)?;

    let cfg = PublicConfig::new(api);
    let client = PublicClient::new(cfg)?;

    let params = GetKlineListParams::new("BTCUSDT", KlineInterval::Minute1).limit(2);
    let response = client.get_kline_list(params).await?;
    info!(?response, "response");

    Ok(())
}
