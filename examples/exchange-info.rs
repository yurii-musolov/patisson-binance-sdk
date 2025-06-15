//! Run with
//!
//! ```not_rust
//! cargo run --example exchange-info
//! ```

use tokio;

use binance::spot::{BASE_URL_API, Client, ClientConfig, GetExchangeInfoParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = ClientConfig {
        base_url: BASE_URL_API.to_string(),
        api_key: None,
        api_secret: None,
    };
    let client = Client::new(cfg);
    let params = GetExchangeInfoParams {
        // symbol: None,
        symbol: Some(String::from("BTCUSDT")),
        symbols: None,
        permissions: None,
        show_permission_sets: None,
        symbol_status: None,
    };
    let response = client.get_exchange_info(params).await?;
    println!("{response:#?}");

    Ok(())
}
