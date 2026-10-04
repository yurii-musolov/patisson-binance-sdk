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

### `BINANCE_ENV`: which Binance environment to use

| Value | Environment | Products |
|---|---|---|
| unset, `prod`, `mainnet` | production | all |
| `testnet` | testnet (`testnet.binance.vision`, `testnet.binancefuture.com`) | spot, USD-M, COIN-M |
| `demo` | [demo mode](https://demo.binance.com) | spot (REST + streams), USD-M (REST only) |

Asking for a product an environment doesn't offer (e.g. `BINANCE_ENV=demo`
with a `coinm-*` example) fails with an explicit error instead of silently
falling back to production. Each environment needs its own API key: testnet
keys come from the testnet site, demo keys from demo.binance.com.

### `API_KEY` / `API_SECRET`: credentials for signed endpoints

Any example that isn't public market data (server time, exchange info,
klines, order book, public streams, `wallet-system-status`) talks to a
signed endpoint and needs them, e.g. `*-account-information`,
`*-cancel-order`, `*-open-orders`, `spot-all-orders`, `spot-my-trades`,
`spot-query-order`, `spot-test-new-order`, `margin-borrow-repay`,
`wallet-withdraw`. The examples wrap them in `SensitiveString` immediately,
so they are never printed.

Keep them in a file only you can read and load it in a subshell, so the
values appear neither on the command line nor in your shell afterwards:

```sh
mkdir -p ~/.config/binance
$EDITOR ~/.config/binance/demo.env   # API_KEY=... and API_SECRET=... lines
chmod 600 ~/.config/binance/demo.env

( set -a; . ~/.config/binance/demo.env; set +a
  BINANCE_ENV=demo cargo run --example spot-account-information )
```

`*-cancel-order` and `wallet-withdraw` act on a real account (cancelling a
live order / submitting a withdrawal) - read them before running against
production (`BINANCE_ENV` unset).
