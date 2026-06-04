//! Run with
//!
//! ```not_rust
//! cargo run --example account-information
//! ```

use tokio;

use binance::{
    SensitiveString,
    spot::{
        BASE_URL_API,
        http::{AccountClient, GetAccountInformationParams},
    },
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

    let params = GetAccountInformationParams {
        omit_zero_balances: Some(true),
        recv_window: None,
        timestamp: timestamp(),
    };

    let response = client.account_information(params).await?;
    println!("{response:#?}");

    Ok(())
}
