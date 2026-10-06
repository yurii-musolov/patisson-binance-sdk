//! Events pushed over the USDⓈ-M Futures user data stream
//! (`wss://fstream.binance.com/private/ws/<listenKey>`, see
//! [`Path::Private`](crate::derivatives::usds_margined_futures::Path::Private)).
//!
//! Field meanings follow the official "User Data Streams" event pages; the
//! one-letter wire names are kept in `#[serde(rename)]`.

// Wire models: names mirror Binance's documentation field by field; they are
// documented where the meaning isn't obvious. Full coverage comes later.
#![allow(missing_docs)]

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    Timestamp,
    derivatives::usds_margined_futures::{
        AlgoType, MarginType, OrderSide, OrderStatus, OrderType, PositionSide, PriceMatch, STPMode,
        TimeInForce, WorkingType,
    },
    serde::{decimal_opt_lenient, string_opt_lenient},
    ws::ReceivedMessage,
};

/// A frame of the user data stream, tagged by the `e` (event type) field.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "e")]
#[allow(clippy::large_enum_variant)]
pub enum UserDataMessage {
    /// An order was created, changed, filled, canceled or expired.
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    OrderTradeUpdate(OrderTradeUpdateEvent),
    /// A shorter, lower-latency trade report for each fill.
    #[serde(rename = "TRADE_LITE")]
    TradeLite(TradeLiteEvent),
    /// Balances and positions changed.
    #[serde(rename = "ACCOUNT_UPDATE")]
    AccountUpdate(AccountUpdateEvent),
    /// Cross positions are close to liquidation.
    #[serde(rename = "MARGIN_CALL")]
    MarginCall(MarginCallEvent),
    /// Leverage or Multi-Assets mode changed.
    #[serde(rename = "ACCOUNT_CONFIG_UPDATE")]
    AccountConfigUpdate(AccountConfigUpdateEvent),
    /// An algo (conditional) order changed; see the Algo service endpoints.
    #[serde(rename = "ALGO_UPDATE")]
    AlgoUpdate(AlgoUpdateEvent),
    /// A triggered conditional order was rejected.
    #[serde(rename = "CONDITIONAL_ORDER_TRIGGER_REJECT")]
    ConditionalOrderTriggerReject(ConditionalOrderTriggerRejectEvent),
    /// A trading strategy changed state.
    #[serde(rename = "STRATEGY_UPDATE")]
    StrategyUpdate(StrategyUpdateEvent),
    /// A grid strategy's position changed.
    #[serde(rename = "GRID_UPDATE")]
    GridUpdate(GridUpdateEvent),
    /// The listenKey expired: create a new one and reconnect.
    #[serde(rename = "listenKeyExpired")]
    ListenKeyExpired(ListenKeyExpiredEvent),
    /// An event type this SDK version doesn't model yet.
    #[serde(other)]
    Unknown,
}

