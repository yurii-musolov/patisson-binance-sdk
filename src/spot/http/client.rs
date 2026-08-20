use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::sign_query,
    http::{HttpClient, RawResponse, SendError},
    rate_limit::Cost,
    serde::{deserialize_json, serialize_query},
    spot::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountInformation, AggregateTrade, CurrentAveragePrice, ExchangeInfo,
            GetAccountInformationParams, GetAggregateTradesParams, GetCurrentAveragePriceParams,
            GetExchangeInfoParams, GetKlineListParams, GetOlderTradesParams, GetOrderBookParams,
            GetRecentTradesParams, GetTickerPriceChangeStatisticsParams, Kline, NewOrderRequest,
            NewOrderResponse, Order, OrderBook, PrivateConfig, PublicConfig, QueryOrderParams,
            RecentTrade, Response, ServerTime, TestCommissionRates, TestConnectivity,
            TickerPriceChangeStatistic,
        },
    },
    timestamp,
};

// Per-endpoint weights (Binance Spot REST docs). The single-symbol path for
// /ticker/24hr and friends is the common case; without a symbol the cost
// scales with the symbol count — caller can install a custom RateLimiter or
// override globally if they care about the fine-grained variant.
const COST_PING: Cost = Cost::weight(1);
const COST_TIME: Cost = Cost::weight(1);
const COST_EXCHANGE_INFO: Cost = Cost::weight(20);
const COST_TRADES: Cost = Cost::weight(25);
const COST_HISTORICAL_TRADES: Cost = Cost::weight(25);
const COST_AGG_TRADES: Cost = Cost::weight(4);
const COST_KLINES: Cost = Cost::weight(2);
const COST_AVG_PRICE: Cost = Cost::weight(2);
const COST_TICKER_24H_SINGLE: Cost = Cost::weight(4);
const COST_ACCOUNT: Cost = Cost::weight(20);
const COST_QUERY_ORDER: Cost = Cost::weight(4);
const COST_NEW_ORDER: Cost = Cost::weight_and_orders(1, 1);
const COST_TEST_ORDER: Cost = Cost::weight(1);

