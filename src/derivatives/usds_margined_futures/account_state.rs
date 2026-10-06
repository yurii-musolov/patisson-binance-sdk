//! USDⓈ-M Futures positions, balances, open orders and open algo orders,
//! kept current from the user data stream.
//!
//! Load a snapshot over REST ([`AccountState::load`]) and apply every user
//! data message ([`AccountState::apply`]); after a reconnect or a `Lagged`
//! gap, [`AccountState::reload`]. Updates older than the data they would
//! replace are ignored:
//!
//! - balances and positions carry the time of their last update;
//!   `ACCOUNT_UPDATE` applies only when its transaction time is not older;
//! - a closed position stays known (with amount 0), so a late update can't
//!   bring it back;
//! - orders go through [`crate::Orders`] (fills counted once, final orders
//!   never reopen); algo orders are dropped once `CANCELED`, `FINISHED`,
//!   `REJECTED` or `EXPIRED`.

use std::collections::{BTreeMap, HashMap};

use rust_decimal::Decimal;

use crate::{
    OrderEvent, OrderState, OrderStatus, Timestamp,
    derivatives::usds_margined_futures::{
        Error, MarginType, PositionSide,
        http::{
            AlgoOrder, GetAccountInformationParams, GetOpenAlgoOrdersParams, GetOpenOrdersParams,
            GetPositionInformationParams, Order, PrivateClient,
        },
        ws::{AccountUpdateEvent, AlgoUpdateEvent, UserDataMessage},
    },
    order_state::OpenOrders,
};

/// Wallet balance of one margin asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalletBalance {
    pub wallet_balance: Decimal,
    pub cross_wallet_balance: Decimal,
    /// Time of the last update applied.
    pub updated_at: Timestamp,
}

/// One position (per symbol and position side).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionState {
    pub symbol: String,
    pub position_side: PositionSide,
    /// Signed: negative for a short in one-way mode. `0` once closed.
    pub amount: Decimal,
    pub entry_price: Decimal,
    pub breakeven_price: Option<Decimal>,
    pub unrealized_pnl: Decimal,
    /// Unknown until the first `ACCOUNT_UPDATE` (the v3 position endpoint
    /// doesn't report it).
    pub margin_type: Option<MarginType>,
    pub isolated_wallet: Decimal,
    /// Time of the last update applied.
    pub updated_at: Timestamp,
}

impl PositionState {
    pub fn is_open(&self) -> bool {
        !self.amount.is_zero()
    }
}

/// An open algo (conditional) order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgoOrderState {
    pub algo_id: u64,
    pub client_algo_id: String,
    pub symbol: String,
    pub quantity: Decimal,
    pub trigger_price: Decimal,
    pub price: Decimal,
    /// e.g. `NEW`, `TRIGGERING`, `TRIGGERED`, `FINISHED`, `CANCELED`.
    pub status: String,
    pub updated_at: Timestamp,
}

impl AlgoOrderState {
    /// The algo order can no longer change.
    pub fn is_final(&self) -> bool {
        matches!(
            self.status.as_str(),
            "CANCELED" | "FINISHED" | "REJECTED" | "EXPIRED"
        )
    }
}

/// What a message changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountChange {
    Balance {
        asset: String,
        balance: WalletBalance,
    },
    /// New state of a position (`amount` 0: closed).
    Position(PositionState),
    /// New state of an order; final orders are reported once, then dropped.
    Order(OrderState),
    /// New state of an algo order; final ones are reported once, then
    /// dropped.
    AlgoOrder(AlgoOrderState),
}

/// USDⓈ-M Futures account: balances, positions, open orders and algo orders.
#[derive(Debug, Clone, Default)]
pub struct AccountState {
    balances: BTreeMap<String, WalletBalance>,
    positions: HashMap<(String, PositionSide), PositionState>,
    orders: OpenOrders,
    algo_orders: BTreeMap<u64, AlgoOrderState>,
}

impl AccountState {
    /// Snapshot: `GET /fapi/v3/account`, `/fapi/v3/positionRisk`,
    /// `/fapi/v1/openOrders` and `/fapi/v1/openAlgoOrders` (all symbols).
    pub async fn load(client: &PrivateClient) -> Result<Self, Error> {
        let account = client
            .account_information(GetAccountInformationParams::new())
            .await?
            .result;
        let positions = client
            .position_information(GetPositionInformationParams::new())
            .await?
            .result;
        let open_orders = client
            .get_open_orders(GetOpenOrdersParams::new())
            .await?
            .result;
        let algo_orders = client
            .get_open_algo_orders(GetOpenAlgoOrdersParams::new())
            .await?
            .result;

        let mut state = Self::default();
        for a in account.assets {
            state.balances.insert(
                a.asset,
                WalletBalance {
                    wallet_balance: a.wallet_balance,
                    cross_wallet_balance: a.cross_wallet_balance,
                    updated_at: a.update_time,
                },
            );
        }
        for p in positions {
            state.positions.insert(
                (p.symbol.clone(), p.position_side),
                PositionState {
                    symbol: p.symbol,
                    position_side: p.position_side,
                    amount: p.position_amt,
                    entry_price: p.entry_price,
                    breakeven_price: p.break_even_price,
                    unrealized_pnl: p.un_realized_profit,
                    margin_type: None,
                    isolated_wallet: p.isolated_wallet,
                    updated_at: p.update_time,
                },
            );
        }
        for order in &open_orders {
            state.orders.apply(order);
        }
        for algo in &algo_orders {
            let algo = AlgoOrderState::from(algo);
            if !algo.is_final() {
                state.algo_orders.insert(algo.algo_id, algo);
            }
        }
        Ok(state)
    }

