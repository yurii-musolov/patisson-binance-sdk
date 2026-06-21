use reqwest::{self, Method, RequestBuilder, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::sign_query,
    derivatives::coin_margined_futures::{
        ApiError, Error, HEADER_RETRY_AFTER, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountInformation, ExchangeInfo, GetAccountInformationParams, GetKlineListParams,
            GetOrderBookParams, Headers, Kline, NewOrderRequest, NewOrderResponse, Order,
            OrderBook, PrivateConfig, PublicConfig, QueryOrderParams, Response, ServerTime,
            TestConnectivity,
        },
    },
    serde::{deserialize_json, serialize_query},
    timestamp,
};

pub struct PublicClient {
    base_url: String,
    headers: HeaderMap,
}

impl PublicClient {
    pub fn new(cfg: PublicConfig) -> Self {
        Self {
            base_url: cfg.base_url,
            headers: cfg.headers.unwrap_or_default(),
        }
    }
}

// General
impl PublicClient {
    pub async fn test_connectivity(&self) -> Result<Response<TestConnectivity>, Error> {
        let url = format!("{}{}", self.base_url, Path::Ping);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
    }

    pub async fn get_server_time(&self) -> Result<Response<ServerTime>, Error> {
        let url = format!("{}{}", self.base_url, Path::Time);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
    }

    pub async fn get_exchange_info(&self) -> Result<Response<ExchangeInfo>, Error> {
        let url = format!("{}{}", self.base_url, Path::ExchangeInfo);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
    }
}

// Market Data
impl PublicClient {
    pub async fn get_order_book(
        &self,
        params: GetOrderBookParams,
    ) -> Result<Response<OrderBook>, Error> {
        let url = format!("{}{}", self.base_url, Path::Depth);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);
        send(request).await
    }

    pub async fn get_kline_list(
        &self,
        params: GetKlineListParams,
    ) -> Result<Response<Vec<Kline>>, Error> {
        let url = format!("{}{}", self.base_url, Path::KLines);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);
        send(request).await
    }
}

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

// Trading
impl PrivateClient {
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
            .body(query); // Binance API accepts POST params in both query and body.
        send(request).await
    }

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

// Account
impl PrivateClient {
    pub async fn account_information(
        &self,
        params: GetAccountInformationParams,
    ) -> Result<Response<AccountInformation>, Error> {
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

        // Binance returns `{"code":-XXXX,"msg":"..."}` on error. Fall back to
        // the raw body if the shape doesn't match so nothing is silently lost.
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