/// Depth-endpoint weight scales with the requested level count.
fn cost_depth(limit: Option<u64>) -> Cost {
    let limit = limit.unwrap_or(100);
    let weight = match limit {
        0..=100 => 5,
        101..=500 => 25,
        501..=1000 => 50,
        _ => 250,
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
    /// Test connectivity to the Rest API.
    pub async fn test_connectivity(&self) -> Result<Response<TestConnectivity>, Error> {
        let req = self.http.request(Method::GET, Path::Ping);
        decode(self.http.send_raw(req, COST_PING).await)
    }

    pub async fn get_server_time(&self) -> Result<Response<ServerTime>, Error> {
        let req = self.http.request(Method::GET, Path::Time);
        decode(self.http.send_raw(req, COST_TIME).await)
    }

    pub async fn get_exchange_info(
        &self,
        params: GetExchangeInfoParams,
    ) -> Result<Response<ExchangeInfo>, Error> {
        let req = self
            .http
            .request(Method::GET, Path::ExchangeInfo)
            .query(&params);
        decode(self.http.send_raw(req, COST_EXCHANGE_INFO).await)
    }
}

//  Market
impl PublicClient {
    pub async fn get_order_book(
        &self,
        params: GetOrderBookParams,
    ) -> Result<Response<OrderBook>, Error> {
        let cost = cost_depth(params.limit);
        let req = self.http.request(Method::GET, Path::Depth).query(&params);
        decode(self.http.send_raw(req, cost).await)
    }

    /// Get recent trades.
    pub async fn recent_trades_list(
        &self,
        params: GetRecentTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        let req = self.http.request(Method::GET, Path::Trades).query(&params);
        decode(self.http.send_raw(req, COST_TRADES).await)
    }

    /// Get older trades.
    pub async fn old_trade_lookup(
        &self,
        params: GetOlderTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        let req = self
            .http
            .request(Method::GET, Path::HistoricalTrades)
            .query(&params);
        decode(self.http.send_raw(req, COST_HISTORICAL_TRADES).await)
    }

    /// Compressed/Aggregate trades list.
    /// Get compressed, aggregate trades. Trades that fill at the time, from the same taker order, with the same price will have the quantity aggregated.
    ///
    /// If fromId, startTime, and endTime are not sent, the most recent aggregate trades will be returned.
    pub async fn aggregate_trades_list(
        &self,
        params: GetAggregateTradesParams,
    ) -> Result<Response<Vec<AggregateTrade>>, Error> {
        let req = self
            .http
            .request(Method::GET, Path::AggTrades)
            .query(&params);
        decode(self.http.send_raw(req, COST_AGG_TRADES).await)
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
        let req = self.http.request(Method::GET, Path::KLines).query(&params);
        decode(self.http.send_raw(req, COST_KLINES).await)
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
        let req = self
            .http
            .request(Method::GET, Path::UIKLines)
            .query(&params);
        decode(self.http.send_raw(req, COST_KLINES).await)
    }

    /// Current average price for a symbol.
    pub async fn get_current_average_price(
        &self,
        params: GetCurrentAveragePriceParams,
    ) -> Result<Response<CurrentAveragePrice>, Error> {
        let req = self
            .http
            .request(Method::GET, Path::AvgPrice)
            .query(&params);
        decode(self.http.send_raw(req, COST_AVG_PRICE).await)
    }

    /// 24 hour rolling window price change statistics. Careful when accessing this with no symbol.
    pub async fn ticker_price_change_statistics(
        &self,
        params: GetTickerPriceChangeStatisticsParams,
    ) -> Result<Response<TickerPriceChangeStatistic>, Error> {
        let req = self
            .http
            .request(Method::GET, Path::Ticker24hr)
            .query(&params);
        decode(self.http.send_raw(req, COST_TICKER_24H_SINGLE).await)
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
        // Binance accepts POST params in either body or URL query; we use the
        // URL form because the body form would clash with `Content-Type` rules
        // some clients add by default.
        let req = self
            .http
            .request(Method::POST, format!("{}?{query}", Path::Order));
        decode(self.http.send_raw(req, COST_NEW_ORDER).await)
    }

    /// Test new order creation and signature/recvWindow long. Creates and validates a new order but does not send it into the matching engine.
    pub async fn test_new_order(
        &self,
        params: NewOrderRequest,
    ) -> Result<Response<TestCommissionRates>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::POST, format!("{}?{query}", Path::OrderTest));
        decode(self.http.send_raw(req, COST_TEST_ORDER).await)
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
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::Account));
        decode(self.http.send_raw(req, COST_ACCOUNT).await)
    }

    /// Check an order's status.
    /// Notes:
    /// Either orderId or origClientOrderId must be sent.
    /// If both orderId and origClientOrderId are provided, the orderId is searched first, then the origClientOrderId from that result is checked against that order. If both conditions are not met the request will be rejected.
    /// For some historical orders cummulativeQuoteQty will be < 0, meaning the data is not available at this time.
    pub async fn query_order(&self, params: QueryOrderParams) -> Result<Response<Order>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::Order));
        decode(self.http.send_raw(req, COST_QUERY_ORDER).await)
    }
}

/// Shared decode step: turn a `RawResponse` (or `SendError`) into a typed
/// `Response<T>` (or product `Error`). All spot endpoints funnel through here.
fn decode<T>(raw: Result<RawResponse, SendError>) -> Result<Response<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let raw = raw?;
    if !raw.status.is_success() {
        #[cfg(debug_assertions)]
        tracing::debug!(status = ?raw.status, body = ?raw.body, "request failed");

        // Binance returns `{"code":-XXXX,"msg":"..."}` on error.
        let api_err = deserialize_json::<ApiError>(&raw.body)?;
        return Err(Error::Api(api_err));
    }
    let result = deserialize_json(&raw.body)?;
    Ok(Response {
        result,
        headers: raw.headers,
    })
}
