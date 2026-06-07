//! Run with
//!
//! ```not_rust
//! cargo run --example kline
//! ```

use binance::spot::{
    BASE_URL_API,
    http::{GetKlineListParams, PublicClient, PublicConfig},
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

    let params =
        GetKlineListParams::new("BTCUSDT", binance::spot::KlineInterval::Minute1).limit(2);
    let response = client.get_kline_list(params).await?;
    info!(?response, "response");

    Ok(())
}