impl ReceivedMessage for UserDataMessage {
    fn server_shutdown_event_time(&self) -> Option<u64> {
        None
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct OrderTradeUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "o")]
    pub order: OrderUpdate,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct OrderUpdate {
    #[serde(rename = "s")]
    pub symbol: String,
    /// Client order id. `autoclose-...` marks a liquidation order,
    /// `adl_autoclose` an ADL order, `settlement_autoclose-...` a
    /// delisting/delivery settlement order.
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "o")]
    pub order_type: OrderType,
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "q")]
    pub original_quantity: Decimal,
    #[serde(rename = "p")]
    pub original_price: Decimal,
    #[serde(rename = "ap")]
    pub average_price: Decimal,
    /// Ignore for `TRAILING_STOP_MARKET`.
    #[serde(rename = "sp")]
    pub stop_price: Decimal,
    #[serde(rename = "x")]
    pub execution_type: ExecutionType,
    #[serde(rename = "X")]
    pub order_status: OrderStatus,
    #[serde(rename = "i")]
    pub order_id: u64,
    /// Only on `AMENDMENT` events when the request carried a modify id.
    #[serde(rename = "M", default)]
    pub modify_id: Option<String>,
    #[serde(rename = "l")]
    pub last_filled_quantity: Decimal,
    #[serde(rename = "z")]
    pub cumulative_filled_quantity: Decimal,
    #[serde(rename = "L")]
    pub last_filled_price: Decimal,
    /// Absent when there is no commission.
    #[serde(rename = "N", default)]
    pub commission_asset: Option<String>,
    #[serde(rename = "n", default, deserialize_with = "decimal_opt_lenient")]
    pub commission: Option<Decimal>,
    #[serde(rename = "T")]
    pub trade_time: Timestamp,
    #[serde(rename = "t")]
    pub trade_id: u64,
    #[serde(rename = "b")]
    pub bids_notional: Decimal,
    #[serde(rename = "a")]
    pub ask_notional: Decimal,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "R")]
    pub is_reduce_only: bool,
    #[serde(rename = "wt")]
    pub working_type: WorkingType,
    #[serde(rename = "ot")]
    pub original_order_type: OrderType,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
    /// Close-all; pushed with conditional orders.
    #[serde(rename = "cp", default)]
    pub close_position: bool,
    /// Only for `TRAILING_STOP_MARKET`.
    #[serde(rename = "AP", default, deserialize_with = "decimal_opt_lenient")]
    pub activation_price: Option<Decimal>,
    /// Only for `TRAILING_STOP_MARKET`.
    #[serde(rename = "cr", default, deserialize_with = "decimal_opt_lenient")]
    pub callback_rate: Option<Decimal>,
    #[serde(rename = "pP", default)]
    pub price_protect: bool,
    /// Documented as "ignore".
    #[serde(rename = "si", default)]
    pub si: Option<i64>,
    /// Documented as "ignore".
    #[serde(rename = "ss", default)]
    pub ss: Option<i64>,
    #[serde(rename = "rp")]
    pub realized_profit: Decimal,
    #[serde(rename = "V", default)]
    pub self_trade_prevention_mode: Option<STPMode>,
    #[serde(rename = "pm", default)]
    pub price_match: Option<PriceMatch>,
    /// Auto-cancel time of a `GTD` order (`0` otherwise).
    #[serde(rename = "gtd", default)]
    pub good_till_date: Option<Timestamp>,
    /// Expiry reason code (`0` = none); see the event documentation.
    #[serde(rename = "er", default, deserialize_with = "string_opt_lenient")]
    pub expiry_reason: Option<String>,
}

/// Execution type (`x`) of an order update.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionType {
    New,
    Canceled,
    /// Liquidation execution.
    Calculated,
    Expired,
    Trade,
    /// The order was modified.
    Amendment,
    /// A value this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TradeLiteEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "q")]
    pub original_quantity: Decimal,
    #[serde(rename = "p")]
    pub original_price: Decimal,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "L")]
    pub last_filled_price: Decimal,
    #[serde(rename = "l")]
    pub last_filled_quantity: Decimal,
    #[serde(rename = "t")]
    pub trade_id: u64,
    #[serde(rename = "i")]
    pub order_id: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AccountUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "a")]
    pub update: AccountUpdate,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AccountUpdate {
    #[serde(rename = "m")]
    pub reason: AccountUpdateReason,
    /// Assets whose balance changed.
    #[serde(rename = "B")]
    pub balances: Vec<BalanceUpdate>,
    /// Positions that changed (for `FUNDING_FEE` in a crossed position only
    /// the balance is pushed).
    #[serde(rename = "P", default)]
    pub positions: Vec<PositionUpdate>,
    /// Symbol, when the update concerns a single symbol.
    #[serde(rename = "S", default)]
    pub symbol: Option<String>,
}

