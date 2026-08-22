use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    http::{self, HttpClient, RawResponse, SendError},
    rate_limit::Cost,
    spot::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountCommission, AccountInformation, AccountTrade, AggregateTrade,
            CancelOpenOrdersParams, CancelOrderParams, CanceledOrder, CanceledOrderOrList,
            CurrentAveragePrice, EmptyResponse, ExchangeInfo, GetAccountCommissionParams,
            GetAccountInformationParams, GetAccountTradeListParams, GetAggregateTradesParams,
            GetAllOrdersParams, GetCurrentAveragePriceParams, GetExchangeInfoParams,
            GetKlineListParams, GetOlderTradesParams, GetOpenOrdersParams, GetOrderBookParams,
            GetOrderRateLimitParams, GetRecentTradesParams, GetSymbolOrderBookTickerParams,
            GetSymbolPriceTickerParams, GetTickerPriceChangeStatisticsParams,
            GetTickerTradingDayParams, Kline, ListenKey, NewOrderRequest, NewOrderResponse, Order,
            OrderBook, OrderRateLimit, PrivateConfig, PublicConfig, QueryOrderParams, RecentTrade,
            Response, ServerTime, SymbolOrderBookTickers, SymbolPriceTickers, TestCommissionRates,
            TestConnectivity, TickerPriceChangeStatistic,
        },
    },
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
const COST_CANCEL_ORDER: Cost = Cost::weight(1);
const COST_CANCEL_OPEN_ORDERS: Cost = Cost::weight(1);
const COST_OPEN_ORDERS_SYMBOL: Cost = Cost::weight(6);
const COST_OPEN_ORDERS_ALL: Cost = Cost::weight(80);
const COST_ALL_ORDERS: Cost = Cost::weight(20);
const COST_MY_TRADES: Cost = Cost::weight(20);
const COST_TICKER_PRICE_SINGLE: Cost = Cost::weight(2);
const COST_TICKER_PRICE_ALL: Cost = Cost::weight(4);
const COST_TICKER_BOOK_SINGLE: Cost = Cost::weight(2);
const COST_TICKER_BOOK_ALL: Cost = Cost::weight(4);
const COST_TICKER_TRADING_DAY_SINGLE: Cost = Cost::weight(4);
const COST_ACCOUNT_COMMISSION: Cost = Cost::weight(20);
const COST_ORDER_RATE_LIMIT: Cost = Cost::weight(40);
const COST_LISTEN_KEY: Cost = Cost::weight(2);

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
    pub fn new(cfg: PublicConfig) -> Result<Self, Error> {
        let http = HttpClient::new(
            cfg.base_url,
            cfg.headers.unwrap_or_default(),
            cfg.rate_limiter,
        )?;
        Ok(Self { http })
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
        send_query(
            &self.http,
            Method::GET,
            Path::ExchangeInfo,
            &params,
            COST_EXCHANGE_INFO,
        )
        .await
    }
}

//  Market
impl PublicClient {
    pub async fn get_order_book(
        &self,
        params: GetOrderBookParams,
    ) -> Result<Response<OrderBook>, Error> {
        let cost = cost_depth(params.limit);
        send_query(&self.http, Method::GET, Path::Depth, &params, cost).await
    }

    /// Get recent trades.
    pub async fn recent_trades_list(
        &self,
        params: GetRecentTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        send_query(&self.http, Method::GET, Path::Trades, &params, COST_TRADES).await
    }

