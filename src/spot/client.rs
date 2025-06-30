use reqwest::{self, Method, RequestBuilder, StatusCode, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::make_sign,
    spot::{
        AccountInformation, AggregateTrade, CurrentAveragePrice, GetAccountInformationParams,
        GetAggregateTradesParams, GetCurrentAveragePriceParams, GetKlineListParams,
        GetOlderTradesParams, GetOrderBookParams, GetRecentTradesParams,
        GetTickerPriceChangeStatisticsParams, Kline, NewOrderParams, NewOrderResponse, Order,
        OrderBook, QueryOrderParams, RecentTrade, TestCommissionRates, TestConnectivity,
        TickerPriceChangeStatistic,
    },
};

use super::{
    Error, ExchangeInfo, GetExchangeInfoParams, Headers, Response, ServerTime,
    serde::deserialize_str, url::*,
};

pub struct GeneralClient {
    base_url: String,
}

impl GeneralClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl GeneralClient {
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

pub struct MarketClient {
    base_url: String,
}

impl MarketClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl MarketClient {
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

    /// 24 hour rolling window price change statistics. Careful when accessing this with no symbol.
    pub async fn ticker_price_change_statistics(
        &self,
        params: GetTickerPriceChangeStatisticsParams,
    ) -> Result<Response<TickerPriceChangeStatistic>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let url = format!("{}{}?{query}", self.base_url, Path::Ticker24hr);

        let client = reqwest::Client::builder().build()?;
        let request = client.request(Method::GET, url);

        let response = send(request).await?;
        Ok(response)
    }
}

pub struct TradingClient {
    base_url: String,
    headers: HeaderMap,
    sign: Box<dyn Fn(&str) -> String>,
}

impl TradingClient {
    pub fn new(base_url: String, api_key: SensitiveString, api_secret: SensitiveString) -> Self {
        let mut headers = HeaderMap::new();

        let api_key = api_key.expose().parse().unwrap();
        headers.append(HEADER_X_MBX_APIKEY, api_key);

        Self {
            base_url,
            headers,
            sign: Box::new(make_sign(api_secret)),
        }
    }
}

impl TradingClient {
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
        params: NewOrderParams,
    ) -> Result<Response<NewOrderResponse>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let body = (*self.sign)(&query);
        let url = format!("{}{}?{query}", self.base_url, Path::Order);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::POST, url)
            .headers(self.headers.clone())
            .body(body);

        let response = send(request).await?;

        Ok(response)
    }

    /// Test new order creation and signature/recvWindow long. Creates and validates a new order but does not send it into the matching engine.
    pub async fn test_new_order(
        &self,
        params: NewOrderParams,
        compute_commission_rates: bool,
    ) -> Result<Response<TestCommissionRates>, Error> {
        let mut query = serde_urlencoded::to_string(&params)?;
        if compute_commission_rates {
            query.push_str("&computeCommissionRates=true");
        }
        let body = (*self.sign)(&query);
        let url = format!("{}{}", self.base_url, Path::OrderTest);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::POST, url)
            .headers(self.headers.clone())
            .body(body);

        let response = send(request).await?;

        Ok(response)
    }
}

pub struct AccountClient {
    base_url: String,
    headers: HeaderMap,
    sign: Box<dyn Fn(&str) -> String>,
}

impl AccountClient {
    pub fn new(base_url: String, api_key: SensitiveString, api_secret: SensitiveString) -> Self {
        let mut headers = HeaderMap::new();

        let api_key = api_key.expose().parse().unwrap();
        headers.append(HEADER_X_MBX_APIKEY, api_key);

        Self {
            base_url,
            headers,
            sign: Box::new(make_sign(api_secret)),
        }
    }
}

impl AccountClient {
    /// Get current account information.
    pub async fn account_information(
        &self,
        params: GetAccountInformationParams,
    ) -> Result<Response<AccountInformation>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let query = (*self.sign)(&query);
        let url = format!("{}{}?{query}", self.base_url, Path::Account);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());

        let response = send(request).await?;

        Ok(response)
    }

    /// Check an order's status.
    /// Notes:
    /// Either orderId or origClientOrderId must be sent.
    /// If both orderId and origClientOrderId are provided, the orderId is searched first, then the origClientOrderId from that result is checked against that order. If both conditions are not met the request will be rejected.
    /// For some historical orders cummulativeQuoteQty will be < 0, meaning the data is not available at this time.
    pub async fn query_order(&self, params: QueryOrderParams) -> Result<Response<Order>, Error> {
        let query = serde_urlencoded::to_string(&params)?;
        let query = (*self.sign)(&query);
        let url = format!("{}{}?{query}", self.base_url, Path::Order);

        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());

        let response = send(request).await?;

        Ok(response)
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

    #[cfg(debug_assertions)]
    {
        if status != StatusCode::OK {
            println!("DEBUG: {status} {json}");
        }
    }

    // TODO: handle ApiError (code + msg)

    let result = deserialize_str(&json)?;
    let response = Response { result, headers };
    Ok(response)
}

/// Parse response headers: Retry-After
fn parse_headers(headers: &HeaderMap) -> Headers {
    let retry_after = headers
        .get(HEADER_RETRY_AFTER)
        .and_then(|h| h.to_str().unwrap_or_default().parse().ok());

    Headers { retry_after }
}
