# Examples of how to use patisson-binance-sdk

This directory contains a number of examples showcasing various capabilities of the `patisson-binance-sdk` crate.

## Example list

`account-information`, `exchange-info`, `kline`, `query-order`, `server-time`, `test-new-order`, `ticker-statistics`

All examples can be executed with:

```sh
cargo run --example $example_name
```

## Environment variables

Some examples that perform queries on private data expect these environment variables:

```sh
export API_KEY="xxxxxxxx"
export API_SECRET="xxxxxxxx"
```
