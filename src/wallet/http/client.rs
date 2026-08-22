use reqwest::{Method, header::HeaderMap};

use crate::{
    SensitiveString,
    http::{self, HttpClient, RawResponse, SendError},
    rate_limit::Cost,
    wallet::{
        ApiError, Error, HEADER_X_MBX_APIKEY, Path,
        http::{
            AccountApiTradingStatus, AccountStatus, AssetDividendRecord, CoinInfo, Deposit,
            DepositAddress, Empty, FastWithdrawSwitchParams, GetAccountApiTradingStatusParams,
            GetAccountStatusParams, GetAllCoinsParams, GetAssetDividendRecordParams,
            GetDepositAddressParams, GetDepositHistoryParams, GetTradeFeeParams,
            GetUniversalTransferHistoryParams, GetUserAssetParams, GetUserWalletBalanceParams,
            GetWithdrawHistoryParams, PrivateConfig, Response, SystemStatusResult, TradeFee,
            UniversalTransferHistory, UniversalTransferResult, UserAsset,
            UserUniversalTransferRequest, WalletBalance, Withdraw, WithdrawRequest, WithdrawResult,
        },
    },
};

// Per-endpoint weights (Binance Wallet REST docs). Wallet endpoints share the
// spot REQUEST_WEIGHT bucket per IP, so share the same `Arc<RateLimiter>` with
// `spot::http::*Client` for correct accounting.
const COST_ALL_COINS: Cost = Cost::weight(10);
const COST_DEPOSIT_ADDRESS: Cost = Cost::weight(10);
const COST_DEPOSIT_HISTORY: Cost = Cost::weight(1);
const COST_WITHDRAW: Cost = Cost::weight(600);
const COST_WITHDRAW_HISTORY: Cost = Cost::weight(18_000);
const COST_ACCOUNT_STATUS: Cost = Cost::weight(1);
const COST_TRADE_FEE: Cost = Cost::weight(1);
const COST_ASSET_DIVIDEND_RECORD: Cost = Cost::weight(10);
const COST_WALLET_BALANCE: Cost = Cost::weight(60);
const COST_UNIVERSAL_TRANSFER: Cost = Cost::weight(900);
const COST_UNIVERSAL_TRANSFER_HISTORY: Cost = Cost::weight(1);
const COST_USER_ASSET: Cost = Cost::weight(5);
const COST_ACCOUNT_API_TRADING_STATUS: Cost = Cost::weight(1);
const COST_SYSTEM_STATUS: Cost = Cost::weight(1);
const COST_FAST_WITHDRAW_SWITCH: Cost = Cost::weight(1);

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

// Capital — coin / network metadata, deposits
impl PrivateClient {
    /// List every coin the account can hold, with per-network deposit and
    /// withdraw configuration. Heavyweight (often >300 coins); cache it.
    pub async fn get_all_coins(
        &self,
        params: GetAllCoinsParams,
    ) -> Result<Response<Vec<CoinInfo>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::CapitalConfigGetAll,
            &params,
            COST_ALL_COINS,
        )
        .await
    }

    /// Fetch the deposit address for a coin (optionally on a specific network).
    pub async fn get_deposit_address(
        &self,
        params: GetDepositAddressParams,
    ) -> Result<Response<DepositAddress>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::CapitalDepositAddress,
            &params,
            COST_DEPOSIT_ADDRESS,
        )
        .await
    }

    /// Recent deposit history. Defaults: last 90 days, up to 1000 records.
    pub async fn get_deposit_history(
        &self,
        params: GetDepositHistoryParams,
    ) -> Result<Response<Vec<Deposit>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::CapitalDepositHistory,
            &params,
            COST_DEPOSIT_HISTORY,
        )
        .await
    }

    /// Recent withdraw history. Defaults: last 90 days, up to 1000 records.
    pub async fn get_withdraw_history(
        &self,
        params: GetWithdrawHistoryParams,
    ) -> Result<Response<Vec<Withdraw>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::CapitalWithdrawHistory,
            &params,
            COST_WITHDRAW_HISTORY,
        )
        .await
    }
}

// Account status
impl PrivateClient {
    /// Coarse account-wide status string (`"Normal"`, `"Margin Account dormant"`, …).
    pub async fn get_account_status(
        &self,
        params: GetAccountStatusParams,
    ) -> Result<Response<AccountStatus>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AccountStatus,
            &params,
            COST_ACCOUNT_STATUS,
        )
        .await
    }
}

