//! Run with
//!
//! ```not_rust
//! cargo run --example query-order
//! ```

use tokio;

use binance::{
    SensitiveString,
    spot::{AccountClient, BASE_URL_API, QueryOrderParams},
    timestamp,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("API_KEY").expect("environment variable API_KEY is required");
    let api_key = SensitiveString::from(api_key);
    let api_secret =
        std::env::var("API_SECRET").expect("environment variable API_SECRET is required");
    let api_secret = SensitiveString::from(api_secret);
    let client = AccountClient::new(BASE_URL_API.into(), api_key, api_secret);

    let params = QueryOrderParams {
        symbol: String::from("BTCUSDT"),
        recv_window: None,
        timestamp: timestamp(),
        order_id: Some(123456789),
        orig_client_order_id: None,
    };

    let response = client.query_order(params).await?;
    println!("{response:#?}");

    Ok(())
}