    /// Replace the state with a fresh snapshot (after a reconnect or a
    /// `Lagged` gap). Finished orders stay remembered.
    pub async fn reload(&mut self, client: &PrivateClient) -> Result<(), Error> {
        let fresh = Self::load(client).await?;
        self.balances = fresh.balances;
        self.positions = fresh.positions;
        self.algo_orders = fresh.algo_orders;
        self.orders.replace_with(fresh.orders);
        Ok(())
    }

    pub fn balance(&self, asset: &str) -> Option<&WalletBalance> {
        self.balances.get(asset)
    }

    pub fn balances(&self) -> impl Iterator<Item = (&str, &WalletBalance)> {
        self.balances.iter().map(|(a, b)| (a.as_str(), b))
    }

    pub fn position(&self, symbol: &str, side: PositionSide) -> Option<&PositionState> {
        self.positions
            .get(&(symbol.to_owned(), side))
            .filter(|p| p.is_open())
    }

    /// Open positions (amount not zero).
    pub fn positions(&self) -> impl Iterator<Item = &PositionState> {
        self.positions.values().filter(|p| p.is_open())
    }

    pub fn open_orders(&self) -> impl Iterator<Item = &OrderState> {
        self.orders.orders().open()
    }

    pub fn order(&self, symbol: &str, order_id: u64) -> Option<&OrderState> {
        self.orders.orders().get(symbol, order_id)
    }

    pub fn order_by_client_id(&self, client_order_id: &str) -> Option<&OrderState> {
        self.orders.orders().get_by_client_id(client_order_id)
    }

    pub fn open_algo_orders(&self) -> impl Iterator<Item = &AlgoOrderState> {
        self.algo_orders.values()
    }

