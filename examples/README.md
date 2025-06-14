# Examples of how to use patisson-binance-sdk

This directory contains a number of examples showcasing various capabilities of the `patisson-binance-sdk` crate.

## Example list

`server-time`

All examples can be executed with:

```sh
cargo run --example $example_name
```

### Server time

```rs
use binance::spot::{BASE_URL_API, Client, ClientConfig};

let cfg = ClientConfig {
    base_url: BASE_URL_API.to_string(),
    api_key: None,
    api_secret: None,
};
let client = Client::new(cfg);
let response = client.get_server_time().await?;
println!("{response:#?}");
```
