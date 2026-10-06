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
            CurrentAveragePrice, ExchangeInfo, GetAccountCommissionParams,
            GetAccountInformationParams, GetAccountTradeListParams, GetAggregateTradesParams,
            GetAllOrdersParams, GetCurrentAveragePriceParams, GetExchangeInfoParams,
            GetKlineListParams, GetOlderTradesParams, GetOpenOrdersParams, GetOrderBookParams,
            GetOrderRateLimitParams, GetRecentTradesParams, GetSymbolOrderBookTickerParams,
            GetSymbolPriceTickerParams, GetTickerPriceChangeStatisticsParams,
            GetTickerTradingDayParams, Kline, NewOrderRequest, NewOrderResponse, Order, OrderBook,
            OrderRateLimit, PrivateConfig, PublicConfig, QueryOrderParams, RecentTrade, Response,
            ServerTime, SymbolOrderBookTickers, SymbolPriceTickers, TestCommissionRates,
            TestConnectivity, TickerPriceChangeStatistic, TickerTradingDay,
        },
    },
};

// Per-endpoint weights (Binance Spot REST docs). Weights that depend on the
// request parameters are computed by the `cost_*` functions below.
const COST_PING: Cost = Cost::weight(1);
const COST_TIME: Cost = Cost::weight(1);
const COST_EXCHANGE_INFO: Cost = Cost::weight(20);
const COST_TRADES: Cost = Cost::weight(25);
const COST_HISTORICAL_TRADES: Cost = Cost::weight(25);
const COST_AGG_TRADES: Cost = Cost::weight(4);
const COST_KLINES: Cost = Cost::weight(2);
const COST_AVG_PRICE: Cost = Cost::weight(2);
const COST_ACCOUNT: Cost = Cost::weight(20);
const COST_QUERY_ORDER: Cost = Cost::weight(4);
const COST_NEW_ORDER: Cost = Cost::weight_and_orders(1, 1);
const COST_CANCEL_ORDER: Cost = Cost::weight(1);
const COST_CANCEL_OPEN_ORDERS: Cost = Cost::weight(1);
const COST_OPEN_ORDERS_SYMBOL: Cost = Cost::weight(6);
const COST_OPEN_ORDERS_ALL: Cost = Cost::weight(80);
const COST_ALL_ORDERS: Cost = Cost::weight(20);
const COST_TICKER_PRICE_SINGLE: Cost = Cost::weight(2);
const COST_TICKER_PRICE_ALL: Cost = Cost::weight(4);
const COST_TICKER_BOOK_SINGLE: Cost = Cost::weight(2);
const COST_TICKER_BOOK_ALL: Cost = Cost::weight(4);
const COST_ACCOUNT_COMMISSION: Cost = Cost::weight(20);
const COST_ORDER_RATE_LIMIT: Cost = Cost::weight(40);

/// `/ticker/24hr`: 2 for 1-20 symbols, 40 for 21-100, 80 for more or for
/// all symbols.
fn cost_ticker_24h(symbol_count: Option<usize>) -> Cost {
    Cost::weight(match symbol_count {
        Some(0..=20) => 2,
        Some(21..=100) => 40,
        _ => 80,
    })
}

/// `/ticker/tradingDay`: 4 per symbol, capped at 200.
fn cost_ticker_trading_day(symbol_count: usize) -> Cost {
    Cost::weight((4 * symbol_count.max(1)).min(200) as u32)
}

/// `/order/test`: 1, or 20 when commission rates are computed.
fn cost_test_order(computes_commission_rates: bool) -> Cost {
    Cost::weight(if computes_commission_rates { 20 } else { 1 })
}

/// `/myTrades`: 5 with `orderId`, 20 without.
fn cost_my_trades(has_order_id: bool) -> Cost {
    Cost::weight(if has_order_id { 5 } else { 20 })
}

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

#[derive(Clone)]
/// Client for the public (unsigned) REST endpoints. Cheap to clone.
pub struct PublicClient {
    http: HttpClient,
}

