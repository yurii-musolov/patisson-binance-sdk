//! Run with
//!
//! ```not_rust
//! cargo run --example server-time
//! ```

use tokio;

use binance::spot::{BASE_URL_API, GeneralClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = GeneralClient::new(BASE_URL_API.into());

    let response = client.get_server_time().await?;
    println!("{response:#?}");

    Ok(())
}