    /// Apply a user data stream message; returns what changed (empty for
    /// stale, repeated or unrelated messages).
    pub fn apply(&mut self, message: &UserDataMessage) -> Vec<AccountChange> {
        match message {
            UserDataMessage::AccountUpdate(e) => self.apply_account_update(e),
            UserDataMessage::OrderTradeUpdate(e) => self
                .orders
                .apply(e)
                .map(AccountChange::Order)
                .into_iter()
                .collect(),
            UserDataMessage::AlgoUpdate(e) => self.apply_algo_update(e).into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn apply_account_update(&mut self, e: &AccountUpdateEvent) -> Vec<AccountChange> {
        let time = e.transaction_time;
        let mut changes = Vec::new();
        for b in &e.update.balances {
            if let Some(current) = self.balances.get(&b.asset)
                && time < current.updated_at
            {
                continue;
            }
            let balance = WalletBalance {
                wallet_balance: b.wallet_balance,
                cross_wallet_balance: b.cross_wallet_balance,
                updated_at: time,
            };
            let previous = self.balances.insert(b.asset.clone(), balance);
            if previous.is_none_or(|p| {
                (p.wallet_balance, p.cross_wallet_balance)
                    != (balance.wallet_balance, balance.cross_wallet_balance)
            }) {
                changes.push(AccountChange::Balance {
                    asset: b.asset.clone(),
                    balance,
                });
            }
        }
        for p in &e.update.positions {
            let key = (p.symbol.clone(), p.position_side);
            if let Some(current) = self.positions.get(&key)
                && time < current.updated_at
            {
                continue;
            }
            let position = PositionState {
                symbol: p.symbol.clone(),
                position_side: p.position_side,
                amount: p.position_amount,
                entry_price: p.entry_price,
                breakeven_price: Some(p.breakeven_price),
                unrealized_pnl: p.unrealized_pnl,
                margin_type: Some(p.margin_type),
                isolated_wallet: p.isolated_wallet,
                updated_at: time,
            };
            let previous = self.positions.insert(key, position.clone());
            if previous.is_none_or(|prev| {
                PositionState {
                    updated_at: time,
                    ..prev
                } != position
            }) {
                changes.push(AccountChange::Position(position));
            }
        }
        changes
    }

    fn apply_algo_update(&mut self, e: &AlgoUpdateEvent) -> Option<AccountChange> {
        let o = &e.order;
        let current = self.algo_orders.get(&o.algo_id);
        if current.is_some_and(|c| e.transaction_time < c.updated_at) {
            return None;
        }
        let algo = AlgoOrderState {
            algo_id: o.algo_id,
            client_algo_id: o.client_algo_id.clone(),
            symbol: o.symbol.clone(),
            quantity: o.quantity,
            trigger_price: o.trigger_price,
            price: o.price,
            status: o.algo_status.clone(),
            updated_at: e.transaction_time,
        };
        if current.is_some_and(|c| {
            AlgoOrderState {
                updated_at: algo.updated_at,
                ..c.clone()
            } == algo
        }) {
            return None;
        }
        if algo.is_final() {
            // Unknown final orders were never open here: nothing to report.
            self.algo_orders.remove(&algo.algo_id)?;
        } else {
            self.algo_orders.insert(algo.algo_id, algo.clone());
        }
        Some(AccountChange::AlgoOrder(algo))
    }
}

impl From<&AlgoOrder> for AlgoOrderState {
    fn from(a: &AlgoOrder) -> Self {
        Self {
            algo_id: a.algo_id as u64,
            client_algo_id: a.client_algo_id.clone(),
            symbol: a.symbol.clone(),
            quantity: a.quantity,
            trigger_price: a.trigger_price,
            price: a.price,
            status: a.algo_status.clone(),
            updated_at: a.update_time,
        }
    }
}

/// An order from `GET /fapi/v1/openOrders` (or any order query) seeds
/// [`OrderState`]; it carries no individual fills.
impl OrderEvent for Order {
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn order_id(&self) -> u64 {
        self.order_id as u64
    }
    fn client_order_id(&self) -> &str {
        &self.client_order_id
    }
    fn status(&self) -> OrderStatus {
        (&self.status).into()
    }
    fn original_qty(&self) -> Decimal {
        self.orig_qty
    }
    fn price(&self) -> Decimal {
        self.price
    }
    fn cumulative_filled_qty(&self) -> Decimal {
        self.executed_qty
    }
    fn average_price(&self) -> Option<Decimal> {
        self.avg_price.filter(|p| !p.is_zero())
    }
    fn fill(&self) -> Option<crate::Fill> {
        None
    }
    fn transaction_time(&self) -> Timestamp {
        self.update_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::dec;
    use serde_json::{Value, json};

    fn fixture(name: &str) -> Value {
        let path = format!(
            "{}/tests/fixtures/usdm_user_data/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn message(v: Value) -> UserDataMessage {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn account_update_respects_time_and_keeps_closed_positions() {
        let mut s = AccountState::default();
        let update = fixture("account_update.json");
        let changes = s.apply(&message(update.clone()));
        // 2 balances, 3 positions (one of them with amount 0).
        assert_eq!(changes.len(), 5);
        assert_eq!(s.positions().count(), 2);
        assert_eq!(
            s.position("BTCUSDT", PositionSide::Long).unwrap().amount,
            dec!(20)
        );
        assert_eq!(
            s.balance("USDT").unwrap().wallet_balance,
            dec!(122624.12345678)
        );
        assert!(s.apply(&message(update.clone())).is_empty(), "repeat");

        // Close the long, then a stale update that would reopen it.
        let mut close = update.clone();
        close["T"] = json!(1564745799000u64);
        close["a"]["B"] = json!([]);
        close["a"]["P"] = json!([{"s":"BTCUSDT","pa":"0","ep":"0","bep":"0","cr":"0","up":"0","mt":"isolated","iw":"0","ps":"LONG"}]);
        assert_eq!(s.apply(&message(close)).len(), 1);
        assert!(s.position("BTCUSDT", PositionSide::Long).is_none());
        assert!(s.apply(&message(update)).is_empty(), "stale");
        assert!(s.position("BTCUSDT", PositionSide::Long).is_none());
    }

    #[test]
    fn algo_orders_open_and_finish() {
        let mut s = AccountState::default();
        let mut new = fixture("algo_update.json");
        new["o"]["X"] = json!("NEW");
        assert_eq!(s.apply(&message(new.clone())).len(), 1);
        assert_eq!(s.open_algo_orders().count(), 1);
        assert!(s.apply(&message(new)).is_empty(), "repeat");
        let canceled = fixture("algo_update.json"); // X = CANCELED
        assert!(matches!(
            &s.apply(&message(canceled.clone()))[..],
            [AccountChange::AlgoOrder(a)] if a.is_final()
        ));
        assert_eq!(s.open_algo_orders().count(), 0);
        assert!(s.apply(&message(canceled)).is_empty());
    }

    #[test]
    fn orders_from_order_trade_update() {
        let mut s = AccountState::default();
        let new = message(fixture("order_trade_update.json"));
        assert!(
            matches!(&s.apply(&new)[..], [AccountChange::Order(o)] if o.status == OrderStatus::New)
        );
        assert_eq!(s.open_orders().count(), 1);
    }
}