    /// Get older trades.
    pub async fn old_trade_lookup(
        &self,
        params: GetOlderTradesParams,
    ) -> Result<Response<Vec<RecentTrade>>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::HistoricalTrades,
            &params,
            COST_HISTORICAL_TRADES,
        )
        .await
    }

    /// Compressed/Aggregate trades list.
    /// Get compressed, aggregate trades. Trades that fill at the time, from the same taker order, with the same price will have the quantity aggregated.
    ///
    /// If fromId, startTime, and endTime are not sent, the most recent aggregate trades will be returned.
    pub async fn aggregate_trades_list(
        &self,
        params: GetAggregateTradesParams,
    ) -> Result<Response<Vec<AggregateTrade>>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::AggTrades,
            &params,
            COST_AGG_TRADES,
        )
        .await
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
        send_query(&self.http, Method::GET, Path::KLines, &params, COST_KLINES).await
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
        send_query(
            &self.http,
            Method::GET,
            Path::UIKLines,
            &params,
            COST_KLINES,
        )
        .await
    }

    /// Current average price for a symbol.
    pub async fn get_current_average_price(
        &self,
        params: GetCurrentAveragePriceParams,
    ) -> Result<Response<CurrentAveragePrice>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::AvgPrice,
            &params,
            COST_AVG_PRICE,
        )
        .await
    }

    /// 24 hour rolling window price change statistics. Careful when accessing this with no symbol.
    pub async fn ticker_price_change_statistics(
        &self,
        params: GetTickerPriceChangeStatisticsParams,
    ) -> Result<Response<TickerPriceChangeStatistic>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::Ticker24hr,
            &params,
            COST_TICKER_24H_SINGLE,
        )
        .await
    }

    /// Trading Day Ticker. Price change statistics for a trading day, same
    /// shape as [`Self::ticker_price_change_statistics`].
    pub async fn ticker_trading_day(
        &self,
        params: GetTickerTradingDayParams,
    ) -> Result<Response<TickerPriceChangeStatistic>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::TickerTradingDay,
            &params,
            COST_TICKER_TRADING_DAY_SINGLE,
        )
        .await
    }

    /// Latest price for a symbol or symbols.
    pub async fn get_symbol_price_ticker(
        &self,
        params: GetSymbolPriceTickerParams,
    ) -> Result<Response<SymbolPriceTickers>, Error> {
        let cost = if params.symbol.is_some() {
            COST_TICKER_PRICE_SINGLE
        } else {
            COST_TICKER_PRICE_ALL
        };
        send_query(&self.http, Method::GET, Path::TickerPrice, &params, cost).await
    }

    /// Best price/qty on the order book for a symbol or symbols.
    pub async fn get_symbol_order_book_ticker(
        &self,
        params: GetSymbolOrderBookTickerParams,
    ) -> Result<Response<SymbolOrderBookTickers>, Error> {
        let cost = if params.symbol.is_some() {
            COST_TICKER_BOOK_SINGLE
        } else {
            COST_TICKER_BOOK_ALL
        };
        send_query(&self.http, Method::GET, Path::TickerBook, &params, cost).await
    }
}

pub struct PrivateClient {
    http: HttpClient,
    api_secret: SensitiveString,
}

impl PrivateClient {
    pub fn new(cfg: PrivateConfig) -> Result<Self, Error> {
        let headers = build_private_headers(&cfg)?;
        let http = HttpClient::new(cfg.base_url, headers, cfg.rate_limiter)?;
        Ok(Self {
            http,
            api_secret: cfg.api_secret,
        })
    }
}

fn build_private_headers(cfg: &PrivateConfig) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();
    let api_key = cfg.api_key.expose().parse()?;
    headers.append(HEADER_X_MBX_APIKEY, api_key);
    if let Some(extra) = &cfg.headers {
        headers.extend(extra.clone());
    }
    Ok(headers)
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
        // Binance accepts POST params in either body or URL query; we use the
        // URL form because the body form would clash with `Content-Type` rules
        // some clients add by default.
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::Order,
            &params,
            COST_NEW_ORDER,
        )
        .await
    }

    /// Test new order creation and signature/recvWindow long. Creates and validates a new order but does not send it into the matching engine.
    pub async fn test_new_order(
        &self,
        params: NewOrderRequest,
    ) -> Result<Response<TestCommissionRates>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::OrderTest,
            &params,
            COST_TEST_ORDER,
        )
        .await
    }

    /// Cancel an active order.
    /// Either orderId or origClientOrderId must be sent.
    pub async fn cancel_order(
        &self,
        params: CancelOrderParams,
    ) -> Result<Response<CanceledOrder>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::DELETE,
            Path::Order,
            &params,
            COST_CANCEL_ORDER,
        )
        .await
    }

    /// Cancels all active orders on a symbol. This includes orders that are
    /// part of an order list.
    pub async fn cancel_open_orders(
        &self,
        params: CancelOpenOrdersParams,
    ) -> Result<Response<Vec<CanceledOrderOrList>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::DELETE,
            Path::OpenOrders,
            &params,
            COST_CANCEL_OPEN_ORDERS,
        )
        .await
    }
}

