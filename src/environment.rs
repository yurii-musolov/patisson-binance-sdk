//! Binance deployments and their base URLs, per product.

use std::fmt;

use crate::{derivatives, margin, spot, wallet};

/// A Binance product family; each has its own hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Product {
    Spot,
    Margin,
    Wallet,
    UsdmFutures,
    CoinmFutures,
}

/// Where to send REST requests and open WebSocket connections.
///
/// The URLs are base URLs without a path: append the product's `Path` (e.g.
/// `spot::Path::Stream`, or `StreamName::path()` for USD-M streams).
///
/// | | Spot | Margin, Wallet | USD-M, COIN-M |
/// |---|---|---|---|
/// | `Production` | yes | yes | yes |
/// | `Testnet` | `testnet.binance.vision` | - | `demo-fapi` / `demo-dapi` (the documented futures testnet) |
/// | `Demo` | `demo-api.binance.com` | - | same as `Testnet` |
///
/// Combinations Binance doesn't offer return [`Unsupported`] instead of
/// silently pointing at production. Every API key works in one environment
/// only.
///
/// ```
/// use binance::{Environment, Product};
///
/// let env = Environment::Demo;
/// assert_eq!(env.api_url(Product::Spot).unwrap(), "https://demo-api.binance.com");
/// assert_eq!(
///     env.stream_url(Product::UsdmFutures).unwrap(),
///     "wss://demo-fstream.binance.com"
/// );
/// assert!(env.api_url(Product::Margin).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Environment {
    /// `api.binance.com`, `fapi.binance.com`, `dapi.binance.com`.
    Production,
    /// Spot testnet (`testnet.binance.vision`) and the futures testnet
    /// (`demo-fapi.binance.com`, `demo-dapi.binance.com`).
    Testnet,
    /// Demo trading (https://demo.binance.com).
    Demo,
    /// Any other deployment; the same URLs are used for every product.
    Custom {
        /// REST base URL, e.g. `https://api.binance.com`.
        api: &'static str,
        /// WebSocket stream base URL, e.g. `wss://stream.binance.com:9443`.
        stream: &'static str,
        /// WebSocket API base URL, e.g. `wss://ws-api.binance.com`.
        ws_api: &'static str,
    },
}

/// `product` is not offered in `environment`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsupported {
    pub product: Product,
    pub environment: Environment,
    pub endpoint: &'static str,
}

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} has no {} endpoint in the {:?} environment",
            self.product, self.endpoint, self.environment
        )
    }
}

impl std::error::Error for Unsupported {}

