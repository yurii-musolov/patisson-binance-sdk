use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    http::{self, HttpClient, RawResponse, SendError},
    margin::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            BorrowRepayParams, BorrowRepayRecords, BorrowRepayResult, CancelAllOpenOrdersParams,
            CancelOrderParams, CanceledOrder, CanceledOrderOrList, EmptyResponse,
            ForceLiquidationRecords, GetAccountTradeListParams, GetAllIsolatedMarginSymbolsParams,
            GetAllMarginAssetsParams, GetAllOrdersParams, GetBorrowRepayRecordsParams,
            GetForceLiquidationRecordParams, GetIsolatedMarginAccountParams,
            GetMarginAccountParams, GetMarginInterestRateHistoryParams, GetMaxBorrowableParams,
            GetMaxTransferOutAmountParams, GetOpenOrdersParams, GetPriceIndexParams,
            InterestRateRecord, IsolatedMarginAccount, IsolatedMarginSymbol, ListenKey,
            MarginAccount, MarginAsset, MaxBorrowable, MaxTransferable, NewOrderRequest,
            NewOrderResponse, Order, PriceIndex, PrivateConfig, QueryOrderParams, Response, Trade,
        },
    },
    rate_limit::Cost,
};

// Per-endpoint weights (Binance Margin REST docs). These charge against the
// shared spot REQUEST_WEIGHT bucket per IP; share the same `Arc<RateLimiter>`
// with `spot::http::*Client` if you want correct accounting.
const COST_ALL_ASSETS: Cost = Cost::weight(1);
const COST_MARGIN_ACCOUNT: Cost = Cost::weight(10);
const COST_NEW_ORDER: Cost = Cost::uid_weight_and_orders(6, 1);
/// `MARGIN_BUY` / `AUTO_BORROW_REPAY` orders borrow and weigh more.
const COST_NEW_ORDER_BORROWING: Cost = Cost::uid_weight_and_orders(1500, 1);
const COST_QUERY_ORDER: Cost = Cost::weight(10);
const COST_MAX_BORROWABLE: Cost = Cost::weight(50);
const COST_LISTEN_KEY: Cost = Cost::uid_weight(1);
const COST_CLOSE_LISTEN_KEY: Cost = Cost::uid_weight(3000);
const COST_CANCEL_ORDER: Cost = Cost::weight(10);
const COST_CANCEL_ALL_OPEN_ORDERS: Cost = Cost::weight(1);
const COST_OPEN_ORDERS_SYMBOL: Cost = Cost::weight(10);
const COST_OPEN_ORDERS_ALL: Cost = Cost::weight(200);
const COST_ALL_ORDERS: Cost = Cost::weight(200);
const COST_MY_TRADES: Cost = Cost::weight(10);
/// Borrow/repay execution is one of the heaviest Margin endpoints on
/// Binance's documented weight table.
const COST_BORROW_REPAY: Cost = Cost::uid_weight(1500);
const COST_BORROW_REPAY_RECORDS: Cost = Cost::weight(10);
const COST_ISOLATED_ACCOUNT: Cost = Cost::weight(10);
const COST_ISOLATED_SYMBOLS: Cost = Cost::weight(10);
const COST_INTEREST_RATE_HISTORY: Cost = Cost::weight(1);
const COST_PRICE_INDEX: Cost = Cost::weight(10);
const COST_MAX_TRANSFERABLE: Cost = Cost::weight(50);
const COST_FORCE_LIQUIDATION_REC: Cost = Cost::weight(1);

/// Client for the authenticated `/sapi/v1/margin/*` surface.
///
/// Margin has no public endpoints — for unauthenticated market data
/// (klines, depth, tickers) and connectivity (`/api/v3/ping`, `/api/v3/time`),
/// use [`crate::spot::http::PublicClient`].
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

// Margin metadata
impl PrivateClient {
    /// Get all margin assets supported by the exchange.
    pub async fn get_all_assets(
        &self,
        params: GetAllMarginAssetsParams,
    ) -> Result<Response<Vec<MarginAsset>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AllAssets,
            &params,
            COST_ALL_ASSETS,
        )
        .await
    }
}

