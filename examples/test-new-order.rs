//! Run with
//!
//! ```not_rust
//! cargo run --example test-new-order
//! ```

use rust_decimal::dec;
use tokio;

use binance::{
    SensitiveString,
    spot::{
        BASE_URL_API, OrderResponseType, OrderSide, OrderType,
        http::{NewOrderRequest, TradingClient},
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
    let client = TradingClient::new(BASE_URL_API.into(), api_key, api_secret);

    let params = NewOrderRequest {
        symbol: String::from("BTCUSDT"),
        side: OrderSide::BUY,
        order_type: OrderType::Market,
        time_in_force: None,
        quantity: Some(dec!(0.00005)),
        quote_order_qty: None,
        price: None,
        new_client_order_id: None,
        strategy_id: None,
        strategy_type: None,
        stop_price: None,
        trailing_delta: None,
        iceberg_qty: None,
        new_order_resp_type: OrderResponseType::FULL,
        self_trade_prevention_mode: None,
        recv_window: None,
        timestamp: timestamp(),
    };
    if !params.is_valid() {
        println!("ERROR: not valid params: {params:#?}");
        return Ok(());
    }

    let response = client.test_new_order(params, true).await?;
    println!("{response:#?}");

    Ok(())
}
