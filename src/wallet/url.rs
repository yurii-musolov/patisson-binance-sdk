//! Binance Wallet endpoints.
//!
//! Wallet shares the spot REST host (`api.binance.com`). All wallet routes
//! live under `/sapi/v1/{capital,account,asset}/...` and require an API key
//! plus signature. For market data and connectivity probes, use
//! [`crate::spot::http::PublicClient`].

// Mainnet
pub const BASE_URL_API: &str = "https://api.binance.com";

// Testnet — note that the spot testnet does NOT expose the wallet `/sapi/v1`
// endpoints. Live wallet calls only work against mainnet.
pub const BASE_URL_TESTNET_API: &str = "https://testnet.binance.vision";

pub enum Path {
    // Capital — deposits / withdrawals
    CapitalConfigGetAll,
    CapitalDepositAddress,
    CapitalDepositHistory,
    CapitalWithdrawApply,
    CapitalWithdrawHistory,

    // Account
    AccountStatus,
    AccountApiTradingStatus,
    AccountEnableFastWithdrawSwitch,
    AccountDisableFastWithdrawSwitch,

    // Asset
    AssetDividendRecord,
    AssetTradeFee,
    AssetTransfer,
    AssetUserAsset,
    AssetWalletBalance,

    // Others
    SystemStatus,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::CapitalConfigGetAll => "/sapi/v1/capital/config/getall",
            Self::CapitalDepositAddress => "/sapi/v1/capital/deposit/address",
            Self::CapitalDepositHistory => "/sapi/v1/capital/deposit/hisrec",
            Self::CapitalWithdrawApply => "/sapi/v1/capital/withdraw/apply",
            Self::CapitalWithdrawHistory => "/sapi/v1/capital/withdraw/history",
            Self::AccountStatus => "/sapi/v1/account/status",
            Self::AccountApiTradingStatus => "/sapi/v1/account/apiTradingStatus",
            Self::AccountEnableFastWithdrawSwitch => "/sapi/v1/account/enableFastWithdrawSwitch",
            Self::AccountDisableFastWithdrawSwitch => "/sapi/v1/account/disableFastWithdrawSwitch",
            Self::AssetDividendRecord => "/sapi/v1/asset/assetDividend",
            Self::AssetTradeFee => "/sapi/v1/asset/tradeFee",
            Self::AssetTransfer => "/sapi/v1/asset/transfer",
            Self::AssetUserAsset => "/sapi/v3/asset/getUserAsset",
            Self::AssetWalletBalance => "/sapi/v1/asset/wallet/balance",
            Self::SystemStatus => "/sapi/v1/system/status",
        };
        write!(f, "{s}")
    }
}

pub const HEADER_X_MBX_APIKEY: &str = "X-MBX-APIKEY";
