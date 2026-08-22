use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    wallet::{DepositStatus, TransferStatus, UniversalTransferType, WithdrawStatus},
};

pub use crate::http::{ParsedHeaders as Headers, Response};

// ===== Coins / capital config =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAllCoinsParams {
    recv_window: Option<u64>,
}

impl GetAllCoinsParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

/// One entry of the `/sapi/v1/capital/config/getall` response — describes a
/// coin and the networks on which it can be deposited or withdrawn.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoinInfo {
    pub coin: String,
    pub name: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub freeze: Decimal,
    pub withdrawing: Decimal,
    pub ipoing: Decimal,
    pub ipoable: Decimal,
    pub storage: Decimal,
    pub deposit_all_enable: bool,
    pub withdraw_all_enable: bool,
    pub trading: bool,
    pub is_legal_money: bool,
    pub network_list: Vec<CoinNetwork>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoinNetwork {
    pub network: String,
    pub coin: String,
    pub name: String,
    pub deposit_enable: bool,
    pub withdraw_enable: bool,
    pub is_default: bool,
    pub min_confirm: u32,
    pub un_lock_confirm: u32,
    pub withdraw_fee: Decimal,
    pub withdraw_min: Decimal,
    pub withdraw_max: Decimal,
    pub deposit_dust: Option<Decimal>,
    pub special_tips: Option<String>,
    pub same_address: bool,
}

// ===== Deposit address =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetDepositAddressParams {
    coin: String,
    network: Option<String>,
    /// Optional pre-fill amount for the deposit address QR.
    amount: Option<Decimal>,
    recv_window: Option<u64>,
}

impl GetDepositAddressParams {
    pub fn new(coin: impl Into<String>) -> Self {
        Self {
            coin: coin.into(),
            network: None,
            amount: None,
            recv_window: None,
        }
    }

    pub fn network(mut self, value: impl Into<String>) -> Self {
        self.network = Some(value.into());
        self
    }

    pub fn amount(mut self, value: Decimal) -> Self {
        self.amount = Some(value);
        self
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddress {
    pub address: String,
    pub coin: String,
    /// Optional memo / destination tag (used by XRP, XLM, etc.).
    pub tag: String,
    pub url: String,
}

// ===== Deposit history =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetDepositHistoryParams {
    coin: Option<String>,
    status: Option<DepositStatus>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    /// Default 1000, max 1000.
    limit: Option<u32>,
    offset: Option<u32>,
    recv_window: Option<u64>,
}

impl GetDepositHistoryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn coin(mut self, value: impl Into<String>) -> Self {
        self.coin = Some(value.into());
        self
    }
    pub fn status(mut self, value: DepositStatus) -> Self {
        self.status = Some(value);
        self
    }
    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }
    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }
    pub fn limit(mut self, value: u32) -> Self {
        self.limit = Some(value);
        self
    }
    pub fn offset(mut self, value: u32) -> Self {
        self.offset = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Deposit {
    /// Binance-internal deposit id.
    pub id: String,
    pub amount: Decimal,
    pub coin: String,
    pub network: String,
    pub status: DepositStatus,
    pub address: String,
    pub address_tag: String,
    pub tx_id: String,
    pub insert_time: Timestamp,
    pub transfer_type: u8,
    pub confirm_times: String,
    pub unlock_confirm: u32,
    pub wallet_type: u8,
}

// ===== Withdraw history =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetWithdrawHistoryParams {
    coin: Option<String>,
    /// Server-side filter on withdraw status.
    status: Option<WithdrawStatus>,
    /// Internal request identifier the user supplied at withdraw time.
    withdraw_order_id: Option<String>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    /// Default 1000, max 1000.
    limit: Option<u32>,
    offset: Option<u32>,
    recv_window: Option<u64>,
}