// Account
impl PrivateClient {
    /// Get current account information.
    pub async fn account_information(
        &self,
        params: GetAccountInformationParams,
    ) -> Result<Response<AccountInformation>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::Account,
            &params,
            COST_ACCOUNT,
        )
        .await
    }

    /// Check an order's status.
    /// Notes:
    /// Either orderId or origClientOrderId must be sent.
    /// If both orderId and origClientOrderId are provided, the orderId is searched first, then the origClientOrderId from that result is checked against that order. If both conditions are not met the request will be rejected.
    /// For some historical orders cummulativeQuoteQty will be < 0, meaning the data is not available at this time.
    pub async fn query_order(&self, params: QueryOrderParams) -> Result<Response<Order>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::Order,
            &params,
            COST_QUERY_ORDER,
        )
        .await
    }

    /// Get all open orders on a symbol, or all symbols if `symbol` is
    /// omitted (much heavier — weight scales with the symbol count).
    pub async fn get_open_orders(
        &self,
        params: GetOpenOrdersParams,
    ) -> Result<Response<Vec<Order>>, Error> {
        let cost = if params.symbol.is_some() {
            COST_OPEN_ORDERS_SYMBOL
        } else {
            COST_OPEN_ORDERS_ALL
        };
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::OpenOrders,
            &params,
            cost,
        )
        .await
    }

    /// Get all account orders: active, canceled, filled or expired.
    pub async fn get_all_orders(
        &self,
        params: GetAllOrdersParams,
    ) -> Result<Response<Vec<Order>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AllOrders,
            &params,
            COST_ALL_ORDERS,
        )
        .await
    }

    /// Get trades for a specific account and symbol.
    pub async fn get_account_trade_list(
        &self,
        params: GetAccountTradeListParams,
    ) -> Result<Response<Vec<AccountTrade>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::MyTrades,
            &params,
            COST_MY_TRADES,
        )
        .await
    }

    /// Get current account commission rates for a symbol.
    pub async fn get_account_commission(
        &self,
        params: GetAccountCommissionParams,
    ) -> Result<Response<AccountCommission>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AccountCommission,
            &params,
            COST_ACCOUNT_COMMISSION,
        )
        .await
    }

    /// Displays the user's current order count usage for all intervals.
    pub async fn get_order_rate_limit(
        &self,
        params: GetOrderRateLimitParams,
    ) -> Result<Response<Vec<OrderRateLimit>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::RateLimitOrder,
            &params,
            COST_ORDER_RATE_LIMIT,
        )
        .await
    }
}

// User data stream lifecycle. Unlike trading/account endpoints, listenKey
// operations are authenticated by API key alone (`X-MBX-APIKEY` header) —
// they do NOT take `timestamp` / `signature`, so these methods go through
// `send_query` (unsigned) rather than `send_signed`.
impl PrivateClient {
    /// Create a new listenKey for the spot user data stream.
    ///
    /// Returns a key that can be used to connect to
    /// `wss://stream.binance.com:9443/ws/<listenKey>`. The key expires after
    /// 60 minutes — extend via [`Self::keepalive_listen_key`] every 30 min.
    pub async fn create_listen_key(&self) -> Result<Response<ListenKey>, Error> {
        let req = self.http.request(Method::POST, Path::UserDataStream);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Extend a listenKey's lifetime by 60 minutes. Idempotent; safe to call
    /// on a schedule (recommended every 30 min).
    pub async fn keepalive_listen_key(
        &self,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        send_query(
            &self.http,
            Method::PUT,
            Path::UserDataStream,
            &[("listenKey", listen_key)],
            COST_LISTEN_KEY,
        )
        .await
    }

    /// Close a listenKey. The WebSocket connection associated with the key
    /// will be dropped by the server.
    pub async fn close_listen_key(
        &self,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        send_query(
            &self.http,
            Method::DELETE,
            Path::UserDataStream,
            &[("listenKey", listen_key)],
            COST_LISTEN_KEY,
        )
        .await
    }
}

/// Thin pins of the shared `crate::http` primitives (see `src/http.rs`) onto
/// this product's own `ApiError`/`Error` types, so every endpoint above can
/// call `decode`/`send_signed`/`send_query` directly instead of hand-copying
/// the serialize → sign → request → send → decode sequence.
fn decode<T>(raw: Result<RawResponse, SendError>) -> Result<Response<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    http::decode::<T, ApiError, Error>(raw)
}

async fn send_signed<T, P>(
    http_client: &HttpClient,
    api_secret: &SensitiveString,
    method: Method,
    path: impl std::fmt::Display,
    params: &P,
    cost: Cost,
) -> Result<Response<T>, Error>
where
    P: serde::Serialize,
    T: serde::de::DeserializeOwned,
{
    http::send_signed::<T, P, ApiError, Error>(http_client, api_secret, method, path, params, cost)
        .await
}

async fn send_query<T, P>(
    http_client: &HttpClient,
    method: Method,
    path: impl std::fmt::Display,
    params: &P,
    cost: Cost,
) -> Result<Response<T>, Error>
where
    P: serde::Serialize,
    T: serde::de::DeserializeOwned,
{
    http::send_query::<T, P, ApiError, Error>(http_client, method, path, params, cost).await
}