impl PublicClient {
    /// Build a client; fails on an invalid proxy URL.
    pub fn new(cfg: PublicConfig) -> Result<Self, Error> {
        let http = HttpClient::new(
            cfg.base_url,
            cfg.headers.unwrap_or_default(),
            cfg.rate_limiter,
            cfg.timeouts,
            cfg.proxy.as_ref(),
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

    /// Server time.
    pub async fn get_server_time(&self) -> Result<Response<ServerTime>, Error> {
        let req = self.http.request(Method::GET, Path::Time);
        decode(self.http.send_raw(req, COST_TIME).await)
    }

    /// Trading rules and symbol information.
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
    /// Order book snapshot.
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
        let cost = cost_ticker_24h(params.symbol_count());
        send_query(&self.http, Method::GET, Path::Ticker24hr, &params, cost).await
    }

    /// Trading Day Ticker. Price change statistics for a trading day; the
    /// FULL shape has fewer fields than [`Self::ticker_price_change_statistics`].
    pub async fn ticker_trading_day(
        &self,
        params: GetTickerTradingDayParams,
    ) -> Result<Response<TickerTradingDay>, Error> {
        let cost = cost_ticker_trading_day(params.symbol_count());
        send_query(
            &self.http,
            Method::GET,
            Path::TickerTradingDay,
            &params,
            cost,
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

#[derive(Clone)]
/// Client for the signed REST endpoints. Cheap to clone; clones share the connection pool, rate limiter and clock offset.
pub struct PrivateClient {
    http: HttpClient,
    api_secret: SensitiveString,
}

impl PrivateClient {
    /// Build a client; fails on an invalid API key header or proxy URL.
    pub fn new(cfg: PrivateConfig) -> Result<Self, Error> {
        let headers = build_private_headers(&cfg)?;
        let http = HttpClient::new(
            cfg.base_url,
            headers,
            cfg.rate_limiter,
            cfg.timeouts,
            cfg.proxy.as_ref(),
        )?
        .with_time_offset(cfg.time_offset)
        .with_time_sync("/api/v3/time", cfg.resync_on_invalid_timestamp);
        Ok(Self {
            http,
            api_secret: cfg.api_secret,
        })
    }
}

impl PrivateClient {
    /// Read the server time and correct this client's clock (shared with
    /// its clones and the config's `TimeOffset`). Returns the offset in ms.
    pub async fn sync_time(&self) -> Result<i64, Error> {
        http::sync_time::<ApiError, Error>(&self.http).await
    }
}

fn build_private_headers(cfg: &PrivateConfig) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();
    let mut api_key: reqwest::header::HeaderValue = cfg.api_key.expose().parse()?;
    api_key.set_sensitive(true);
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
            cost_test_order(params.computes_commission_rates()),
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
            cost_my_trades(params.has_order_id()),
        )
        .await
    }

    /// Every order from `params.order_id` on (from the oldest when unset),
    /// walking pages of 1000 by `orderId`; at most `max_pages` requests.
    /// Other params (time range, ...) are passed through. Binance's own
    /// retention limits still apply.
    pub async fn get_all_orders_all(
        &self,
        params: GetAllOrdersParams,
        max_pages: usize,
    ) -> Result<crate::AllPages<Order>, Error> {
        let start = params.order_id.unwrap_or(0).max(0) as u64;
        crate::pagination::walk_by_id(
            start,
            max_pages,
            |from| {
                let mut params = params.clone();
                params.order_id = Some(from as i64);
                params.limit = Some(1000);
                async move { Ok(self.get_all_orders(params).await?.result) }
            },
            |order: &Order| order.order_id as u64,
        )
        .await
    }

    /// Every trade from `params.from_id` on (from the oldest when unset),
    /// walking pages of 1000 by `fromId`; at most `max_pages` requests.
    /// Binance rejects `fromId` together with some other params (see the
    /// endpoint's documentation).
    pub async fn get_account_trade_list_all(
        &self,
        params: GetAccountTradeListParams,
        max_pages: usize,
    ) -> Result<crate::AllPages<AccountTrade>, Error> {
        let start = params.from_id.unwrap_or(0).max(0) as u64;
        crate::pagination::walk_by_id(
            start,
            max_pages,
            |from| {
                let mut params = params.clone();
                params.from_id = Some(from as i64);
                params.limit = Some(1000);
                async move { Ok(self.get_account_trade_list(params).await?.result) }
            },
            |trade: &AccountTrade| trade.id as u64,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_header_is_marked_sensitive() {
        let cfg = PrivateConfig::new(
            "https://example.com",
            SensitiveString::from("my-api-key"),
            SensitiveString::from("my-api-secret"),
        );
        let headers = build_private_headers(&cfg).unwrap();
        let value = headers.get(HEADER_X_MBX_APIKEY).unwrap();
        assert!(value.is_sensitive());
        assert!(!format!("{headers:?}").contains("my-api-key"));
    }

    #[test]
    fn config_carries_default_and_overridden_timeouts() {
        use std::time::Duration;

        let cfg = PublicConfig::new("https://example.com");
        assert_eq!(cfg.timeouts, crate::Timeouts::default());
        assert_eq!(cfg.timeouts.request, crate::DEFAULT_HTTP_TIMEOUT);

        let cfg = cfg
            .timeout(Duration::from_secs(3))
            .connect_timeout(Duration::from_secs(1));
        assert_eq!(cfg.timeouts.request, Duration::from_secs(3));
        assert_eq!(cfg.timeouts.connect, Duration::from_secs(1));
        PublicClient::new(cfg).expect("client builds with custom timeouts");
    }

    #[test]
    fn parameter_dependent_weights_follow_binance_docs() {
        assert_eq!(cost_ticker_24h(Some(1)), Cost::weight(2));
        assert_eq!(cost_ticker_24h(Some(20)), Cost::weight(2));
        assert_eq!(cost_ticker_24h(Some(21)), Cost::weight(40));
        assert_eq!(cost_ticker_24h(Some(101)), Cost::weight(80));
        assert_eq!(cost_ticker_24h(None), Cost::weight(80));

        assert_eq!(cost_ticker_trading_day(1), Cost::weight(4));
        assert_eq!(cost_ticker_trading_day(10), Cost::weight(40));
        assert_eq!(cost_ticker_trading_day(51), Cost::weight(200));

        assert_eq!(cost_test_order(false), Cost::weight(1));
        assert_eq!(cost_test_order(true), Cost::weight(20));
        assert_eq!(cost_my_trades(true), Cost::weight(5));
        assert_eq!(cost_my_trades(false), Cost::weight(20));
    }

    #[test]
    fn ticker_params_expose_symbol_count_and_serialize_symbols() {
        use crate::spot::http::{GetTickerPriceChangeStatisticsParams, SymbolOrSymbols};
        let all = GetTickerPriceChangeStatisticsParams::Full(SymbolOrSymbols::new());
        assert_eq!(all.symbol_count(), None);
        let many = GetTickerPriceChangeStatisticsParams::Mini(
            SymbolOrSymbols::new().symbols(vec!["BTCUSDT".into(), "BNBBTC".into()]),
        );
        assert_eq!(many.symbol_count(), Some(2));
        assert_eq!(
            crate::serde::serialize_query(&many).unwrap(),
            "type=MINI&symbols=%5B%22BTCUSDT%22%2C%22BNBBTC%22%5D"
        );
    }

    #[test]
    fn new_documented_parameters_serialize() {
        use crate::{
            serde::serialize_query,
            spot::{
                CancelRestrictions, OrderResponseType, OrderSide, OrderType, PegOffsetType,
                PegPriceType, SymbolStatus,
                http::{CancelOrderParams, GetOrderBookParams, NewOrderRequest},
            },
        };

        let depth = GetOrderBookParams::new("BTCUSDT").symbol_status(SymbolStatus::Trading);
        assert_eq!(
            serialize_query(&depth).unwrap(),
            "symbol=BTCUSDT&symbolStatus=TRADING"
        );

        let cancel = CancelOrderParams::new("BTCUSDT")
            .order_id(1)
            .cancel_restrictions(CancelRestrictions::OnlyNew);
        assert_eq!(
            serialize_query(&cancel).unwrap(),
            "symbol=BTCUSDT&orderId=1&cancelRestrictions=ONLY_NEW"
        );

        let order = NewOrderRequest::new(
            "BTCUSDT",
            OrderSide::BUY,
            OrderType::Limit,
            OrderResponseType::ACK,
        )
        .peg_price_type(PegPriceType::PrimaryPeg)
        .peg_offset_value(3)
        .peg_offset_type(PegOffsetType::PriceLevel);
        let query = serialize_query(&order).unwrap();
        assert!(
            query.contains("pegPriceType=PRIMARY_PEG&pegOffsetValue=3&pegOffsetType=PRICE_LEVEL"),
            "{query}"
        );
    }

    #[test]
    fn clients_are_cheap_to_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<PublicClient>();
        assert_clone::<PrivateClient>();
        assert_clone::<crate::margin::http::PrivateClient>();
        assert_clone::<crate::wallet::http::PrivateClient>();
        assert_clone::<crate::derivatives::usds_margined_futures::http::PublicClient>();
        assert_clone::<crate::derivatives::usds_margined_futures::http::PrivateClient>();
        assert_clone::<crate::derivatives::coin_margined_futures::http::PublicClient>();
        assert_clone::<crate::derivatives::coin_margined_futures::http::PrivateClient>();
    }

    #[test]
    fn proxy_credentials_are_redacted_in_debug() {
        let cfg = PublicConfig::new("https://example.com").proxy("http://user:hunter2@proxy:8080");
        assert!(!format!("{cfg:?}").contains("hunter2"));
    }
}
