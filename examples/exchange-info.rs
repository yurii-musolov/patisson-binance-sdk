//! Run with
//!
//! ```not_rust
//! cargo run --example exchange-info
//! ```

use tokio;

use binance::spot::{
    BASE_URL_API,
    http::{GeneralClient, GetExchangeInfoParams},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = GeneralClient::new(BASE_URL_API.into());

    let params = GetExchangeInfoParams {
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
