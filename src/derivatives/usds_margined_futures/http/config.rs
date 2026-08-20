use std::sync::Arc;

use reqwest::header::HeaderMap;

use crate::{SensitiveString, rate_limit::RateLimiter};

#[derive(Debug, Clone)]
pub struct PublicConfig {
    pub base_url: String,
    pub headers: Option<HeaderMap>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
}

impl PublicConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            headers: None,
            rate_limiter: None,
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

    /// Attach an optional client-side rate limiter. USDⓈ-M Futures has its
    /// own per-IP REQUEST_WEIGHT bucket on `/fapi/*` (separate from spot), so
    /// you typically want one `Arc<RateLimiter>` shared between this client's
    /// public and private halves but a different one from spot/margin/wallet.
    pub fn rate_limiter(mut self, rate_limiter: Arc<RateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
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
}
