use reqwest::{self, Method, RequestBuilder, header::HeaderMap};

use crate::spot::{
    AggregateTrade, CurrentAveragePrice, GetAggregateTradesParams, GetCurrentAveragePriceParams,
    GetKlineListParams, GetOlderTradesParams, GetOrderBookParams, GetRecentTradesParams, Kline,
    OrderBook, RecentTrade, TestConnectivity,
};

use super::{
    Error, ExchangeInfo, GetExchangeInfoParams, Headers, Response, ServerTime,
    crypto::SensitiveString, serde::deserialize_str, url::*,
};

pub struct ClientConfig {
    pub base_url: String,
    pub api_key: Option<SensitiveString>,
    pub api_secret: Option<SensitiveString>,
}

pub struct Client {
    base_url: String,
    cfg: ClientConfig,
}

impl Client {
    pub fn new(cfg: ClientConfig) -> Self {
        Self {
            base_url: cfg.base_url.clone(),
            cfg,
        }
    }
}

// General.
impl Client {
    /// Test connectivity to the Rest API.
    pub async fn test_connectivity(&self) -> Result<Response<TestConnectivity>, Error> {
        let url = format!("{}{}", self.base_url, Path::Time);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    pub async fn get_server_time(&self) -> Result<Response<ServerTime>, Error> {
        let url = format!("{}{}", self.base_url, Path::Time);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    pub async fn get_exchange_info(
        &self,
        params: GetExchangeInfoParams,
    ) -> Result<Response<ExchangeInfo>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::ExchangeInfo);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }
}

// Market Data.
impl Client {
    pub async fn get_order_book(
        &self,
        params: GetOrderBookParams,
    ) -> Result<Response<OrderBook>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::ExchangeInfo);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    /// Get recent trades.
    pub async fn recent_trades_list(
        &self,
        params: GetRecentTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::Trades);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    /// Get older trades.
    pub async fn old_trade_lookup(
        &self,
        params: GetOlderTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::HistoricalTrades);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    /// Compressed/Aggregate trades list.
    /// Get compressed, aggregate trades. Trades that fill at the time, from the same taker order, with the same price will have the quantity aggregated.
    ///
    /// If fromId, startTime, and endTime are not sent, the most recent aggregate trades will be returned.
    pub async fn aggregate_trades_list(
        &self,
        params: GetAggregateTradesParams,
    ) -> Result<Response<Vec<AggregateTrade>>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::AggTrades);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    /// Kline/candlestick bars for a symbol. Klines are uniquely identified by their open time.
    ///
    /// If startTime and endTime are not sent, the most recent klines are returned.
    /// Supported values for timeZone:
    /// Hours and minutes (e.g. -1:00, 05:45)
    /// Only hours (e.g. 0, 8, 4)
    /// Accepted range is strictly [-12:00 to +14:00] inclusive
    /// If timeZone provided, kline intervals are interpreted in that timezone instead of UTC.
    /// Note that startTime and endTime are always interpreted in UTC, regardless of timeZone.
    pub async fn get_kline_list(
        &self,
        params: GetKlineListParams,
    ) -> Result<Response<Vec<Kline>>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::KLines);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    /// UIKlines
    ///
    /// The request is similar to klines having the same parameters and response.
    /// uiKlines return modified kline data, optimized for presentation of candlestick charts.
    ///
    /// If startTime and endTime are not sent, the most recent klines are returned.
    /// Supported values for timeZone:
    /// Hours and minutes (e.g. -1:00, 05:45)
    /// Only hours (e.g. 0, 8, 4)
    /// Accepted range is strictly [-12:00 to +14:00] inclusive
    /// If timeZone provided, kline intervals are interpreted in that timezone instead of UTC.
    /// Note that startTime and endTime are always interpreted in UTC, regardless of timeZone.
    pub async fn get_ui_kline_list(
        &self,
        params: GetKlineListParams,
    ) -> Result<Response<Vec<Kline>>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::UIKLines);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }

    /// Current average price for a symbol.
    pub async fn get_current_average_price(
        &self,
        params: GetCurrentAveragePriceParams,
    ) -> Result<Response<CurrentAveragePrice>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::AvgPrice);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }
}

async fn send<T>(request: RequestBuilder) -> Result<Response<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let response = request.send().await?;
    let headers = parse_headers(&response.headers());
    let json = response.text().await?;

    let result = deserialize_str(&json)?;
    let response = Response { result, headers };
    Ok(response)
}

/// Parse response headers: Retry-After
fn parse_headers(headers: &HeaderMap) -> Headers {
    let retry_after = headers
        .get(HEADER_RETRY_AFTER)
        .map(|h| h.to_str().unwrap_or_default().parse().ok())
        .flatten();

    Headers { retry_after }
}
