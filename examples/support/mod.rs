//! Shared helpers for the examples: pick the Binance environment and read
//! API credentials.
//!
//! The environment comes from `BINANCE_ENV`:
//!
//! | `BINANCE_ENV`            | Environment                                   |
//! |--------------------------|-----------------------------------------------|
//! | unset, `prod`, `mainnet` | production                                    |
//! | `testnet`                | testnet (testnet.binance.vision / binancefuture.com) |
//! | `demo`                   | demo mode (demo.binance.com)                  |
//!
//! Not every product exists everywhere: margin and wallet are production
//! only and COIN-M has no demo mode. Asking for such a combination is an
//! error, not a silent fallback to production.
//!
//! Credentials come from `API_KEY` / `API_SECRET` and are wrapped in
//! `SensitiveString` right away, so they never show up in logs.
#![allow(dead_code)] // each example uses only part of this module

use anyhow::{Context, bail};
use binance::{SensitiveString, derivatives, spot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Production,
    Testnet,
    Demo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Product {
    Spot,
    UsdmFutures,
    CoinmFutures,
    Margin,
    Wallet,
}

/// The environment selected by `BINANCE_ENV` (production when unset).
pub fn env() -> anyhow::Result<Env> {
    let value = match std::env::var("BINANCE_ENV") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(Env::Production),
        Err(e) => return Err(e).context("BINANCE_ENV"),
    };
    match value.as_str() {
        "" | "prod" | "production" | "mainnet" => Ok(Env::Production),
        "testnet" => Ok(Env::Testnet),
        "demo" => Ok(Env::Demo),
        other => bail!("unknown BINANCE_ENV={other:?}; use prod, testnet or demo"),
    }
}

/// REST base URL of `product` in the selected environment.
pub fn api(product: Product) -> anyhow::Result<&'static str> {
    let env = env()?;
    let url = match (product, env) {
        (Product::Spot, Env::Production) => spot::BASE_URL_API,
        (Product::Spot, Env::Testnet) => spot::BASE_URL_TESTNET_API,
        (Product::Spot, Env::Demo) => spot::BASE_URL_DEMO_API,
        (Product::UsdmFutures, Env::Production) => derivatives::usds_margined_futures::BASE_URL_API,
        (Product::UsdmFutures, Env::Testnet) => {
            derivatives::usds_margined_futures::BASE_URL_TESTNET_API
        }
        (Product::UsdmFutures, Env::Demo) => derivatives::usds_margined_futures::BASE_URL_DEMO_API,
        (Product::CoinmFutures, Env::Production) => {
            derivatives::coin_margined_futures::BASE_URL_API
        }
        (Product::CoinmFutures, Env::Testnet) => {
            derivatives::coin_margined_futures::BASE_URL_TESTNET_API
        }
        (Product::Margin, Env::Production) => binance::margin::BASE_URL_API,
        (Product::Wallet, Env::Production) => binance::wallet::BASE_URL_API,
        _ => unsupported(product, env)?,
    };
    Ok(url)
}

/// WebSocket stream base URL of `product` in the selected environment.
pub fn stream(product: Product) -> anyhow::Result<&'static str> {
    let env = env()?;
    let url = match (product, env) {
        (Product::Spot, Env::Production) => spot::BASE_URL_STREAM3,
        (Product::Spot, Env::Testnet) => spot::BASE_URL_TESTNET_STREAM3,
        (Product::Spot, Env::Demo) => spot::BASE_URL_DEMO_STREAM2,
        (Product::UsdmFutures, Env::Production) => {
            derivatives::usds_margined_futures::BASE_URL_STREAM
        }
        (Product::UsdmFutures, Env::Testnet) => {
            derivatives::usds_margined_futures::BASE_URL_TESTNET_STREAM
        }
        (Product::UsdmFutures, Env::Demo) => {
            derivatives::usds_margined_futures::BASE_URL_DEMO_STREAM
        }
        (Product::CoinmFutures, Env::Production) => {
            derivatives::coin_margined_futures::BASE_URL_STREAM
        }
        (Product::CoinmFutures, Env::Testnet) => {
            derivatives::coin_margined_futures::BASE_URL_TESTNET_STREAM
        }
        (Product::Margin, Env::Production) => binance::margin::BASE_URL_STREAM,
        _ => unsupported(product, env)?,
    };
    Ok(url)
}

/// WebSocket API base URL (without the `/ws-api/v3` path) of `product`.
pub fn ws_api(product: Product) -> anyhow::Result<&'static str> {
    let env = env()?;
    let url = match (product, env) {
        (Product::Spot, Env::Production) => spot::BASE_URL_WEBSOCKET_API3,
        (Product::Spot, Env::Testnet) => spot::BASE_URL_TESTNET_WEBSOCKET_API1,
        (Product::Spot, Env::Demo) => spot::BASE_URL_DEMO_WEBSOCKET_API1,
        _ => unsupported(product, env)?,
    };
    Ok(url)
}

fn unsupported(product: Product, env: Env) -> anyhow::Result<&'static str> {
    bail!("{product:?} is not available in the {env:?} environment (BINANCE_ENV)")
}

/// `API_KEY` / `API_SECRET` from the process environment.
pub fn credentials() -> anyhow::Result<(SensitiveString, SensitiveString)> {
    let key = std::env::var("API_KEY").context("environment variable API_KEY is required")?;
    let secret =
        std::env::var("API_SECRET").context("environment variable API_SECRET is required")?;
    Ok((SensitiveString::from(key), SensitiveString::from(secret)))
}