impl GetWithdrawHistoryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn coin(mut self, value: impl Into<String>) -> Self {
        self.coin = Some(value.into());
        self
    }
    pub fn status(mut self, value: WithdrawStatus) -> Self {
        self.status = Some(value);
        self
    }
    pub fn withdraw_order_id(mut self, value: impl Into<String>) -> Self {
        self.withdraw_order_id = Some(value.into());
        self
    }
    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }
    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }
    pub fn limit(mut self, value: u32) -> Self {
        self.limit = Some(value);
        self
    }
    pub fn offset(mut self, value: u32) -> Self {
        self.offset = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Withdraw {
    pub id: String,
    pub amount: Decimal,
    pub transaction_fee: Decimal,
    pub coin: String,
    pub status: WithdrawStatus,
    pub address: String,
    pub tx_id: String,
    pub apply_time: String,
    pub network: String,
    pub transfer_type: u8,
    pub withdraw_order_id: Option<String>,
    pub info: Option<String>,
    pub confirm_no: Option<u32>,
    pub wallet_type: u8,
    pub tx_key: Option<String>,
    pub complete_time: Option<String>,
}

// ===== Account status =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountStatusParams {
    recv_window: Option<u64>,
}

impl GetAccountStatusParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AccountStatus {
    /// `"Normal"`, `"Margin Account dormant"`, etc.
    pub data: String,
}

// ===== Trade fee =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTradeFeeParams {
    symbol: Option<String>,
    recv_window: Option<u64>,
}

impl GetTradeFeeParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol(mut self, value: impl Into<String>) -> Self {
        self.symbol = Some(value.into());
        self
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TradeFee {
    pub symbol: String,
    pub maker_commission: Decimal,
    pub taker_commission: Decimal,
}

// ===== Withdraw =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawRequest {
    coin: String,
    address: String,
    amount: Decimal,
    withdraw_order_id: Option<String>,
    network: Option<String>,
    address_tag: Option<String>,
    /// `true` to return the network fee to the destination account, `false`
    /// (default) to return it to the departure account.
    transaction_fee_flag: Option<bool>,
    /// Description of the address, used on the withdrawal address list.
    name: Option<String>,
    /// 0 = spot wallet (default), 1 = funding wallet.
    wallet_type: Option<u8>,
    recv_window: Option<u64>,
}

impl WithdrawRequest {
    pub fn new(coin: impl Into<String>, address: impl Into<String>, amount: Decimal) -> Self {
        Self {
            coin: coin.into(),
            address: address.into(),
            amount,
            withdraw_order_id: None,
            network: None,
            address_tag: None,
            transaction_fee_flag: None,
            name: None,
            wallet_type: None,
            recv_window: None,
        }
    }

    pub fn withdraw_order_id(mut self, value: impl Into<String>) -> Self {
        self.withdraw_order_id = Some(value.into());
        self
    }
    pub fn network(mut self, value: impl Into<String>) -> Self {
        self.network = Some(value.into());
        self
    }
    pub fn address_tag(mut self, value: impl Into<String>) -> Self {
        self.address_tag = Some(value.into());
        self
    }
    pub fn transaction_fee_flag(mut self, value: bool) -> Self {
        self.transaction_fee_flag = Some(value);
        self
    }
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }
    pub fn wallet_type(mut self, value: u8) -> Self {
        self.wallet_type = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct WithdrawResult {
    /// Binance-internal withdrawal id, usable to look the record up via
    /// `get_withdraw_history`.
    pub id: String,
}

// ===== Asset dividend record =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetDividendRecordParams {
    asset: Option<String>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    /// Default 20, max 500.
    limit: Option<u32>,
    recv_window: Option<u64>,
}

impl GetAssetDividendRecordParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn asset(mut self, value: impl Into<String>) -> Self {
        self.asset = Some(value.into());
        self
    }
    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }
    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }
    pub fn limit(mut self, value: u32) -> Self {
        self.limit = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AssetDividendRecord {
    pub rows: Vec<AssetDividend>,
    pub total: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetDividend {
    pub id: u64,
    pub amount: Decimal,
    pub asset: String,
    pub div_time: Timestamp,
    pub en_info: String,
    pub tran_id: u64,
}

// ===== Wallet balance =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetUserWalletBalanceParams {
    recv_window: Option<u64>,
}

impl GetUserWalletBalanceParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    pub activate: bool,
    pub balance: Decimal,
    pub wallet_name: String,
}

// ===== Universal transfer =====

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserUniversalTransferRequest {
    #[serde(rename = "type")]
    transfer_type: UniversalTransferType,
    asset: String,
    amount: Decimal,
    /// Required only for isolated-margin transfers.
    from_symbol: Option<String>,
    /// Required only for isolated-margin transfers.
    to_symbol: Option<String>,
    recv_window: Option<u64>,
}

