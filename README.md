# Binance SDK

[![Crates.io](https://img.shields.io/crates/v/patisson-binance-sdk.svg)](https://crates.io/crates/patisson-binance-sdk)
[![Documentation](https://docs.rs/patisson-binance-sdk/badge.svg)](https://docs.rs/patisson-binance-sdk)
[![MIT licensed](https://img.shields.io/crates/l/patisson-binance-sdk.svg)](LICENSE)

Unofficial Rust SDK for the [Binance exchange API](https://developers.binance.com/en).

## Features

- REST API support (Spot)
- Unauthenticated endpoints
- Only async clients

## Examples

### Server time

```rs
use binance::spot::{BASE_URL_API, http::GeneralClient};

let client = GeneralClient::new(BASE_URL_API.into());

let response = client.get_server_time().await?;
println!("{response:#?}");
```

### Exchange info

```rs
use binance::spot::{BASE_URL_API, http::{GetKlineListParams, MarketClient}};

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
```

## License

This project is licensed under the [MIT license](LICENSE).
