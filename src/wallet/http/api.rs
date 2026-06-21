use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Timestamp,
    wallet::{DepositStatus, WithdrawStatus},
};

#[derive(Debug, PartialEq)]
pub struct Response<T> {
    pub result: T,
    pub headers: Headers,
}

#[derive(Debug, PartialEq)]
pub struct Headers {
    pub retry_after: Option<Timestamp>,
}

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
