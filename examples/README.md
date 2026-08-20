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

`spot-account-information`, `spot-all-orders`, `spot-cancel-order`,
`spot-exchange-info`, `spot-kline`, `spot-my-trades`, `spot-open-orders`,
`spot-order-book`, `spot-query-order`, `spot-server-time`,
`spot-stream-public`, `spot-test-new-order`, `spot-ticker-statistics`

### USDⓈ-M Futures

`usdm-account-information`, `usdm-cancel-order`, `usdm-change-leverage`,
`usdm-exchange-info`, `usdm-kline`, `usdm-open-orders`, `usdm-order-book`,
`usdm-server-time`, `usdm-stream-public`

### COIN-M Futures

`coinm-account-information`, `coinm-cancel-order`, `coinm-change-leverage`,
`coinm-exchange-info`, `coinm-kline`, `coinm-open-orders`, `coinm-order-book`,
`coinm-server-time`, `coinm-stream-public`

### Margin

`margin-account`, `margin-borrow-repay`, `margin-cancel-order`,
`margin-max-borrowable`, `margin-user-data-stream`

### Wallet

`wallet-account-status`, `wallet-deposit-address`, `wallet-system-status`,
`wallet-withdraw`

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

Any example that isn't public market data (server time, exchange info,
klines, order book, public streams, `wallet-system-status`) talks to a
signed endpoint and needs them — e.g. `*-account-information`,
`*-cancel-order`, `*-open-orders`, `spot-all-orders`, `spot-my-trades`,
`spot-query-order`, `spot-test-new-order`, `margin-borrow-repay`,
`wallet-withdraw`.

`*-cancel-order` and `wallet-withdraw` act on a real account (cancelling a
live order / submitting a withdrawal) — read them before running against
mainnet.