/// Why an `ACCOUNT_UPDATE` was pushed (`m`).
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountUpdateReason {
    Deposit,
    Withdraw,
    Order,
    FundingFee,
    WithdrawReject,
    Adjustment,
    InsuranceClear,
    AdminDeposit,
    AdminWithdraw,
    MarginTransfer,
    MarginTypeChange,
    AssetTransfer,
    OptionsPremiumFee,
    OptionsSettleProfit,
    AutoExchange,
    CoinSwapDeposit,
    CoinSwapWithdraw,
    /// A value this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct BalanceUpdate {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "wb")]
    pub wallet_balance: Decimal,
    #[serde(rename = "cw")]
    pub cross_wallet_balance: Decimal,
    /// Balance change except PnL and commission.
    #[serde(rename = "bc")]
    pub balance_change: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PositionUpdate {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    #[serde(rename = "ep")]
    pub entry_price: Decimal,
    #[serde(rename = "bep")]
    pub breakeven_price: Decimal,
    /// Accumulated realized PnL (pre-fee).
    #[serde(rename = "cr")]
    pub accumulated_realized: Decimal,
    #[serde(rename = "up")]
    pub unrealized_pnl: Decimal,
    #[serde(rename = "mt")]
    pub margin_type: MarginType,
    /// Isolated wallet (if isolated position).
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MarginCallEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    /// Only pushed with crossed positions.
    #[serde(rename = "cw", default, deserialize_with = "decimal_opt_lenient")]
    pub cross_wallet_balance: Option<Decimal>,
    #[serde(rename = "p")]
    pub positions: Vec<MarginCallPosition>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MarginCallPosition {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    #[serde(rename = "mt")]
    pub margin_type: MarginType,
    /// Isolated wallet (if isolated position).
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    #[serde(rename = "mp")]
    pub mark_price: Decimal,
    #[serde(rename = "up")]
    pub unrealized_pnl: Decimal,
    /// Maintenance margin required.
    #[serde(rename = "mm")]
    pub maintenance_margin: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AccountConfigUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    /// Leverage of a symbol changed.
    #[serde(rename = "ac", default)]
    pub leverage: Option<LeverageUpdate>,
    /// Multi-Assets mode changed.
    #[serde(rename = "ai", default)]
    pub multi_assets: Option<MultiAssetsUpdate>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LeverageUpdate {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "l")]
    pub leverage: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MultiAssetsUpdate {
    #[serde(rename = "j")]
    pub multi_assets_mode: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AlgoUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "o")]
    pub order: AlgoOrderUpdate,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AlgoOrderUpdate {
    #[serde(rename = "caid")]
    pub client_algo_id: String,
    #[serde(rename = "aid")]
    pub algo_id: u64,
    #[serde(rename = "at")]
    pub algo_type: AlgoType,
    #[serde(rename = "o")]
    pub order_type: OrderType,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    /// e.g. `NEW`, `CANCELED`, `TRIGGERING`, `TRIGGERED`, `FINISHED`,
    /// `REJECTED`, `EXPIRED`.
    #[serde(rename = "X")]
    pub algo_status: String,
    /// Id of the order placed in the matching engine once triggered.
    #[serde(rename = "ai", default, deserialize_with = "string_opt_lenient")]
    pub actual_order_id: Option<String>,
    /// Average fill price in the matching engine.
    #[serde(rename = "ap", default, deserialize_with = "decimal_opt_lenient")]
    pub actual_price: Option<Decimal>,
    /// Executed quantity in the matching engine.
    #[serde(rename = "aq", default, deserialize_with = "decimal_opt_lenient")]
    pub actual_quantity: Option<Decimal>,
    /// Actual order type in the matching engine.
    #[serde(rename = "act", default, deserialize_with = "string_opt_lenient")]
    pub actual_order_type: Option<String>,
    #[serde(rename = "tp")]
    pub trigger_price: Decimal,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "V")]
    pub self_trade_prevention_mode: STPMode,
    #[serde(rename = "wt")]
    pub working_type: WorkingType,
    #[serde(rename = "pm")]
    pub price_match: PriceMatch,
    #[serde(rename = "cp")]
    pub close_position: bool,
    #[serde(rename = "pP")]
    pub price_protect: bool,
    #[serde(rename = "R")]
    pub reduce_only: bool,
    /// Trigger time (`0` until triggered).
    #[serde(rename = "tt")]
    pub trigger_time: Timestamp,
    #[serde(rename = "gtd")]
    pub good_till_date: Timestamp,
    /// Failure reason, if any.
    #[serde(rename = "rm", default, deserialize_with = "string_opt_lenient")]
    pub reason: Option<String>,
    /// Iceberg algo order.
    #[serde(rename = "ia", default)]
    pub is_iceberg: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConditionalOrderTriggerRejectEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub message_send_time: Timestamp,
    #[serde(rename = "or")]
    pub order: RejectedOrder,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RejectedOrder {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "r")]
    pub reject_reason: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct StrategyUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "su")]
    pub update: StrategyUpdate,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct StrategyUpdate {
    #[serde(rename = "si")]
    pub strategy_id: u64,
    /// e.g. `GRID`.
    #[serde(rename = "st")]
    pub strategy_type: String,
    /// e.g. `NEW`, `WORKING`, `CANCELLED`, `EXPIRED`.
    #[serde(rename = "ss")]
    pub strategy_status: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "ut")]
    pub update_time: Timestamp,
    /// Operation code; see the event documentation.
    #[serde(rename = "c")]
    pub op_code: i64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct GridUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: Timestamp,
    #[serde(rename = "T")]
    pub transaction_time: Timestamp,
    #[serde(rename = "gu")]
    pub update: GridUpdate,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct GridUpdate {
    #[serde(rename = "si")]
    pub strategy_id: u64,
    #[serde(rename = "st")]
    pub strategy_type: String,
    #[serde(rename = "ss")]
    pub strategy_status: String,
    #[serde(rename = "s")]
    pub symbol: String,
    /// Realized PnL.
    #[serde(rename = "r")]
    pub realized_pnl: Decimal,
    /// Unmatched average price.
    #[serde(rename = "up")]
    pub unmatched_average_price: Decimal,
    /// Unmatched quantity.
    #[serde(rename = "uq")]
    pub unmatched_quantity: Decimal,
    /// Unmatched fee.
    #[serde(rename = "uf")]
    pub unmatched_fee: Decimal,
    /// Matched PnL.
    #[serde(rename = "mp")]
    pub matched_pnl: Decimal,
    #[serde(rename = "ut")]
    pub update_time: Timestamp,
}

