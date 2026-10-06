//! Enum definitions for the Binance Wallet API.

// Wire models: names mirror Binance's documentation field by field; they are
// documented where the meaning isn't obvious. Full coverage comes later.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// Implements the wire format of a numeric status enum by hand.
///
/// `serde_repr` cannot fall back to a catch-all variant, so a status code
/// Binance adds later would fail the whole history response. Instead the
/// code is read as a plain integer: known codes map to their variant, any
/// other code to `Unknown(code)`. Serialization writes the integer back, so
/// `Unknown(code)` can also be used to filter by a code this SDK doesn't
/// name yet.
macro_rules! numeric_status {
    ($name:ident { $($variant:ident = $code:literal),+ $(,)? }) => {
        impl $name {
            /// The numeric code Binance uses for this status.
            pub const fn code(self) -> i64 {
                match self {
                    $(Self::$variant => $code,)+
                    Self::Unknown(code) => code,
                }
            }

            /// The status for a numeric code; unknown codes are preserved.
            pub const fn from_code(code: i64) -> Self {
                match code {
                    $($code => Self::$variant,)+
                    other => Self::Unknown(other),
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_i64(self.code())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                i64::deserialize(deserializer).map(Self::from_code)
            }
        }
    };
}

/// Deposit status reported by the deposit-history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositStatus {
    Pending,
    /// Credited but cannot withdraw yet.
    CreditedButCannotWithdraw,
    /// Wrong deposit (e.g. wrong address / chain).
    WrongDeposit,
    /// Waiting for user confirmation.
    WaitingUserConfirm,
    Success,
    /// A status code this SDK version doesn't know yet.
    Unknown(i64),
}

numeric_status!(DepositStatus {
    Pending = 0,
    Success = 1,
    CreditedButCannotWithdraw = 6,
    WrongDeposit = 7,
    WaitingUserConfirm = 8,
});

/// Withdraw status reported by the withdraw-history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawStatus {
    EmailSent,
    Cancelled,
    AwaitingApproval,
    Rejected,
    Processing,
    Failure,
    Completed,
    /// A status code this SDK version doesn't know yet.
    Unknown(i64),
}

numeric_status!(WithdrawStatus {
    EmailSent = 0,
    Cancelled = 1,
    AwaitingApproval = 2,
    Rejected = 3,
    Processing = 4,
    Failure = 5,
    Completed = 6,
});

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
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Status of a universal-transfer record reported by the transfer-history
/// endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferStatus {
    Confirmed,
    Failed,
    Pending,
    /// A value this SDK version doesn't know yet. Keeps the response
    /// deserializable when Binance adds a new value; never sent in requests.
    #[serde(other, skip_serializing)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_status_round_trips_known_and_unknown_codes() {
        let parsed: Vec<DepositStatus> = serde_json::from_str("[0, 1, 6, 7, 8, 42]").unwrap();
        assert_eq!(
            parsed,
            vec![
                DepositStatus::Pending,
                DepositStatus::Success,
                DepositStatus::CreditedButCannotWithdraw,
                DepositStatus::WrongDeposit,
                DepositStatus::WaitingUserConfirm,
                DepositStatus::Unknown(42),
            ]
        );
        let parsed: WithdrawStatus = serde_json::from_str("6").unwrap();
        assert_eq!(parsed, WithdrawStatus::Completed);
        let parsed: WithdrawStatus = serde_json::from_str("9").unwrap();
        assert_eq!(parsed, WithdrawStatus::Unknown(9));

        assert_eq!(
            serde_json::to_string(&WithdrawStatus::Rejected).unwrap(),
            "3"
        );
        assert_eq!(
            serde_json::to_string(&DepositStatus::Unknown(42)).unwrap(),
            "42"
        );
        assert!(serde_json::from_str::<DepositStatus>(r#""1""#).is_err());
    }

    #[test]
    fn numeric_status_serializes_as_query_parameter() {
        #[derive(Serialize)]
        struct Q {
            status: Option<DepositStatus>,
        }
        let q = Q {
            status: Some(DepositStatus::Success),
        };
        assert_eq!(crate::serde::serialize_query(&q).unwrap(), "status=1");
    }
}
