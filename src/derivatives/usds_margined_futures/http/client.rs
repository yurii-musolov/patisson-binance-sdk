use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    derivatives::usds_margined_futures::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountBalance, AccountInformation, AccountTrade, ActionResult, AlgoActionResult,
            AlgoOrder, AlgoOrderIdParams, CancelAllAlgoOpenOrdersParams, CancelAllOpenOrdersParams,
            CancelOrderParams, CanceledAlgoOrder, ChangeInitialLeverageParams,
            ChangeMarginTypeParams, ChangePositionModeParams, EmptyResponse, ExchangeInfo,
            GetAccountBalanceParams, GetAccountInformationParams, GetAccountTradeListParams,
            GetAllAlgoOrdersParams, GetAllOrdersParams, GetCurrentPositionModeParams,
            GetKlineListParams, GetMarkPriceParams, GetOpenAlgoOrdersParams, GetOpenOrdersParams,
            GetOrderBookParams, GetPositionInformationParams, GetSymbolOrderBookTickerParams,
            GetSymbolPriceTickerParams, Kline, Leverage, ListenKey, MarkPrice, NewAlgoOrderRequest,
            NewOrderRequest, NewOrderResponse, Order, OrderBook, Position, PositionMode,
            PrivateConfig, PublicConfig, QueryOrderParams, Response, ServerTime,
            SymbolOrderBookTicker, SymbolPriceTicker, TestConnectivity,
        },
    },
    http::{self, HttpClient, RawResponse, SendError},
    rate_limit::Cost,
};

// Per-endpoint weights (Binance USDⓈ-M Futures REST docs). These charge
// against the /fapi REQUEST_WEIGHT bucket per IP, separate from spot.
const COST_PING: Cost = Cost::weight(1);
const COST_TIME: Cost = Cost::weight(1);
const COST_EXCHANGE_INFO: Cost = Cost::weight(1);
const COST_KLINES: Cost = Cost::weight(5);
const COST_ACCOUNT: Cost = Cost::weight(5);
/// New orders cost no IP weight, only order count.
const COST_NEW_ORDER: Cost = Cost::weight_and_orders(0, 1);
const COST_QUERY_ORDER: Cost = Cost::weight(1);
const COST_CANCEL_ORDER: Cost = Cost::weight(1);
const COST_CANCEL_ALL_OPEN_ORDERS: Cost = Cost::weight(1);
const COST_OPEN_ORDERS_SYMBOL: Cost = Cost::weight(1);
const COST_OPEN_ORDERS_ALL: Cost = Cost::weight(40);
const COST_ALL_ORDERS: Cost = Cost::weight(5);
const COST_ACCOUNT_TRADE_LIST: Cost = Cost::weight(5);
const COST_CHANGE_LEVERAGE: Cost = Cost::weight(1);
const COST_CHANGE_MARGIN_TYPE: Cost = Cost::weight(1);
const COST_CHANGE_POSITION_MODE: Cost = Cost::weight(1);
const COST_GET_POSITION_MODE: Cost = Cost::weight(30);
const COST_POSITION_INFORMATION: Cost = Cost::weight(5);
const COST_ACCOUNT_BALANCE: Cost = Cost::weight(5);
const COST_LISTEN_KEY: Cost = Cost::weight(1);
const COST_TICKER_PRICE: Cost = Cost::weight(2);
const COST_TICKER_BOOK: Cost = Cost::weight(2);
const COST_MARK_PRICE: Cost = Cost::weight(1);

/// Depth-endpoint weight scales with the requested level count.
/// Per /fapi/v1/depth docs: 5/10/20=2, 50=5, 100=10, 500=20, 1000=50.
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

#[derive(Clone)]
pub struct PublicClient {
    http: HttpClient,
}

impl PublicClient {
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
        send_query(&self.http, Method::GET, Path::Depth, &params, cost).await
    }

    pub async fn get_kline_list(
        &self,
        params: GetKlineListParams,
    ) -> Result<Response<Vec<Kline>>, Error> {
        send_query(&self.http, Method::GET, Path::KLines, &params, COST_KLINES).await
    }

    /// Latest price for a symbol.
    pub async fn symbol_price_ticker(
        &self,
        params: GetSymbolPriceTickerParams,
    ) -> Result<Response<SymbolPriceTicker>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::TickerPrice,
            &params,
            COST_TICKER_PRICE,
        )
        .await
    }

    /// Best price/qty on the order book for a symbol.
    pub async fn symbol_order_book_ticker(
        &self,
        params: GetSymbolOrderBookTickerParams,
    ) -> Result<Response<SymbolOrderBookTicker>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::TickerBookTicker,
            &params,
            COST_TICKER_BOOK,
        )
        .await
    }

    /// Mark price, index price, and funding rate for a symbol.
    pub async fn mark_price(
        &self,
        params: GetMarkPriceParams,
    ) -> Result<Response<MarkPrice>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::PremiumIndex,
            &params,
            COST_MARK_PRICE,
        )
        .await
    }
}