// Cross margin account
impl PrivateClient {
    /// Get the caller's cross-margin account snapshot (balances, level, etc.).
    pub async fn margin_account(
        &self,
        params: GetMarginAccountParams,
    ) -> Result<Response<MarginAccount>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::Account,
            &params,
            COST_MARGIN_ACCOUNT,
        )
        .await
    }
}

// Margin trading
impl PrivateClient {
    /// Place a new margin order.
    ///
    /// Set `is_isolated = IsIsolated::True` to route the order to the isolated
    /// margin account for the symbol; otherwise the cross-margin account is used.
    /// Combine with [`crate::margin::SideEffectType::MarginBuy`] /
    /// [`crate::margin::SideEffectType::AutoRepay`]
    /// to opt into automatic borrowing or repayment when the order fills.
    pub async fn new_order(
        &self,
        params: NewOrderRequest,
    ) -> Result<Response<NewOrderResponse>, Error> {
        let cost = if params.borrows() {
            COST_NEW_ORDER_BORROWING
        } else {
            COST_NEW_ORDER
        };
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::Order,
            &params,
            cost,
        )
        .await
    }

    /// Look up a single margin order by `order_id` or `orig_client_order_id`.
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

    /// Cancel an active margin order.
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

    /// Cancel all active orders on a symbol, including OCO orders.
    pub async fn cancel_all_open_orders(
        &self,
        params: CancelAllOpenOrdersParams,
    ) -> Result<Response<Vec<CanceledOrderOrList>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::DELETE,
            Path::OpenOrders,
            &params,
            COST_CANCEL_ALL_OPEN_ORDERS,
        )
        .await
    }

    /// Get all open orders. Careful when accessing this with no symbol —
    /// the request weight scales with the number of symbols currently trading.
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

    /// Get all orders on a symbol: active, canceled, or filled.
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

    /// Get trades for a specific margin account and symbol.
    pub async fn get_account_trade_list(
        &self,
        params: GetAccountTradeListParams,
    ) -> Result<Response<Vec<Trade>>, Error> {
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

    /// Every order from `params.order_id` on (from the oldest when unset),
    /// walking pages of 500 by `orderId`; at most `max_pages` requests.
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
                params.limit = Some(500);
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
    ) -> Result<crate::AllPages<Trade>, Error> {
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
            |trade: &Trade| trade.id as u64,
        )
        .await
    }
}

// Margin risk / liquidation
impl PrivateClient {
    /// Get force-liquidation records for the margin account.
    pub async fn get_force_liquidation_record(
        &self,
        params: GetForceLiquidationRecordParams,
    ) -> Result<Response<ForceLiquidationRecords>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::ForceLiquidationRec,
            &params,
            COST_FORCE_LIQUIDATION_REC,
        )
        .await
    }
}

// Margin borrow / repay
impl PrivateClient {
    /// Query the maximum borrowable amount for an asset.
    ///
    /// Pass `isolated_symbol` to ask about the isolated account for a specific
    /// symbol; without it the call reports the cross-margin limit.
    pub async fn max_borrowable(
        &self,
        params: GetMaxBorrowableParams,
    ) -> Result<Response<MaxBorrowable>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::MaxBorrowable,
            &params,
            COST_MAX_BORROWABLE,
        )
        .await
    }

    /// Execute a borrow or repay against the cross- or isolated-margin
    /// account. This is the action endpoint — [`Self::max_borrowable`] only
    /// queries the limit.
    pub async fn borrow_repay(
        &self,
        params: BorrowRepayParams,
    ) -> Result<Response<BorrowRepayResult>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::BorrowRepay,
            &params,
            COST_BORROW_REPAY,
        )
        .await
    }

    /// Query past borrow/repay records for the margin account.
    pub async fn get_borrow_repay_records(
        &self,
        params: GetBorrowRepayRecordsParams,
    ) -> Result<Response<BorrowRepayRecords>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::BorrowRepay,
            &params,
            COST_BORROW_REPAY_RECORDS,
        )
        .await
    }

    /// Query the daily interest rate history charged for an asset.
    pub async fn get_margin_interest_rate_history(
        &self,
        params: GetMarginInterestRateHistoryParams,
    ) -> Result<Response<Vec<InterestRateRecord>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::InterestRateHistory,
            &params,
            COST_INTEREST_RATE_HISTORY,
        )
        .await
    }

    /// Query the maximum amount transferable out of the margin account for
    /// an asset.
    pub async fn get_max_transfer_out_amount(
        &self,
        params: GetMaxTransferOutAmountParams,
    ) -> Result<Response<MaxTransferable>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::MaxTransferable,
            &params,
            COST_MAX_TRANSFERABLE,
        )
        .await
    }
}

