//! Internal HTTP layer shared by every product (spot, margin, wallet, USDⓈ-M,
//! COIN-M). Centralises the `reqwest::Client`, response-header parsing
//! (including `X-MBX-USED-WEIGHT-*` / `X-MBX-ORDER-COUNT-*`), and the optional
//! [`RateLimiter`] hook.
//!
//! Product clients build a [`HttpClient`] in their constructor and use
//! [`HttpClient::request`] + [`HttpClient::send_raw`] to issue every call.
//! `send_raw` returns a [`RawResponse`] on every non-rate-limit status (incl.
//! 4xx Binance error bodies) so each product can decode its own
//! `ApiError` shape; only transport failure and 429/418 short-circuit through
//! [`SendError`].

use std::{
    collections::BTreeMap,
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::{Method, RequestBuilder, StatusCode, header::HeaderMap};

use crate::rate_limit::{Cost, ObservedUsage, RateLimitSource, RateLimited, RateLimiter};

const HEADER_RETRY_AFTER: &str = "retry-after";
const HEADER_USED_WEIGHT_PREFIX: &str = "x-mbx-used-weight-";
const HEADER_ORDER_COUNT_PREFIX: &str = "x-mbx-order-count-";

/// Parsed rate-limit-related response headers.
///
/// `retry_after` is set on 429/418 responses (`Retry-After` header, delta
/// seconds). The two `BTreeMap`s carry the latest authoritative usage Binance
/// reports per bucket interval, e.g. a `1m` window keys to `Duration::from_secs(60)`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParsedHeaders {
    pub retry_after: Option<Duration>,
    pub used_weight: BTreeMap<Duration, u32>,
    pub order_count: BTreeMap<Duration, u32>,
}

impl ParsedHeaders {
    fn to_observed(&self) -> ObservedUsage {
        ObservedUsage {
            weight: self.used_weight.clone(),
            orders: self.order_count.clone(),
        }
    }
}

/// What [`HttpClient::send_raw`] returns on a non-rate-limit status. Includes
/// 4xx Binance error bodies so the per-product `send` can decode an `ApiError`.
#[derive(Debug)]
pub struct RawResponse {
    pub status: StatusCode,
    pub headers: ParsedHeaders,
    pub body: String,
}

/// Errors surfaced by [`HttpClient::send_raw`] that the per-product `send`
/// can't usefully handle itself.
#[derive(Debug)]
pub enum SendError {
    Reqwest(reqwest::Error),
    RateLimited {
        retry_after: Duration,
        source: RateLimitSource,
    },
}

impl From<reqwest::Error> for SendError {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<RateLimited> for SendError {
    fn from(r: RateLimited) -> Self {
        Self::RateLimited {
            retry_after: r.retry_after,
            source: r.source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: String,
    headers: HeaderMap,
    rate_limiter: Option<Arc<RateLimiter>>,
}

impl HttpClient {
    pub fn new(
        base_url: String,
        headers: HeaderMap,
        rate_limiter: Option<Arc<RateLimiter>>,
    ) -> Result<Self, reqwest::Error> {
        let inner = reqwest::Client::builder().build()?;
        Ok(Self {
            inner,
            base_url,
            headers,
            rate_limiter,
        })
    }

    /// Start building a request rooted at `<base_url><path_suffix>` with the
    /// configured default headers already applied. `path_suffix` can be a
    /// `&str` or any product's `Path` enum (both implement `Display`).
    pub fn request(&self, method: Method, path_suffix: impl Display) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path_suffix);
        self.inner
            .request(method, url)
            .headers(self.headers.clone())
    }