#[derive(Clone)]
pub struct PrivateClient {
    http: HttpClient,
    api_secret: SensitiveString,
}

impl PrivateClient {
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
        .with_time_sync("/fapi/v1/time", cfg.resync_on_invalid_timestamp);
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
    /// Send a new order (`POST /fapi/v1/order`).
    ///
    /// The current API specification lists additional mandatory parameters
    /// only for `LIMIT` and `MARKET`: place conditional orders (`STOP`,
    /// `TAKE_PROFIT`, `*_MARKET`, `TRAILING_STOP_MARKET`) with
    /// [`Self::new_algo_order`].
    pub async fn new_order(
        &self,
        params: NewOrderRequest,
    ) -> Result<Response<NewOrderResponse>, Error> {
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

    /// Cancel an active order.
    pub async fn cancel_order(&self, params: CancelOrderParams) -> Result<Response<Order>, Error> {
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

    /// Cancel all open orders on a symbol, including OCO / conditional orders.
    pub async fn cancel_all_open_orders(
        &self,
        params: CancelAllOpenOrdersParams,
    ) -> Result<Response<ActionResult>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::DELETE,
            Path::AllOpenOrders,
            &params,
            COST_CANCEL_ALL_OPEN_ORDERS,
        )
        .await
    }

    /// Current open orders. If `symbol` is omitted, returns open orders for
    /// all symbols (heavier weight — see Binance docs).
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

    /// All orders (active, canceled, or filled) for a symbol.
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

    /// Trades for a specific account and symbol.
    pub async fn account_trade_list(
        &self,
        params: GetAccountTradeListParams,
    ) -> Result<Response<Vec<AccountTrade>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::UserTrades,
            &params,
            COST_ACCOUNT_TRADE_LIST,
        )
        .await
    }

    /// Change initial leverage for a symbol.
    pub async fn change_initial_leverage(
        &self,
        params: ChangeInitialLeverageParams,
    ) -> Result<Response<Leverage>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::Leverage,
            &params,
            COST_CHANGE_LEVERAGE,
        )
        .await
    }

    /// Change margin type (ISOLATED / CROSSED) for a symbol. No open
    /// position or order is allowed on the symbol when calling this.
    pub async fn change_margin_type(
        &self,
        params: ChangeMarginTypeParams,
    ) -> Result<Response<ActionResult>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::MarginType,
            &params,
            COST_CHANGE_MARGIN_TYPE,
        )
        .await
    }

    /// Change position mode (Hedge / One-way) for all symbols. No open
    /// position or order is allowed when calling this.
    pub async fn change_position_mode(
        &self,
        params: ChangePositionModeParams,
    ) -> Result<Response<ActionResult>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::PositionSideDual,
            &params,
            COST_CHANGE_POSITION_MODE,
        )
        .await
    }

    /// Get current position mode (Hedge / One-way) on this account.
    pub async fn get_current_position_mode(
        &self,
        params: GetCurrentPositionModeParams,
    ) -> Result<Response<PositionMode>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::PositionSideDual,
            &params,
            COST_GET_POSITION_MODE,
        )
        .await
    }

    /// Position information (v3). If `symbol` is omitted, returns positions
    /// for all symbols.
    pub async fn position_information(
        &self,
        params: GetPositionInformationParams,
    ) -> Result<Response<Vec<Position>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::PositionRiskV3,
            &params,
            COST_POSITION_INFORMATION,
        )
        .await
    }
}

// Account
impl PrivateClient {
    pub async fn account_information(
        &self,
        params: GetAccountInformationParams,
    ) -> Result<Response<AccountInformation>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AccountV3,
            &params,
            COST_ACCOUNT,
        )
        .await
    }

    /// Account balance (v2), one entry per asset.
    pub async fn futures_account_balance(
        &self,
        params: GetAccountBalanceParams,
    ) -> Result<Response<Vec<AccountBalance>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::BalanceV2,
            &params,
            COST_ACCOUNT_BALANCE,
        )
        .await
    }
}

