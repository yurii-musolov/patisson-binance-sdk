use std::{sync::Arc, time::Duration};

use reqwest::header::HeaderMap;

use crate::{SensitiveString, http::Timeouts, rate_limit::RateLimiter};

/// Configuration for [`super::PrivateClient`].
///
/// Wallet has no public endpoints (everything under `/sapi/v1/...` requires
/// an API key + signature). For unauthenticated market data and connectivity,
/// use [`crate::spot::http::PublicClient`].
#[derive(Debug, Clone)]
pub struct PrivateConfig {
    pub base_url: String,
    pub api_key: SensitiveString,
    pub api_secret: SensitiveString,
    pub headers: Option<HeaderMap>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
    pub timeouts: Timeouts,
}

impl PrivateConfig {
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

    /// Attach an optional client-side rate limiter. Wallet endpoints all
    /// share the spot REQUEST_WEIGHT bucket per IP, so you typically want to
    /// share one `Arc<RateLimiter>` across `spot::http::*Client` and this client.
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
}
