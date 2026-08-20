use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::sign_query,
    derivatives::coin_margined_futures::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountInformation, ExchangeInfo, GetAccountInformationParams, GetKlineListParams,
            GetOrderBookParams, Kline, NewOrderRequest, NewOrderResponse, Order, OrderBook,
            PrivateConfig, PublicConfig, QueryOrderParams, Response, ServerTime, TestConnectivity,
        },
    },
    http::{HttpClient, RawResponse, SendError},
    rate_limit::Cost,
    serde::{deserialize_json, serialize_query},
    timestamp,
};

// Per-endpoint weights (Binance COIN-M Futures REST docs). These charge
// against the /dapi REQUEST_WEIGHT bucket per IP, separate from spot and USDⓈ-M.
const COST_PING: Cost = Cost::weight(1);
const COST_TIME: Cost = Cost::weight(1);
const COST_EXCHANGE_INFO: Cost = Cost::weight(1);
const COST_KLINES: Cost = Cost::weight(5);
const COST_ACCOUNT: Cost = Cost::weight(5);
const COST_NEW_ORDER: Cost = Cost::weight_and_orders(1, 1);
const COST_QUERY_ORDER: Cost = Cost::weight(1);

/// Depth-endpoint weight scales with the requested level count.
/// Per /dapi/v1/depth docs: 5/10/20=2, 50=5, 100=10, 500=20, 1000=50.
fn cost_depth(limit: Option<u64>) -> Cost {
    let limit = limit.unwrap_or(500);
    let weight = match limit {
        0..=20 => 2,
        21..=50 => 5,
        51..=100 => 10,
        101..=500 => 20,
        _ => 50,
    };
    Cost::weight(weight)
}

pub struct PublicClient {
    http: HttpClient,
}

impl PublicClient {
    pub fn new(cfg: PublicConfig) -> Self {
        let http = HttpClient::new(
            cfg.base_url,
            cfg.headers.unwrap_or_default(),
            cfg.rate_limiter,
        )
        .expect("reqwest client builder failed");
        Self { http }
    }
}

// General
impl PublicClient {
    pub async fn test_connectivity(&self) -> Result<Response<TestConnectivity>, Error> {
        let req = self.http.request(Method::GET, Path::Ping);
        decode(self.http.send_raw(req, COST_PING).await)
    }

    pub async fn get_server_time(&self) -> Result<Response<ServerTime>, Error> {
        let req = self.http.request(Method::GET, Path::Time);
        decode(self.http.send_raw(req, COST_TIME).await)
    }

    pub async fn get_exchange_info(&self) -> Result<Response<ExchangeInfo>, Error> {
        let req = self.http.request(Method::GET, Path::ExchangeInfo);
        decode(self.http.send_raw(req, COST_EXCHANGE_INFO).await)
    }
}

// Market Data
impl PublicClient {
    pub async fn get_order_book(
        &self,
        params: GetOrderBookParams,
    ) -> Result<Response<OrderBook>, Error> {
        let cost = cost_depth(params.limit);
        let req = self.http.request(Method::GET, Path::Depth).query(&params);
        decode(self.http.send_raw(req, cost).await)
    }

    pub async fn get_kline_list(
        &self,
        params: GetKlineListParams,
    ) -> Result<Response<Vec<Kline>>, Error> {
        let req = self.http.request(Method::GET, Path::KLines).query(&params);
        decode(self.http.send_raw(req, COST_KLINES).await)
    }
}

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

// Trading
impl PrivateClient {
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

    pub async fn query_order(&self, params: QueryOrderParams) -> Result<Response<Order>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::Order));
        decode(self.http.send_raw(req, COST_QUERY_ORDER).await)
    }
}

// Account
impl PrivateClient {
    pub async fn account_information(
        &self,
        params: GetAccountInformationParams,
    ) -> Result<Response<AccountInformation>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::Account));
        decode(self.http.send_raw(req, COST_ACCOUNT).await)
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
