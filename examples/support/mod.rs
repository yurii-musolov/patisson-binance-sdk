//! Shared helpers for the examples: pick the Binance environment and read
//! API credentials.
//!
//! The environment comes from `BINANCE_ENV`:
//!
//! | `BINANCE_ENV`            | [`Environment`] |
//! |--------------------------|-----------------|
//! | unset, `prod`, `mainnet` | `Production`    |
//! | `testnet`                | `Testnet`       |
//! | `demo`                   | `Demo`          |
//!
//! Base URLs come from [`Environment`]; a product an environment doesn't
//! offer (margin and wallet outside production) is an error, not a silent
//! fallback to production.
//!
//! Credentials come from `API_KEY` / `API_SECRET` and are wrapped in
//! `SensitiveString` right away, so they never show up in logs.
#![allow(dead_code)] // each example uses only part of this module

use anyhow::{Context, bail};
pub use binance::Product;
use binance::{Environment, SensitiveString};

/// The environment selected by `BINANCE_ENV` (production when unset).
pub fn env() -> anyhow::Result<Environment> {
    let value = match std::env::var("BINANCE_ENV") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(Environment::Production),
        Err(e) => return Err(e).context("BINANCE_ENV"),
    };
    match value.as_str() {
        "" | "prod" | "production" | "mainnet" => Ok(Environment::Production),
        "testnet" => Ok(Environment::Testnet),
        "demo" => Ok(Environment::Demo),
        other => bail!("unknown BINANCE_ENV={other:?}; use prod, testnet or demo"),
    }
}

/// REST base URL of `product` in the selected environment.
pub fn api(product: Product) -> anyhow::Result<&'static str> {
    Ok(env()?.api_url(product)?)
}

/// WebSocket stream base URL of `product` in the selected environment.
pub fn stream(product: Product) -> anyhow::Result<&'static str> {
    Ok(env()?.stream_url(product)?)
}

/// WebSocket API base URL of `product` in the selected environment.
pub fn ws_api(product: Product) -> anyhow::Result<&'static str> {
    Ok(env()?.ws_api_url(product)?)
}

/// `API_KEY` / `API_SECRET` from the process environment.
pub fn credentials() -> anyhow::Result<(SensitiveString, SensitiveString)> {
    let key = std::env::var("API_KEY").context("environment variable API_KEY is required")?;
    let secret =
        std::env::var("API_SECRET").context("environment variable API_SECRET is required")?;
    Ok((SensitiveString::from(key), SensitiveString::from(secret)))
}
