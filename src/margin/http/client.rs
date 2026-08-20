use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::sign_query,
    http::{HttpClient, RawResponse, SendError},
    margin::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            EmptyResponse, GetAllMarginAssetsParams, GetMarginAccountParams,
            GetMaxBorrowableParams, ListenKey, MarginAccount, MarginAsset, MaxBorrowable,
            NewOrderRequest, NewOrderResponse, Order, PrivateConfig, QueryOrderParams, Response,
        },
    },
    rate_limit::Cost,
    serde::{deserialize_json, serialize_query},
    timestamp,
};

// Per-endpoint weights (Binance Margin REST docs). These charge against the
// shared spot REQUEST_WEIGHT bucket per IP; share the same `Arc<RateLimiter>`
// with `spot::http::*Client` if you want correct accounting.
const COST_ALL_ASSETS: Cost = Cost::weight(1);
const COST_MARGIN_ACCOUNT: Cost = Cost::weight(10);
const COST_NEW_ORDER: Cost = Cost::weight_and_orders(6, 1);
const COST_QUERY_ORDER: Cost = Cost::weight(10);
const COST_MAX_BORROWABLE: Cost = Cost::weight(50);
const COST_LISTEN_KEY: Cost = Cost::weight(1);

/// Client for the authenticated `/sapi/v1/margin/*` surface.
///
/// Margin has no public endpoints — for unauthenticated market data
/// (klines, depth, tickers) and connectivity (`/api/v3/ping`, `/api/v3/time`),
/// use [`crate::spot::http::PublicClient`].
pub struct PrivateClient {
    http: HttpClient,
    api_secret: SensitiveString,
}

impl PrivateClient {
    pub fn new(cfg: PrivateConfig) -> Self {
        let headers = build_private_headers(&cfg);
        let http = HttpClient::new(cfg.base_url, headers, cfg.rate_limiter)
            .expect("reqwest client builder failed");
        Self {
            http,
            api_secret: cfg.api_secret,
        }
    }
}

fn build_private_headers(cfg: &PrivateConfig) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let api_key = cfg.api_key.expose().parse().unwrap();
    headers.append(HEADER_X_MBX_APIKEY, api_key);
    if let Some(extra) = &cfg.headers {
        headers.extend(extra.clone());
    }
    headers
}

// Margin metadata
impl PrivateClient {
    /// Get all margin assets supported by the exchange.
    pub async fn get_all_assets(
        &self,
        params: GetAllMarginAssetsParams,
    ) -> Result<Response<Vec<MarginAsset>>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::AllAssets));
        decode(self.http.send_raw(req, COST_ALL_ASSETS).await)
    }
}

// Cross margin account
impl PrivateClient {
    /// Get the caller's cross-margin account snapshot (balances, level, etc.).
    pub async fn margin_account(
        &self,
        params: GetMarginAccountParams,
    ) -> Result<Response<MarginAccount>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::Account));
        decode(self.http.send_raw(req, COST_MARGIN_ACCOUNT).await)
    }
}

// Margin trading
impl PrivateClient {
    /// Place a new margin order.
    ///
    /// Set `is_isolated = IsIsolated::True` to route the order to the isolated
    /// margin account for the symbol; otherwise the cross-margin account is used.
    /// Combine with [`SideEffectType::MarginBuy`] / [`SideEffectType::AutoRepay`]
    /// to opt into automatic borrowing or repayment when the order fills.
    pub async fn new_order(
        &self,
        params: NewOrderRequest,
    ) -> Result<Response<NewOrderResponse>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::POST, format!("{}?{query}", Path::Order));
        decode(self.http.send_raw(req, COST_NEW_ORDER).await)
    }

    /// Look up a single margin order by `order_id` or `orig_client_order_id`.
    pub async fn query_order(&self, params: QueryOrderParams) -> Result<Response<Order>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::Order));
        decode(self.http.send_raw(req, COST_QUERY_ORDER).await)
    }
}

// Margin borrow / repay
impl PrivateClient {
    /// Query the maximum borrowable amount for an asset.
    ///
    /// Pass `isolated_symbol` to ask about the isolated account for a specific
    /// symbol; without it the call reports the cross-margin limit.
    pub async fn max_borrowable(
        &self,
        params: GetMaxBorrowableParams,
    ) -> Result<Response<MaxBorrowable>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::MaxBorrowable));
        decode(self.http.send_raw(req, COST_MAX_BORROWABLE).await)
    }
}

// User data stream — cross margin.
//
// Unlike the trading endpoints, listenKey operations are authenticated by
// API key alone (`X-MBX-APIKEY` header). They do NOT take `timestamp` /
// `signature`, so these methods skip `sign_query` entirely.
impl PrivateClient {
    /// Create a new listenKey for the cross-margin user data stream.
    ///
    /// Returns a key that can be used to connect to
    /// `wss://stream.binance.com:9443/ws/<listenKey>`. The key expires after
    /// 60 minutes — extend via [`Self::keepalive_listen_key`] every 30 min.
    pub async fn create_listen_key(&self) -> Result<Response<ListenKey>, Error> {
        let req = self.http.request(Method::POST, Path::UserDataStream);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Extend a cross-margin listenKey's lifetime by 60 minutes. Idempotent;
    /// safe to call on a schedule (recommended every 30 min).
    pub async fn keepalive_listen_key(
        &self,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let req = self
            .http
            .request(Method::PUT, Path::UserDataStream)
            .query(&[("listenKey", listen_key)]);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Close a cross-margin listenKey. The WebSocket connection associated
    /// with the key will be dropped by the server.
    pub async fn close_listen_key(
        &self,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let req = self
            .http
            .request(Method::DELETE, Path::UserDataStream)
            .query(&[("listenKey", listen_key)]);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }
}

// User data stream — isolated margin. Same lifecycle as cross-margin but
// every call carries the isolated-account `symbol`.
impl PrivateClient {
    /// Create a new listenKey for an isolated-margin account's user data
    /// stream. Each isolated account has its own key.
    pub async fn create_isolated_listen_key(
        &self,
        symbol: &str,
    ) -> Result<Response<ListenKey>, Error> {
        let req = self
            .http
            .request(Method::POST, Path::UserDataStreamIsolated)
            .query(&[("symbol", symbol)]);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Extend an isolated-margin listenKey's lifetime by 60 minutes.
    pub async fn keepalive_isolated_listen_key(
        &self,
        symbol: &str,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let req = self
            .http
            .request(Method::PUT, Path::UserDataStreamIsolated)
            .query(&[("symbol", symbol), ("listenKey", listen_key)]);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Close an isolated-margin listenKey.
    pub async fn close_isolated_listen_key(
        &self,
        symbol: &str,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let req = self
            .http
            .request(Method::DELETE, Path::UserDataStreamIsolated)
            .query(&[("symbol", symbol), ("listenKey", listen_key)]);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }
}

fn decode<T>(raw: Result<RawResponse, SendError>) -> Result<Response<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let raw = raw?;
    if !raw.status.is_success() {
        #[cfg(debug_assertions)]
        tracing::debug!(status = ?raw.status, body = ?raw.body, "request failed");

        let api_err = deserialize_json::<ApiError>(&raw.body)?;
        return Err(Error::Api(api_err));
    }
    let result = deserialize_json(&raw.body)?;
    Ok(Response {
        result,
        headers: raw.headers,
    })
}
