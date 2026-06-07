use reqwest::header::HeaderMap;

use crate::SensitiveString;

#[derive(Debug, Clone)]
pub struct PublicConfig {
    pub base_url: String,
    pub headers: Option<HeaderMap>,
}

impl PublicConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
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
