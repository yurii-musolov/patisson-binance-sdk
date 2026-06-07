# Examples of how to use patisson-binance-sdk

This directory contains examples showcasing capabilities of the
`patisson-binance-sdk` crate. Examples are prefixed by product:

- `spot-*` — Spot
- `usdm-*` — USDⓈ-M Futures
- `coinm-*` — COIN-M Futures
- `margin-*` — Margin Trading
- `wallet-*` — Wallet (deposits, withdrawals, account status)

## Example list

### Spot

`spot-account-information`, `spot-exchange-info`, `spot-kline`,
`spot-query-order`, `spot-server-time`, `spot-stream-public`,
`spot-test-new-order`, `spot-ticker-statistics`

### USDⓈ-M Futures

`usdm-account-information`, `usdm-exchange-info`, `usdm-kline`,
`usdm-server-time`, `usdm-stream-public`

### COIN-M Futures

`coinm-account-information`, `coinm-exchange-info`, `coinm-kline`,
`coinm-server-time`, `coinm-stream-public`

### Margin

`margin-account`, `margin-max-borrowable`, `margin-user-data-stream`

### Wallet

`wallet-account-status`, `wallet-deposit-address`

## Running

All examples can be executed with:

```sh
cargo run --example $example_name
```

For instance:

```sh
cargo run --example spot-server-time
cargo run --example usdm-kline
cargo run --example coinm-stream-public
```

## Environment variables

Examples that query private endpoints expect these environment variables:

```sh
export API_KEY="xxxxxxxx"
export API_SECRET="xxxxxxxx"
```

The `*-account-information`, `spot-query-order`, and `spot-test-new-order`
examples need them.