    /// Charge `cost` against the rate limiter (if any), then send. Parses
    /// rate-limit headers, observes them back into the limiter, and treats
    /// 429/418 as a rate-limit error (freezing the limiter until `Retry-After`
    /// elapses).
    pub async fn send_raw(
        &self,
        request: RequestBuilder,
        cost: Cost,
    ) -> Result<RawResponse, SendError> {
        if let Some(rl) = &self.rate_limiter {
            rl.try_acquire(cost)?;
        }

        let response = request.send().await?;
        let status = response.status();
        let headers = parse_headers(response.headers());
        if let Some(rl) = &self.rate_limiter {
            rl.observe(&headers.to_observed());
        }
        let body = response.text().await?;

        if status == StatusCode::TOO_MANY_REQUESTS || status.as_u16() == 418 {
            // Binance: 429 = soft throttle, 418 = IP banned for repeated 429s.
            // Treat both as "back off until Retry-After" and freeze any shared
            // limiter so concurrent in-flight calls also see the brake.
            let retry_after = headers
                .retry_after
                .unwrap_or_else(|| Duration::from_secs(60));
            if let Some(rl) = &self.rate_limiter {
                rl.freeze_until(Instant::now() + retry_after);
            }
            return Err(SendError::RateLimited {
                retry_after,
                source: RateLimitSource::Server,
            });
        }

        Ok(RawResponse {
            status,
            headers,
            body,
        })
    }
}

fn parse_headers(headers: &HeaderMap) -> ParsedHeaders {
    let mut parsed = ParsedHeaders::default();
    for (name, value) in headers.iter() {
        let name = name.as_str();
        let value = match value.to_str() {
            Ok(v) => v,
            Err(_) => continue,
        };

        if name.eq_ignore_ascii_case(HEADER_RETRY_AFTER) {
            if let Ok(seconds) = value.trim().parse::<u64>() {
                parsed.retry_after = Some(Duration::from_secs(seconds));
            }
            continue;
        }

        if let Some(suffix) = name.strip_prefix(HEADER_USED_WEIGHT_PREFIX)
            && let Some((interval, count)) = parse_interval_and_count(suffix, value)
        {
            parsed.used_weight.insert(interval, count);
        } else if let Some(suffix) = name.strip_prefix(HEADER_ORDER_COUNT_PREFIX)
            && let Some((interval, count)) = parse_interval_and_count(suffix, value)
        {
            parsed.order_count.insert(interval, count);
        }
    }
    parsed
}

/// Parse the suffix of `x-mbx-used-weight-(NUM)(UNIT)` style headers — e.g.
/// `"1m"` → `Duration::from_secs(60)` — together with the header value.
fn parse_interval_and_count(suffix: &str, value: &str) -> Option<(Duration, u32)> {
    let (digits, unit) = suffix
        .trim()
        .find(|c: char| c.is_ascii_alphabetic())
        .map(|i| suffix.split_at(i))?;
    let num: u64 = digits.parse().ok()?;
    let unit_secs: u64 = match unit.to_ascii_lowercase().as_str() {
        "s" => 1,
        "m" => 60,
        "h" => 3_600,
        "d" => 86_400,
        _ => return None,
    };
    let interval = Duration::from_secs(num.checked_mul(unit_secs)?);
    let count: u32 = value.trim().parse().ok()?;
    Some((interval, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    fn header_map(entries: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in entries {
            let name = reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap();
            h.append(name, HeaderValue::from_str(v).unwrap());
        }
        h
    }

    #[test]
    fn parses_retry_after_seconds() {
        let h = parse_headers(&header_map(&[("retry-after", "120")]));
        assert_eq!(h.retry_after, Some(Duration::from_secs(120)));
    }

    #[test]
    fn parses_used_weight_intervals() {
        let h = parse_headers(&header_map(&[
            ("x-mbx-used-weight-1m", "240"),
            ("x-mbx-used-weight-10s", "12"),
        ]));
        assert_eq!(h.used_weight.get(&Duration::from_secs(60)), Some(&240));
        assert_eq!(h.used_weight.get(&Duration::from_secs(10)), Some(&12));
    }

    #[test]
    fn parses_order_count_intervals() {
        let h = parse_headers(&header_map(&[
            ("x-mbx-order-count-10s", "3"),
            ("x-mbx-order-count-1d", "1500"),
        ]));
        assert_eq!(h.order_count.get(&Duration::from_secs(10)), Some(&3));
        assert_eq!(h.order_count.get(&Duration::from_secs(86_400)), Some(&1500));
    }

    #[test]
    fn unknown_unit_is_ignored() {
        let h = parse_headers(&header_map(&[("x-mbx-used-weight-1y", "1")]));
        assert!(h.used_weight.is_empty());
    }

    #[test]
    fn case_insensitive_header_names() {
        // reqwest stores names lowercase internally so case insensitivity is
        // really about the prefix match; this protects against future changes.
        let h = parse_headers(&header_map(&[("Retry-After", "5")]));
        assert_eq!(h.retry_after, Some(Duration::from_secs(5)));
    }
}
