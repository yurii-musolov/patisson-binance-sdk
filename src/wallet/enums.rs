//! Enum definitions for the Binance Wallet API.

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Deposit status reported by the deposit-history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum DepositStatus {
    Pending = 0,
    /// Credited but cannot withdraw yet.
    CreditedButCannotWithdraw = 6,
    /// Wrong deposit (e.g. wrong address / chain).
    WrongDeposit = 7,
    /// Waiting for user confirmation.
    WaitingUserConfirm = 8,
    Success = 1,
}

/// Withdraw status reported by the withdraw-history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum WithdrawStatus {
    EmailSent = 0,
    Cancelled = 1,
    AwaitingApproval = 2,
    Rejected = 3,
    Processing = 4,
    Failure = 5,
    Completed = 6,
}

/// Account / wallet a universal transfer operates on.
///
/// Used as the `type` parameter on `/sapi/v1/asset/transfer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UniversalTransferType {
    MainUmfuture,
    MainCmfuture,
    MainMargin,
    UmfutureMain,
    UmfutureMargin,
    CmfutureMain,
    CmfutureMargin,
    MarginMain,
    MarginUmfuture,
    MarginCmfuture,
    IsolatedmarginMargin,
    MarginIsolatedmargin,
    IsolatedmarginIsolatedmargin,
    MainFunding,
    FundingMain,
    FundingUmfuture,
    UmfutureFunding,
    MarginFunding,
    FundingMargin,
    FundingCmfuture,
    CmfutureFunding,
}