// User data stream — listenKey lifecycle. These carry only the API-key
// header (already applied by `HttpClient`); unlike trading/account calls
// they are not HMAC-signed.
impl PrivateClient {
    /// Create a new listenKey, valid for 60 minutes — extend via
    /// [`Self::keepalive_listen_key`] every 30 min.
    pub async fn create_listen_key(&self) -> Result<Response<ListenKey>, Error> {
        let req = self.http.request(Method::POST, Path::ListenKey);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Extend a listenKey's lifetime by 60 minutes. Idempotent; safe to call
    /// on a schedule (recommended every 30 min).
    pub async fn keepalive_listen_key(&self) -> Result<Response<EmptyResponse>, Error> {
        let req = self.http.request(Method::PUT, Path::ListenKey);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Keep the account's listenKey alive in the background (keepalive every
    /// 30 minutes, see [`crate::KEEPALIVE_INTERVAL`]). Dropping the returned
    /// keeper stops it.
    pub fn keep_listen_key_alive(&self) -> crate::ListenKeyKeeper<Error> {
        let client = self.clone();
        crate::ListenKeyKeeper::spawn(crate::KEEPALIVE_INTERVAL, move || {
            let client = client.clone();
            async move { client.keepalive_listen_key().await }
        })
    }

    /// Close the listenKey. The WebSocket connection associated with the
    /// key will be dropped by the server.
    pub async fn close_listen_key(&self) -> Result<Response<EmptyResponse>, Error> {
        let req = self.http.request(Method::DELETE, Path::ListenKey);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }
}

// Algo (conditional) orders. Weights from the API specification; placing
// an algo order costs no IP weight, only order count.
const COST_NEW_ALGO_ORDER: Cost = Cost::weight_and_orders(0, 1);
const COST_ALGO_ORDER: Cost = Cost::weight(1);
const COST_ALL_ALGO_ORDERS: Cost = Cost::weight(5);

/// `/openAlgoOrders`: 1 for one symbol, 40 for every symbol.
fn cost_open_algo_orders(has_symbol: bool) -> Cost {
    Cost::weight(if has_symbol { 1 } else { 40 })
}

impl PrivateClient {
    /// Place a conditional order (`STOP`, `TAKE_PROFIT`, `STOP_MARKET`,
    /// `TAKE_PROFIT_MARKET`, `TRAILING_STOP_MARKET`) through the Algo Order
    /// API (`POST /fapi/v1/algoOrder`).
    pub async fn new_algo_order(
        &self,
        params: NewAlgoOrderRequest,
    ) -> Result<Response<AlgoOrder>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::AlgoOrder,
            &params,
            COST_NEW_ALGO_ORDER,
        )
        .await
    }

    /// Cancel one algo order (`DELETE /fapi/v1/algoOrder`).
    pub async fn cancel_algo_order(
        &self,
        params: AlgoOrderIdParams,
    ) -> Result<Response<CanceledAlgoOrder>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::DELETE,
            Path::AlgoOrder,
            &params,
            COST_ALGO_ORDER,
        )
        .await
    }

    /// Look up one algo order (`GET /fapi/v1/algoOrder`).
    pub async fn query_algo_order(
        &self,
        params: AlgoOrderIdParams,
    ) -> Result<Response<AlgoOrder>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AlgoOrder,
            &params,
            COST_ALGO_ORDER,
        )
        .await
    }

    /// Open algo orders (`GET /fapi/v1/openAlgoOrders`).
    pub async fn get_open_algo_orders(
        &self,
        params: GetOpenAlgoOrdersParams,
    ) -> Result<Response<Vec<AlgoOrder>>, Error> {
        let cost = cost_open_algo_orders(params.symbol.is_some());
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::OpenAlgoOrders,
            &params,
            cost,
        )
        .await
    }

    /// Algo orders of a symbol, open or not (`GET /fapi/v1/allAlgoOrders`).
    pub async fn get_all_algo_orders(
        &self,
        params: GetAllAlgoOrdersParams,
    ) -> Result<Response<Vec<AlgoOrder>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AllAlgoOrders,
            &params,
            COST_ALL_ALGO_ORDERS,
        )
        .await
    }

    /// Cancel every open algo order of a symbol
    /// (`DELETE /fapi/v1/algoOpenOrders`).
    pub async fn cancel_all_algo_open_orders(
        &self,
        params: CancelAllAlgoOpenOrdersParams,
    ) -> Result<Response<AlgoActionResult>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::DELETE,
            Path::AlgoOpenOrders,
            &params,
            COST_ALGO_ORDER,
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
    fn algo_order_request_and_weights() {
        use crate::{
            derivatives::usds_margined_futures::{OrderSide, OrderType, WorkingType},
            serde::serialize_query,
        };
        use rust_decimal::dec;

        let order =
            NewAlgoOrderRequest::conditional("BNBUSDT", OrderSide::SELL, OrderType::TakeProfit)
                .quantity(dec!(0.01))
                .price(dec!(750))
                .trigger_price(dec!(750))
                .working_type(WorkingType::ContractPrice)
                .reduce_only(false)
                .client_algo_id("tp-1");
        assert_eq!(
            serialize_query(&order).unwrap(),
            "algoType=CONDITIONAL&symbol=BNBUSDT&side=SELL&type=TAKE_PROFIT&quantity=0.01\
             &price=750&triggerPrice=750&workingType=CONTRACT_PRICE&reduceOnly=false\
             &clientAlgoId=tp-1"
        );
        assert_eq!(
            serialize_query(&AlgoOrderIdParams::by_client_algo_id("tp-1")).unwrap(),
            "clientAlgoId=tp-1"
        );
        assert_eq!(cost_open_algo_orders(true), Cost::weight(1));
        assert_eq!(cost_open_algo_orders(false), Cost::weight(40));
    }
}
