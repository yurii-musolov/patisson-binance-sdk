use std::{sync::Arc, time::Duration};

use reqwest::header::HeaderMap;

use crate::{
    Environment, Product, SensitiveString, TimeOffset, Unsupported, http::Timeouts,
    rate_limit::RateLimiter,
};

#[derive(Debug, Clone)]
pub struct PublicConfig {
    pub base_url: String,
    pub headers: Option<HeaderMap>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
    pub timeouts: Timeouts,
    /// Proxy for REST requests (`http://`, `https://` or `socks5://`, with
    /// optional `user:password@`); `None` uses `HTTP(S)_PROXY` / `ALL_PROXY`.
    /// Kept as a [`SensitiveString`] since it may carry credentials.
    pub proxy: Option<SensitiveString>,
}

impl PublicConfig {
    /// Config for the REST API of `env`.
    pub fn for_env(env: Environment) -> Result<Self, Unsupported> {
        Ok(Self::new(env.api_url(Product::Spot)?))
    }

    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            headers: None,
            rate_limiter: None,
            timeouts: Timeouts::default(),
            proxy: None,
        }
    }

    pub fn headers(mut self, headers: Option<HeaderMap>) -> Self {
        if let Some(headers) = headers {
            self.headers
                .get_or_insert_with(HeaderMap::new)
                .extend(headers);
        }

        self
    }

    /// Attach an optional client-side rate limiter. The same `Arc<RateLimiter>`
    /// can be shared across `PublicClient` and `PrivateClient` so all calls
    /// charge against one budget.
    pub fn rate_limiter(mut self, rate_limiter: Arc<RateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

    /// Total time budget for a single request (connect + send + read the
    /// whole response). Defaults to [`crate::DEFAULT_HTTP_TIMEOUT`].
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeouts.request = timeout;
        self
    }

    /// Time budget for establishing the TCP/TLS connection. Defaults to
    /// [`crate::DEFAULT_HTTP_CONNECT_TIMEOUT`].
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.timeouts.connect = timeout;
        self
    }

    /// Send REST requests through a proxy, e.g. `http://127.0.0.1:8080` or
    /// `socks5://user:password@proxy:1080`.
    pub fn proxy(mut self, url: impl Into<SensitiveString>) -> Self {
        self.proxy = Some(url.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct PrivateConfig {
    pub base_url: String,
    pub api_key: SensitiveString,
    pub api_secret: SensitiveString,
    pub headers: Option<HeaderMap>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
    pub timeouts: Timeouts,
    /// Proxy for REST requests (`http://`, `https://` or `socks5://`, with
    /// optional `user:password@`); `None` uses `HTTP(S)_PROXY` / `ALL_PROXY`.
    /// Kept as a [`SensitiveString`] since it may carry credentials.
    pub proxy: Option<SensitiveString>,
    /// Clock correction applied to signed requests; see [`TimeOffset`].
    pub time_offset: TimeOffset,
}

impl PrivateConfig {
    /// Config for the REST API of `env`. API keys are environment-specific.
    pub fn for_env(
        env: Environment,
        api_key: SensitiveString,
        api_secret: SensitiveString,
    ) -> Result<Self, Unsupported> {
        Ok(Self::new(env.api_url(Product::Spot)?, api_key, api_secret))
    }

    pub fn new(
        base_url: impl Into<String>,
        api_key: SensitiveString,
        api_secret: SensitiveString,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            api_key,
            api_secret,
            headers: None,
            rate_limiter: None,
            timeouts: Timeouts::default(),
            proxy: None,
            time_offset: TimeOffset::default(),
        }
    }

    pub fn headers(mut self, headers: Option<HeaderMap>) -> Self {
        if let Some(headers) = headers {
            self.headers
                .get_or_insert_with(HeaderMap::new)
                .extend(headers);
        }

        self
    }

    pub fn rate_limiter(mut self, rate_limiter: Arc<RateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

    /// Total time budget for a single request (connect + send + read the
    /// whole response). Defaults to [`crate::DEFAULT_HTTP_TIMEOUT`].
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeouts.request = timeout;
        self
    }

    /// Time budget for establishing the TCP/TLS connection. Defaults to
    /// [`crate::DEFAULT_HTTP_CONNECT_TIMEOUT`].
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.timeouts.connect = timeout;
        self
    }

    /// Send REST requests through a proxy, e.g. `http://127.0.0.1:8080` or
    /// `socks5://user:password@proxy:1080`.
    pub fn proxy(mut self, url: impl Into<SensitiveString>) -> Self {
        self.proxy = Some(url.into());
        self
    }

    /// Correct the timestamp of signed requests for local clock drift. Share
    /// the same [`TimeOffset`] across clients and refresh it periodically.
    pub fn time_offset(mut self, time_offset: TimeOffset) -> Self {
        self.time_offset = time_offset;
        self
    }
}