impl Environment {
    /// REST base URL of `product`.
    pub fn api_url(&self, product: Product) -> Result<&'static str, Unsupported> {
        use derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm};
        let url = match (self, product) {
            (Self::Custom { api, .. }, _) => api,
            (Self::Production, Product::Spot) => spot::BASE_URL_API,
            (Self::Production, Product::Margin) => margin::BASE_URL_API,
            (Self::Production, Product::Wallet) => wallet::BASE_URL_API,
            (Self::Production, Product::UsdmFutures) => usdm::BASE_URL_API,
            (Self::Production, Product::CoinmFutures) => coinm::BASE_URL_API,
            (Self::Testnet, Product::Spot) => spot::BASE_URL_TESTNET_API,
            (Self::Demo, Product::Spot) => spot::BASE_URL_DEMO_API,
            (Self::Testnet | Self::Demo, Product::UsdmFutures) => usdm::BASE_URL_DEMO_API,
            (Self::Testnet | Self::Demo, Product::CoinmFutures) => coinm::BASE_URL_DEMO_API,
            _ => return Err(self.unsupported(product, "REST")),
        };
        Ok(url)
    }

    /// WebSocket market/user data stream base URL of `product`.
    pub fn stream_url(&self, product: Product) -> Result<&'static str, Unsupported> {
        use derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm};
        let url = match (self, product) {
            (Self::Custom { stream, .. }, _) => stream,
            (Self::Production, Product::Spot) => spot::BASE_URL_STREAM3,
            (Self::Production, Product::Margin) => margin::BASE_URL_STREAM,
            (Self::Production, Product::UsdmFutures) => usdm::BASE_URL_STREAM,
            (Self::Production, Product::CoinmFutures) => coinm::BASE_URL_STREAM,
            (Self::Testnet, Product::Spot) => spot::BASE_URL_TESTNET_STREAM3,
            (Self::Demo, Product::Spot) => spot::BASE_URL_DEMO_STREAM2,
            (Self::Testnet | Self::Demo, Product::UsdmFutures) => usdm::BASE_URL_DEMO_STREAM,
            (Self::Testnet | Self::Demo, Product::CoinmFutures) => coinm::BASE_URL_DEMO_STREAM,
            _ => return Err(self.unsupported(product, "stream")),
        };
        Ok(url)
    }

    /// WebSocket API base URL of `product` (append `Path::WebSocketApiV3`,
    /// or `Path::WebSocketApi` for futures).
    pub fn ws_api_url(&self, product: Product) -> Result<&'static str, Unsupported> {
        use derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm};
        let url = match (self, product) {
            (Self::Custom { ws_api, .. }, _) => ws_api,
            (Self::Production, Product::Spot) => spot::BASE_URL_WEBSOCKET_API1,
            (Self::Production, Product::UsdmFutures) => usdm::BASE_URL_WEBSOCKET_API,
            (Self::Production, Product::CoinmFutures) => coinm::BASE_URL_WEBSOCKET_API,
            (Self::Testnet, Product::Spot) => spot::BASE_URL_TESTNET_WEBSOCKET_API1,
            (Self::Demo, Product::Spot) => spot::BASE_URL_DEMO_WEBSOCKET_API1,
            (Self::Testnet | Self::Demo, Product::UsdmFutures) => {
                usdm::BASE_URL_TESTNET_WEBSOCKET_API
            }
            (Self::Testnet | Self::Demo, Product::CoinmFutures) => {
                coinm::BASE_URL_TESTNET_WEBSOCKET_API
            }
            _ => return Err(self.unsupported(product, "WebSocket API")),
        };
        Ok(url)
    }

    fn unsupported(&self, product: Product, endpoint: &'static str) -> Unsupported {
        Unsupported {
            product,
            environment: *self,
            endpoint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Product; 5] = [
        Product::Spot,
        Product::Margin,
        Product::Wallet,
        Product::UsdmFutures,
        Product::CoinmFutures,
    ];

    #[test]
    fn production_serves_every_product_over_rest() {
        for product in ALL {
            assert!(
                Environment::Production.api_url(product).is_ok(),
                "{product:?}"
            );
        }
        assert!(Environment::Production.stream_url(Product::Wallet).is_err());
    }

    #[test]
    fn documented_hosts() {
        let cases = [
            (
                Environment::Testnet,
                Product::Spot,
                "https://testnet.binance.vision",
            ),
            (
                Environment::Demo,
                Product::Spot,
                "https://demo-api.binance.com",
            ),
            (
                Environment::Testnet,
                Product::UsdmFutures,
                "https://demo-fapi.binance.com",
            ),
            (
                Environment::Demo,
                Product::CoinmFutures,
                "https://demo-dapi.binance.com",
            ),
        ];
        for (env, product, url) in cases {
            assert_eq!(env.api_url(product).unwrap(), url, "{env:?} {product:?}");
        }
        assert_eq!(
            Environment::Testnet.stream_url(Product::Spot).unwrap(),
            "wss://stream.testnet.binance.vision:9443"
        );
        assert_eq!(
            Environment::Demo.stream_url(Product::CoinmFutures).unwrap(),
            "wss://demo-dstream.binance.com"
        );
        assert_eq!(
            Environment::Testnet
                .ws_api_url(Product::CoinmFutures)
                .unwrap(),
            "wss://testnet.binancefuture.com"
        );
    }

    #[test]
    fn margin_and_wallet_are_production_only() {
        for env in [Environment::Testnet, Environment::Demo] {
            for product in [Product::Margin, Product::Wallet] {
                let err = env.api_url(product).unwrap_err();
                assert_eq!(err.product, product);
                assert!(err.to_string().contains("REST"));
            }
        }
    }

    #[test]
    fn custom_applies_to_every_product() {
        let env = Environment::Custom {
            api: "https://example.com",
            stream: "wss://example.com/s",
            ws_api: "wss://example.com/a",
        };
        for product in ALL {
            assert_eq!(env.api_url(product).unwrap(), "https://example.com");
            assert_eq!(env.stream_url(product).unwrap(), "wss://example.com/s");
            assert_eq!(env.ws_api_url(product).unwrap(), "wss://example.com/a");
        }
    }
}
