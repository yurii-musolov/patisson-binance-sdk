use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    derivatives::usds_margined_futures::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountBalance, AccountInformation, AccountTrade, ActionResult,
            CancelAllOpenOrdersParams, CancelOrderParams, ChangeInitialLeverageParams,
            ChangeMarginTypeParams, ChangePositionModeParams, EmptyResponse, ExchangeInfo,
            GetAccountBalanceParams, GetAccountInformationParams, GetAccountTradeListParams,
            GetAllOrdersParams, GetCurrentPositionModeParams, GetKlineListParams,
            GetMarkPriceParams, GetOpenOrdersParams, GetOrderBookParams,
            GetPositionInformationParams, GetSymbolOrderBookTickerParams,
            GetSymbolPriceTickerParams, Kline, Leverage, ListenKey, MarkPrice, NewOrderRequest,
            NewOrderResponse, Order, OrderBook, Position, PositionMode, PrivateConfig,
            PublicConfig, QueryOrderParams, Response, ServerTime, SymbolOrderBookTicker,
            SymbolPriceTicker, TestConnectivity,
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
const COST_NEW_ORDER: Cost = Cost::weight_and_orders(1, 1);
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

    /// Close the listenKey. The WebSocket connection associated with the
    /// key will be dropped by the server.
    pub async fn close_listen_key(&self) -> Result<Response<EmptyResponse>, Error> {
        let req = self.http.request(Method::DELETE, Path::ListenKey);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
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