// Asset
impl PrivateClient {
    /// Maker / taker commission rates per symbol. Omit `symbol` to fetch all.
    pub async fn get_trade_fee(
        &self,
        params: GetTradeFeeParams,
    ) -> Result<Response<Vec<TradeFee>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AssetTradeFee,
            &params,
            COST_TRADE_FEE,
        )
        .await
    }

    /// Asset dividend records (e.g. staking / airdrop distributions).
    pub async fn get_asset_dividend_record(
        &self,
        params: GetAssetDividendRecordParams,
    ) -> Result<Response<AssetDividendRecord>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AssetDividendRecord,
            &params,
            COST_ASSET_DIVIDEND_RECORD,
        )
        .await
    }

    /// Per-wallet balances (Spot, Funding, Cross Margin, …), each converted
    /// to BTC.
    pub async fn query_user_wallet_balance(
        &self,
        params: GetUserWalletBalanceParams,
    ) -> Result<Response<Vec<WalletBalance>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AssetWalletBalance,
            &params,
            COST_WALLET_BALANCE,
        )
        .await
    }

    /// Move `asset` between two account types (e.g. `MAIN_UMFUTURE`,
    /// `MAIN_MARGIN`, …). See [`crate::wallet::UniversalTransferType`].
    pub async fn user_universal_transfer(
        &self,
        params: UserUniversalTransferRequest,
    ) -> Result<Response<UniversalTransferResult>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::AssetTransfer,
            &params,
            COST_UNIVERSAL_TRANSFER,
        )
        .await
    }

    /// History of universal transfers previously submitted via
    /// [`Self::user_universal_transfer`].
    pub async fn query_user_universal_transfer_history(
        &self,
        params: GetUniversalTransferHistoryParams,
    ) -> Result<Response<UniversalTransferHistory>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AssetTransfer,
            &params,
            COST_UNIVERSAL_TRANSFER_HISTORY,
        )
        .await
    }

    /// User assets, optionally filtered to a single `asset` and optionally
    /// including a BTC valuation. Binance serves this endpoint over `POST`
    /// despite being a read.
    pub async fn user_asset(
        &self,
        params: GetUserAssetParams,
    ) -> Result<Response<Vec<UserAsset>>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::AssetUserAsset,
            &params,
            COST_USER_ASSET,
        )
        .await
    }
}

// Account
impl PrivateClient {
    /// Whether API trading is currently locked, and the counters that would
    /// trigger a lock.
    pub async fn account_api_trading_status(
        &self,
        params: GetAccountApiTradingStatusParams,
    ) -> Result<Response<AccountApiTradingStatus>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::GET,
            Path::AccountApiTradingStatus,
            &params,
            COST_ACCOUNT_API_TRADING_STATUS,
        )
        .await
    }

    pub async fn enable_fast_withdraw_switch(
        &self,
        params: FastWithdrawSwitchParams,
    ) -> Result<Response<Empty>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::AccountEnableFastWithdrawSwitch,
            &params,
            COST_FAST_WITHDRAW_SWITCH,
        )
        .await
    }

    pub async fn disable_fast_withdraw_switch(
        &self,
        params: FastWithdrawSwitchParams,
    ) -> Result<Response<Empty>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::AccountDisableFastWithdrawSwitch,
            &params,
            COST_FAST_WITHDRAW_SWITCH,
        )
        .await
    }
}

// Withdraw
impl PrivateClient {
    /// Submit a withdrawal. Irreversible once accepted — double-check
    /// `coin`, `network`, `address` and `amount` before calling this.
    pub async fn withdraw(
        &self,
        params: WithdrawRequest,
    ) -> Result<Response<WithdrawResult>, Error> {
        send_signed(
            &self.http,
            &self.api_secret,
            Method::POST,
            Path::CapitalWithdrawApply,
            &params,
            COST_WITHDRAW,
        )
        .await
    }
}

// System
impl PrivateClient {
    /// System status (`0` = normal, `1` = maintenance). Public — Binance
    /// doesn't require a signature — but wallet has no dedicated
    /// `PublicClient`, so it's exposed here for convenience; the request is
    /// sent unsigned.
    pub async fn system_status(&self) -> Result<Response<SystemStatusResult>, Error> {
        let req = self.http.request(Method::GET, Path::SystemStatus);
        decode(self.http.send_raw(req, COST_SYSTEM_STATUS).await)
    }
}

/// Thin pins of the shared `crate::http` primitives (see `src/http.rs`) onto
/// this product's own `ApiError`/`Error` types, so every endpoint above can
/// call `decode`/`send_signed` directly instead of hand-copying the
/// serialize → sign → request → send → decode sequence.
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
