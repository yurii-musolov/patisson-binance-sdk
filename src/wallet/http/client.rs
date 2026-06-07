use reqwest::{self, Method, RequestBuilder, header::HeaderMap};
use tracing::debug;

use crate::{
    SensitiveString,
    crypto::sign_query,
    serde::{deserialize_json, serialize_query},
    timestamp,
    wallet::{
        ApiError, Error, HEADER_RETRY_AFTER, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountStatus, CoinInfo, Deposit, DepositAddress, GetAccountStatusParams,
            GetAllCoinsParams, GetDepositAddressParams, GetDepositHistoryParams, GetTradeFeeParams,
            GetWithdrawHistoryParams, Headers, PrivateConfig, Response, TradeFee, Withdraw,
        },
    },
};

/// Client for the authenticated `/sapi/v1/{capital,account,asset}/*` surface.
///
/// Wallet has no public endpoints — for unauthenticated market data
/// (klines, depth, tickers) and connectivity (`/api/v3/ping`, `/api/v3/time`),
/// use [`crate::spot::http::PublicClient`].
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
            if let Some(extra) = cfg.headers {
                headers.extend(extra);
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
        let url = format!("{}{}?{query}", self.base_url, Path::CapitalConfigGetAll);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
    }

    /// Fetch the deposit address for a coin (optionally on a specific network).
    pub async fn get_deposit_address(
        &self,
        params: GetDepositAddressParams,
    ) -> Result<Response<DepositAddress>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let url = format!("{}{}?{query}", self.base_url, Path::CapitalDepositAddress);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
    }

    /// Recent deposit history. Defaults: last 90 days, up to 1000 records.
    pub async fn get_deposit_history(
        &self,
        params: GetDepositHistoryParams,
    ) -> Result<Response<Vec<Deposit>>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let url = format!("{}{}?{query}", self.base_url, Path::CapitalDepositHistory);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
    }

    /// Recent withdraw history. Defaults: last 90 days, up to 1000 records.
    pub async fn get_withdraw_history(
        &self,
        params: GetWithdrawHistoryParams,
    ) -> Result<Response<Vec<Withdraw>>, Error> {
        let query = serialize_query(&params)?;
        let query = sign_query(&self.api_secret, timestamp(), &query);
        let url = format!("{}{}?{query}", self.base_url, Path::CapitalWithdrawHistory);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
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
        let url = format!("{}{}?{query}", self.base_url, Path::AccountStatus);
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        send(request).await
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
        let url = format!("{}{}?{query}", self.base_url, Path::AssetTradeFee);
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
        debug!(?status, ?json, "request failed");

        // Binance returns `{"code":-XXXX,"msg":"..."}` on error.
        let api_err = deserialize_json::<ApiError>(&json)?;
        return Err(Error::Api(api_err));
    }

    let result = deserialize_json(&json)?;
    Ok(Response { result, headers })
}

fn parse_headers(headers: &HeaderMap) -> Headers {
    let retry_after = headers
        .get(HEADER_RETRY_AFTER)
        .and_then(|h| h.to_str().unwrap_or_default().parse().ok());
    Headers { retry_after }
}
