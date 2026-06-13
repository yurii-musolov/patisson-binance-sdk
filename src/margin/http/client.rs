use reqwest::{self, Method, RequestBuilder, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::sign_query,
    margin::{
        ApiError, Error, HEADER_RETRY_AFTER, HEADER_X_MBX_APIKEY, Path,
        http::{
            EmptyResponse, GetAllMarginAssetsParams, GetMarginAccountParams,
            GetMaxBorrowableParams, Headers, ListenKey, MarginAccount, MarginAsset, MaxBorrowable,
            NewOrderRequest, NewOrderResponse, Order, PrivateConfig, QueryOrderParams, Response,
        },
    },
    serde::{deserialize_json, serialize_query},
    timestamp,
};

/// Client for the authenticated `/sapi/v1/margin/*` surface.
///
/// Margin has no public endpoints — for unauthenticated market data
/// (klines, depth, tickers, exchange info) and connectivity (`/api/v3/ping`,
/// `/api/v3/time`), use [`crate::spot::http::PublicClient`].
pub struct PrivateClient {
    base_url: String,
    headers: HeaderMap,
    api_secret: SensitiveString,
}

impl PrivateClient {
    pub fn new(cfg: PrivateConfig) -> Self {
        let headers = {
            let mut headers = HeaderMap::new();
            let api_key = cfg.api_key.expose().parse().unwrap();
            headers.append(HEADER_X_MBX_APIKEY, api_key);
            if let Some(extra) = cfg.headers {
                headers.extend(extra);
            }
            headers
        };
        Self {
            base_url: cfg.base_url,
            headers,
            api_secret: cfg.api_secret,
        }
    }
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
        let url = format!("{}{}?{query}", self.base_url, Path::AllAssets);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
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
        let url = format!("{}{}?{query}", self.base_url, Path::Account);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
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
        let url = format!("{}{}", self.base_url, Path::Order);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::POST, url)
            .headers(self.headers.clone())
            .body(query);
        send(request).await
    }

    /// Look up a single margin order by `order_id` or `orig_client_order_id`.
    pub async fn query_order(&self, params: QueryOrderParams) -> Result<Response<Order>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let url = format!("{}{}?{query}", self.base_url, Path::Order);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
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
        let url = format!("{}{}?{query}", self.base_url, Path::MaxBorrowable);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
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
        let url = format!("{}{}", self.base_url, Path::UserDataStream);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::POST, url)
            .headers(self.headers.clone());
        send(request).await
    }

    /// Extend a cross-margin listenKey's lifetime by 60 minutes. Idempotent;
    /// safe to call on a schedule (recommended every 30 min).
    pub async fn keepalive_listen_key(
        &self,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let url = format!("{}{}", self.base_url, Path::UserDataStream);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::PUT, url)
            .headers(self.headers.clone())
            .query(&[("listenKey", listen_key)]);
        send(request).await
    }

    /// Close a cross-margin listenKey. The WebSocket connection associated
    /// with the key will be dropped by the server.
    pub async fn close_listen_key(
        &self,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let url = format!("{}{}", self.base_url, Path::UserDataStream);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::DELETE, url)
            .headers(self.headers.clone())
            .query(&[("listenKey", listen_key)]);
        send(request).await
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
        let url = format!("{}{}", self.base_url, Path::UserDataStreamIsolated);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::POST, url)
            .headers(self.headers.clone())
            .query(&[("symbol", symbol)]);
        send(request).await
    }

    /// Extend an isolated-margin listenKey's lifetime by 60 minutes.
    pub async fn keepalive_isolated_listen_key(
        &self,
        symbol: &str,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let url = format!("{}{}", self.base_url, Path::UserDataStreamIsolated);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::PUT, url)
            .headers(self.headers.clone())
            .query(&[("symbol", symbol), ("listenKey", listen_key)]);
        send(request).await
    }

    /// Close an isolated-margin listenKey.
    pub async fn close_isolated_listen_key(
        &self,
        symbol: &str,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        let url = format!("{}{}", self.base_url, Path::UserDataStreamIsolated);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::DELETE, url)
            .headers(self.headers.clone())
            .query(&[("symbol", symbol), ("listenKey", listen_key)]);
        send(request).await
    }
}

async fn send<T>(request: RequestBuilder) -> Result<Response<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let response = request.send().await?;
    let status = response.status();
    let headers = parse_headers(response.headers());
    let json = response.text().await?;

    if !status.is_success() {
        #[cfg(debug_assertions)]
        tracing::debug!(?status, ?json, "request failed");

        let api_err = deserialize_json::<ApiError>(&json)?;
        return Err(Error::Api(api_err));
    }

    let result = deserialize_json(&json)?;
    Ok(Response { result, headers })
}

fn parse_headers(headers: &HeaderMap) -> Headers {
    let retry_after = headers
        .get(HEADER_RETRY_AFTER)
        .and_then(|h| h.to_str().unwrap_or_default().parse().ok());
    Headers { retry_after }
}