/// `Debug` redacts the listenKey: it grants read access to the account's
/// events and must not end up in logs.
#[derive(Deserialize, PartialEq)]
pub struct ListenKeyExpiredEvent {
    #[serde(
        rename = "E",
        deserialize_with = "crate::serde::u64_from_number_or_string"
    )]
    pub event_time: Timestamp,
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

impl std::fmt::Debug for ListenKeyExpiredEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListenKeyExpiredEvent")
            .field("event_time", &self.event_time)
            .field("listen_key", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_event_types_do_not_fail() {
        let msg: UserDataMessage =
            serde_json::from_str(r#"{"e":"SOMETHING_NEW","E":1,"x":{}}"#).unwrap();
        assert_eq!(msg, UserDataMessage::Unknown);
    }

    #[test]
    fn listen_key_expired_accepts_a_string_event_time() {
        let msg: UserDataMessage =
            serde_json::from_str(r#"{"e":"listenKeyExpired","E":"1","listenKey":"k"}"#).unwrap();
        assert!(matches!(&msg, UserDataMessage::ListenKeyExpired(e) if e.event_time == 1));
    }

    #[test]
    fn listen_key_is_redacted_in_debug() {
        let msg: UserDataMessage =
            serde_json::from_str(r#"{"e":"listenKeyExpired","E":1,"listenKey":"secret-key"}"#)
                .unwrap();
        assert!(!format!("{msg:?}").contains("secret-key"));
    }
}