// Isolated margin account
impl PrivateClient {
    /// Get the caller's isolated-margin account snapshot. Without `symbols`,
    /// every isolated pair with non-zero assets/liabilities/borrow history
    /// is returned.
    pub async fn get_isolated_margin_account(
        &self,
        params: GetIsolatedMarginAccountParams,
    ) -> Result<Response<IsolatedMarginAccount>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::IsolatedAccount,
            &params,
            COST_ISOLATED_ACCOUNT,
        )
        .await
    }
}

// Margin metadata
impl PrivateClient {
    /// Get all isolated-margin symbols supported by the exchange (or a
    /// single symbol's eligibility, if `symbol` is set).
    pub async fn get_all_isolated_margin_symbols(
        &self,
        params: GetAllIsolatedMarginSymbolsParams,
    ) -> Result<Response<Vec<IsolatedMarginSymbol>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::IsolatedAllPairs,
            &params,
            COST_ISOLATED_SYMBOLS,
        )
        .await
    }

    /// Get the current price index for an isolated-margin symbol — used to
    /// calculate margin level.
    ///
    /// Market-data endpoint: authenticated by `X-MBX-APIKEY` header only, no
    /// `timestamp`/`signature` (mirrors the listen-key lifecycle calls below).
    pub async fn get_price_index(
        &self,
        params: GetPriceIndexParams,
    ) -> Result<Response<PriceIndex>, Error> {
        send_query(
            &self.http,
            Method::GET,
            Path::PriceIndex,
            &params,
            COST_PRICE_INDEX,
        )
        .await
    }
}

// User data stream (`/sapi/v1/margin/listen-key`).
//
// Binance retired `/sapi/v1/userDataStream` and `/sapi/v1/userDataStream/isolated`
// (deprecated 2025-10-27, removed from the API specification 2025-11-10);
// `/sapi/v1/margin/listen-key` replaces them and has no isolated variant.
// listenKey operations are authenticated by API key alone (`X-MBX-APIKEY`),
// without `timestamp` / `signature`, so they go through `send_query`.
impl PrivateClient {
    /// Create a listenKey for the margin user data stream.
    ///
    /// Connect to `wss://stream.binance.com:9443/ws/<listenKey>`. The key
    /// expires after 60 minutes; extend it with [`Self::keepalive_listen_key`]
    /// every 30 minutes.
    pub async fn create_listen_key(&self) -> Result<Response<ListenKey>, Error> {
        let req = self.http.request(Method::POST, Path::ListenKey);
        decode(self.http.send_raw(req, COST_LISTEN_KEY).await)
    }

    /// Extend the listenKey's lifetime by 60 minutes. Idempotent; safe to
    /// call on a schedule (recommended every 30 min).
    pub async fn keepalive_listen_key(
        &self,
        listen_key: &str,
    ) -> Result<Response<EmptyResponse>, Error> {
        send_query(
            &self.http,
            Method::PUT,
            Path::ListenKey,
            &[("listenKey", listen_key)],
            COST_LISTEN_KEY,
        )
        .await
    }

    /// Keep `listen_key` alive in the background (keepalive every 30
    /// minutes, see [`crate::KEEPALIVE_INTERVAL`]). Dropping the returned
    /// keeper stops it.
    pub fn keep_listen_key_alive(&self, listen_key: String) -> crate::ListenKeyKeeper<Error> {
        let client = self.clone();
        crate::ListenKeyKeeper::spawn(crate::KEEPALIVE_INTERVAL, move || {
            let client = client.clone();
            let listen_key = listen_key.clone();
            async move { client.keepalive_listen_key(&listen_key).await }
        })
    }

    /// Close the margin user data stream. The WebSocket connection associated
    /// with the key is dropped by the server.
    pub async fn close_listen_key(&self) -> Result<Response<EmptyResponse>, Error> {
        let req = self.http.request(Method::DELETE, Path::ListenKey);
        decode(self.http.send_raw(req, COST_CLOSE_LISTEN_KEY).await)
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