impl UserUniversalTransferRequest {
    pub fn new(
        transfer_type: UniversalTransferType,
        asset: impl Into<String>,
        amount: Decimal,
    ) -> Self {
        Self {
            transfer_type,
            asset: asset.into(),
            amount,
            from_symbol: None,
            to_symbol: None,
            recv_window: None,
        }
    }

    pub fn from_symbol(mut self, value: impl Into<String>) -> Self {
        self.from_symbol = Some(value.into());
        self
    }
    pub fn to_symbol(mut self, value: impl Into<String>) -> Self {
        self.to_symbol = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UniversalTransferResult {
    pub tran_id: u64,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetUniversalTransferHistoryParams {
    #[serde(rename = "type")]
    transfer_type: UniversalTransferType,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    /// Default 1.
    current: Option<u32>,
    /// Default 10, max 100.
    size: Option<u32>,
    from_symbol: Option<String>,
    to_symbol: Option<String>,
    recv_window: Option<u64>,
}

impl GetUniversalTransferHistoryParams {
    pub fn new(transfer_type: UniversalTransferType) -> Self {
        Self {
            transfer_type,
            start_time: None,
            end_time: None,
            current: None,
            size: None,
            from_symbol: None,
            to_symbol: None,
            recv_window: None,
        }
    }

    pub fn start_time(mut self, value: Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }
    pub fn end_time(mut self, value: Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }
    pub fn current(mut self, value: u32) -> Self {
        self.current = Some(value);
        self
    }
    pub fn size(mut self, value: u32) -> Self {
        self.size = Some(value);
        self
    }
    pub fn from_symbol(mut self, value: impl Into<String>) -> Self {
        self.from_symbol = Some(value.into());
        self
    }
    pub fn to_symbol(mut self, value: impl Into<String>) -> Self {
        self.to_symbol = Some(value.into());
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UniversalTransferHistory {
    pub total: u64,
    pub rows: Vec<UniversalTransfer>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UniversalTransfer {
    pub asset: String,
    pub amount: Decimal,
    #[serde(rename = "type")]
    pub transfer_type: UniversalTransferType,
    pub status: TransferStatus,
    pub tran_id: u64,
    pub timestamp: Timestamp,
}

// ===== User asset (v3) =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetUserAssetParams {
    asset: Option<String>,
    need_btc_valuation: Option<bool>,
    recv_window: Option<u64>,
}

impl GetUserAssetParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn asset(mut self, value: impl Into<String>) -> Self {
        self.asset = Some(value.into());
        self
    }
    pub fn need_btc_valuation(mut self, value: bool) -> Self {
        self.need_btc_valuation = Some(value);
        self
    }
    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAsset {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub freeze: Decimal,
    pub withdrawing: Decimal,
    pub ipoable: Decimal,
    pub btc_valuation: Decimal,
}

// ===== Account API trading status =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountApiTradingStatusParams {
    recv_window: Option<u64>,
}

impl GetAccountApiTradingStatusParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AccountApiTradingStatus {
    pub data: AccountApiTradingStatusData,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountApiTradingStatusData {
    pub is_locked: bool,
    pub planned_recover_time: Timestamp,
    pub trigger_condition: TriggerCondition,
    pub update_time: Timestamp,
}

/// Thresholds that would trigger an API trading lock, expressed as counts
/// over the evaluation window Binance uses internally.
#[derive(Debug, Deserialize, PartialEq)]
pub struct TriggerCondition {
    /// GTC cancellation ratio trigger count.
    #[serde(rename = "GCR")]
    pub gcr: u32,
    /// IOC/FOK expiration ratio trigger count.
    #[serde(rename = "IFER")]
    pub ifer: u32,
    /// Unfilled ratio trigger count.
    #[serde(rename = "UFR")]
    pub ufr: u32,
}

// ===== System status =====

#[derive(Debug, Deserialize, PartialEq)]
pub struct SystemStatusResult {
    /// `0` = normal, `1` = system maintenance.
    pub status: u8,
    pub msg: String,
}

// ===== Fast withdraw switch =====

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FastWithdrawSwitchParams {
    recv_window: Option<u64>,
}

impl FastWithdrawSwitchParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv_window(mut self, value: u64) -> Self {
        self.recv_window = Some(value);
        self
    }
}

/// Marker type for endpoints that reply with an empty JSON object (`{}`).
#[derive(Debug, Deserialize, PartialEq)]
pub struct Empty {}
