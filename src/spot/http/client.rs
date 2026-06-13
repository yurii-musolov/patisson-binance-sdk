use reqwest::{self, Method, RequestBuilder, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::sign_query,
    serde::{deserialize_json, serialize_query},
    spot::{
        ApiError, Error, HEADER_RETRY_AFTER, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountInformation, AggregateTrade, CurrentAveragePrice, ExchangeInfo,
            GetAccountInformationParams, GetAggregateTradesParams, GetCurrentAveragePriceParams,
            GetExchangeInfoParams, GetKlineListParams, GetOlderTradesParams, GetOrderBookParams,
            GetRecentTradesParams, GetTickerPriceChangeStatisticsParams, Headers, Kline,
            NewOrderRequest, NewOrderResponse, Order, OrderBook, PrivateConfig, PublicConfig,
            QueryOrderParams, RecentTrade, Response, ServerTime, TestCommissionRates,
            TestConnectivity, TickerPriceChangeStatistic,
        },
    },
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
    /// Test connectivity to the Rest API.
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

    pub async fn get_exchange_info(
        &self,
        params: GetExchangeInfoParams,
    ) -> Result<Response<ExchangeInfo>, Error> {
        let url = format!("{}{}", self.base_url, Path::ExchangeInfo);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
    }
}

//  Market
impl PublicClient {
    pub async fn get_order_book(
        &self,
        params: GetOrderBookParams,
    ) -> Result<Response<OrderBook>, Error> {
        let url = format!("{}{}", self.base_url, Path::ExchangeInfo);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
    }

    /// Get recent trades.
    pub async fn recent_trades_list(
        &self,
        params: GetRecentTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        let url = format!("{}{}", self.base_url, Path::Trades);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
    }

    /// Get older trades.
    pub async fn old_trade_lookup(
        &self,
        params: GetOlderTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        let url = format!("{}{}", self.base_url, Path::HistoricalTrades);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
    }

    /// Compressed/Aggregate trades list.
    /// Get compressed, aggregate trades. Trades that fill at the time, from the same taker order, with the same price will have the quantity aggregated.
    ///
    /// If fromId, startTime, and endTime are not sent, the most recent aggregate trades will be returned.
    pub async fn aggregate_trades_list(
        &self,
        params: GetAggregateTradesParams,
    ) -> Result<Response<Vec<AggregateTrade>>, Error> {
        let url = format!("{}{}", self.base_url, Path::AggTrades);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
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
        let url = format!("{}{}", self.base_url, Path::KLines);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
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
        let url = format!("{}{}", self.base_url, Path::UIKLines);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
    }

    /// Current average price for a symbol.
    pub async fn get_current_average_price(
        &self,
        params: GetCurrentAveragePriceParams,
    ) -> Result<Response<CurrentAveragePrice>, Error> {
        let url = format!("{}{}", self.base_url, Path::AvgPrice);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone())
            .query(&params);

        send(request).await
    }

    /// 24 hour rolling window price change statistics. Careful when accessing this with no symbol.
    pub async fn ticker_price_change_statistics(
        &self,
        params: GetTickerPriceChangeStatisticsParams,
    ) -> Result<Response<TickerPriceChangeStatistic>, Error> {
        let url = format!("{}{}", self.base_url, Path::Ticker24hr);

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

            if let Some(extra_headers) = cfg.headers {
                headers.extend(extra_headers);
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
    /// Send in a new order.
    /// This adds 1 order to the EXCHANGE_MAX_ORDERS filter and the MAX_NUM_ORDERS filter.
    ///
    /// Other info:
    /// Any LIMIT or LIMIT_MAKER type order can be made an iceberg order by sending an icebergQty.
    /// Any order with an icebergQty MUST have timeInForce set to GTC.
    /// For STOP_LOSS, STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT and TAKE_PROFIT orders, trailingDelta can be combined with stopPrice.
    /// MARKET orders using quoteOrderQty will not break LOT_SIZE filter rules; the order will execute a quantity that will have the notional value as close as possible to quoteOrderQty. Trigger order price rules against market price for both MARKET and LIMIT versions:
    /// Price above market price: STOP_LOSS BUY, TAKE_PROFIT SELL
    /// Price below market price: STOP_LOSS SELL, TAKE_PROFIT BUY
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

    /// Test new order creation and signature/recvWindow long. Creates and validates a new order but does not send it into the matching engine.
    pub async fn test_new_order(
        &self,
        params: NewOrderRequest,
    ) -> Result<Response<TestCommissionRates>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let url = format!("{}{}", self.base_url, Path::OrderTest);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::POST, url)
            .headers(self.headers.clone())
            .body(query);

        send(request).await
    }
}

// Account
impl PrivateClient {
    /// Get current account information.
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

    /// Check an order's status.
    /// Notes:
    /// Either orderId or origClientOrderId must be sent.
    /// If both orderId and origClientOrderId are provided, the orderId is searched first, then the origClientOrderId from that result is checked against that order. If both conditions are not met the request will be rejected.
    /// For some historical orders cummulativeQuoteQty will be < 0, meaning the data is not available at this time.
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

        // Binance returns `{"code":-XXXX,"msg":"..."}` on error.
        let api_err = deserialize_json::<ApiError>(&json)?;
        return Err(Error::Api(api_err));
    }

    let result = deserialize_json(&json)?;
    Ok(Response { result, headers })
}

/// Parse response headers: Retry-After
fn parse_headers(headers: &HeaderMap) -> Headers {
    let retry_after = headers
        .get(HEADER_RETRY_AFTER)
        .and_then(|h| h.to_str().unwrap_or_default().parse().ok());

    Headers { retry_after }
}
