//! Run with
//!
//! ```not_rust
//! cargo run --example server-time
//! ```

use std::time::Instant;

use tokio;

use binance::spot::{BASE_URL_API, http::GeneralClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = GeneralClient::new(BASE_URL_API.into());

    let start = Instant::now();
    let response = client.get_server_time().await?;
    let duration = start.elapsed();

    println!("{response:#?}");
    println!("Duration: {duration:?}");

    Ok(())
}
