//! Run with
//!
//! ```not_rust
//! cargo run --example server-time
//! ```

use tokio;

use binance::spot::{BASE_URL_API, Client, ClientConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = ClientConfig {
        base_url: BASE_URL_API.to_string(),
        api_key: None,
        api_secret: None,
    };
    let client = Client::new(cfg);

    let response = client.get_server_time().await?;
    println!("{response:#?}");

    Ok(())
}
