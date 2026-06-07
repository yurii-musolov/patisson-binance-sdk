use reqwest::header::HeaderMap;

use crate::SensitiveString;

/// Configuration for [`super::PrivateClient`].
///
/// Margin has no public endpoints (everything under `/sapi/v1/margin/*`
/// requires an API key + signature). For unauthenticated market data and
/// connectivity, use [`crate::spot::http::PublicClient`].
#[derive(Debug, Clone)]
pub struct PrivateConfig {
    pub base_url: String,
    pub api_key: SensitiveString,
    pub api_secret: SensitiveString,
    pub headers: Option<HeaderMap>,
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
}
