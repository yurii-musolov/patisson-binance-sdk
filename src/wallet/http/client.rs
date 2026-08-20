use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    crypto::sign_query,
    http::{HttpClient, RawResponse, SendError},
    rate_limit::Cost,
    serde::{deserialize_json, serialize_query},
    timestamp,
    wallet::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountStatus, CoinInfo, Deposit, DepositAddress, GetAccountStatusParams,
            GetAllCoinsParams, GetDepositAddressParams, GetDepositHistoryParams, GetTradeFeeParams,
            GetWithdrawHistoryParams, PrivateConfig, Response, TradeFee, Withdraw,
        },
    },
};

// Per-endpoint weights (Binance Wallet REST docs). Wallet endpoints share the
// spot REQUEST_WEIGHT bucket per IP, so share the same `Arc<RateLimiter>` with
// `spot::http::*Client` for correct accounting.
const COST_ALL_COINS: Cost = Cost::weight(10);
const COST_DEPOSIT_ADDRESS: Cost = Cost::weight(10);
const COST_DEPOSIT_HISTORY: Cost = Cost::weight(1);
const COST_WITHDRAW_HISTORY: Cost = Cost::weight(18_000);
const COST_ACCOUNT_STATUS: Cost = Cost::weight(1);
const COST_TRADE_FEE: Cost = Cost::weight(1);

/// Client for the authenticated `/sapi/v1/{capital,account,asset}/*` surface.
///
/// Wallet has no public endpoints — for unauthenticated market data
/// (klines, depth, tickers) and connectivity (`/api/v3/ping`, `/api/v3/time`),
/// use [`crate::spot::http::PublicClient`].
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

// Capital — coin / network metadata, deposits
impl PrivateClient {
    /// List every coin the account can hold, with per-network deposit and
    /// withdraw configuration. Heavyweight (often >300 coins); cache it.
    pub async fn get_all_coins(
        &self,
        params: GetAllCoinsParams,
    ) -> Result<Response<Vec<CoinInfo>>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self.http.request(
            Method::GET,
            format!("{}?{query}", Path::CapitalConfigGetAll),
        );
        decode(self.http.send_raw(req, COST_ALL_COINS).await)
    }

    /// Fetch the deposit address for a coin (optionally on a specific network).
    pub async fn get_deposit_address(
        &self,
        params: GetDepositAddressParams,
    ) -> Result<Response<DepositAddress>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self.http.request(
            Method::GET,
            format!("{}?{query}", Path::CapitalDepositAddress),
        );
        decode(self.http.send_raw(req, COST_DEPOSIT_ADDRESS).await)
    }

    /// Recent deposit history. Defaults: last 90 days, up to 1000 records.
    pub async fn get_deposit_history(
        &self,
        params: GetDepositHistoryParams,
    ) -> Result<Response<Vec<Deposit>>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self.http.request(
            Method::GET,
            format!("{}?{query}", Path::CapitalDepositHistory),
        );
        decode(self.http.send_raw(req, COST_DEPOSIT_HISTORY).await)
    }

    /// Recent withdraw history. Defaults: last 90 days, up to 1000 records.
    pub async fn get_withdraw_history(
        &self,
        params: GetWithdrawHistoryParams,
    ) -> Result<Response<Vec<Withdraw>>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self.http.request(
            Method::GET,
            format!("{}?{query}", Path::CapitalWithdrawHistory),
        );
        decode(self.http.send_raw(req, COST_WITHDRAW_HISTORY).await)
    }
}

// Account status
impl PrivateClient {
    /// Coarse account-wide status string (`"Normal"`, `"Margin Account dormant"`, …).
    pub async fn get_account_status(
        &self,
        params: GetAccountStatusParams,
    ) -> Result<Response<AccountStatus>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::AccountStatus));
        decode(self.http.send_raw(req, COST_ACCOUNT_STATUS).await)
    }
}

// Asset
impl PrivateClient {
    /// Maker / taker commission rates per symbol. Omit `symbol` to fetch all.
    pub async fn get_trade_fee(
        &self,
        params: GetTradeFeeParams,
    ) -> Result<Response<Vec<TradeFee>>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let req = self
            .http
            .request(Method::GET, format!("{}?{query}", Path::AssetTradeFee));
        decode(self.http.send_raw(req, COST_TRADE_FEE).await)
    }
}

fn decode<T>(raw: Result<RawResponse, SendError>) -> Result<Response<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let raw = raw?;
    if !raw.status.is_success() {
        #[cfg(debug_assertions)]
        tracing::debug!(status = ?raw.status, body = ?raw.body, "request failed");

        let api_err = deserialize_json::<ApiError>(&raw.body)?;
        return Err(Error::Api(api_err));
    }
    let result = deserialize_json(&raw.body)?;
    Ok(Response {
        result,
        headers: raw.headers,
    })
}
