//! Spot account balances and open orders, kept current from the user data
//! stream.
//!
//! Load a snapshot over REST ([`AccountState::load`]) and apply every user
//! data event ([`AccountState::apply`]). Events older than the data they
//! would replace are ignored, so the usual flow is safe:
//!
//! 1. subscribe to the user data stream and buffer its events;
//! 2. load the snapshot;
//! 3. apply the buffered events, then the live ones;
//! 4. after a reconnect, [`AccountState::reload`] and continue.
//!
//! Balance rules:
//! - `outboundAccountPosition` carries absolute balances and wins when its
//!   `u` (last account update) is not older than the asset's last update;
//! - `balanceUpdate` (deposits, withdrawals, transfers) and
//!   `externalLockUpdate` are deltas, applied only when their time is newer
//!   than the asset's last update, so a delta already contained in the
//!   snapshot or in a later `outboundAccountPosition` is not counted twice.
//!   An external lock moves `delta` from free to locked.

use std::collections::BTreeMap;

use rust_decimal::Decimal;

use crate::{
    OrderEvent, OrderState, OrderStatus, Timestamp,
    order_state::OpenOrders,
    spot::{
        Error,
        http::{GetAccountInformationParams, GetOpenOrdersParams, Order, PrivateClient},
        ws_api::UserDataEvent,
    },
};

/// Balance of one asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetBalance {
    pub free: Decimal,
    pub locked: Decimal,
    /// Time of the last update applied to this asset.
    pub updated_at: Timestamp,
}

impl AssetBalance {
    pub fn total(&self) -> Decimal {
        self.free + self.locked
    }
}

/// What an event changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountChange {
    /// New balance of an asset.
    Balance {
        asset: String,
        balance: AssetBalance,
    },
    /// New state of an order. Final orders are reported once, then dropped
    /// from [`AccountState::open_orders`].
    Order(OrderState),
}

/// Spot balances and open orders.
#[derive(Debug, Clone, Default)]
pub struct AccountState {
    balances: BTreeMap<String, AssetBalance>,
    orders: OpenOrders,
}

impl AccountState {
    /// Snapshot: `GET /api/v3/account` and `GET /api/v3/openOrders` (all
    /// symbols, weight 80).
    pub async fn load(client: &PrivateClient) -> Result<Self, Error> {
        let account = client
            .account_information(GetAccountInformationParams::new())
            .await?
            .result;
        let open_orders = client
            .get_open_orders(GetOpenOrdersParams::new())
            .await?
            .result;
        let mut state = Self::default();
        for b in account.balances {
            state.balances.insert(
                b.asset,
                AssetBalance {
                    free: b.free,
                    locked: b.locked,
                    updated_at: account.update_time,
                },
            );
        }
        for order in &open_orders {
            state.orders.apply(order);
        }
        Ok(state)
    }

    /// Replace the state with a fresh snapshot (after a reconnect, when
    /// events may have been missed).
    pub async fn reload(&mut self, client: &PrivateClient) -> Result<(), Error> {
        let fresh = Self::load(client).await?;
        self.balances = fresh.balances;
        // Keeps the finished orders so late events still can't reopen them.
        self.orders.replace_with(fresh.orders);
        Ok(())
    }

    pub fn balance(&self, asset: &str) -> Option<&AssetBalance> {
        self.balances.get(asset)
    }

    /// Every asset, sorted by name.
    pub fn balances(&self) -> impl Iterator<Item = (&str, &AssetBalance)> {
        self.balances.iter().map(|(a, b)| (a.as_str(), b))
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

    /// Apply a user data stream event; returns what changed (empty for
    /// stale, repeated or unrelated events).
    pub fn apply(&mut self, event: &UserDataEvent) -> Vec<AccountChange> {
        match event {
            UserDataEvent::OutboundAccountPosition(e) => e
                .balances
                .iter()
                .filter_map(|b| self.set_balance(&b.asset, b.free, b.locked, e.last_update_time))
                .collect(),
            UserDataEvent::BalanceUpdate(e) => self
                .add_delta(&e.asset, e.delta, Decimal::ZERO, e.clear_time)
                .into_iter()
                .collect(),
            UserDataEvent::ExternalLockUpdate(e) => self
                .add_delta(&e.asset, -e.delta, e.delta, e.transaction_time)
                .into_iter()
                .collect(),
            UserDataEvent::ExecutionReport(e) => self.apply_order(e).into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn apply_order(&mut self, event: &impl OrderEvent) -> Option<AccountChange> {
        self.orders.apply(event).map(AccountChange::Order)
    }

    fn set_balance(
        &mut self,
        asset: &str,
        free: Decimal,
        locked: Decimal,
        time: Timestamp,
    ) -> Option<AccountChange> {
        if let Some(current) = self.balances.get(asset)
            && time < current.updated_at
        {
            return None;
        }
        let balance = AssetBalance {
            free,
            locked,
            updated_at: time,
        };
        let previous = self.balances.insert(asset.to_owned(), balance);
        let changed = previous.is_none_or(|p| (p.free, p.locked) != (free, locked));
        changed.then(|| AccountChange::Balance {
            asset: asset.to_owned(),
            balance,
        })
    }

    fn add_delta(
        &mut self,
        asset: &str,
        free_delta: Decimal,
        locked_delta: Decimal,
        time: Timestamp,
    ) -> Option<AccountChange> {
        let current = self.balances.get(asset).copied().unwrap_or(AssetBalance {
            free: Decimal::ZERO,
            locked: Decimal::ZERO,
            updated_at: 0,
        });
        if time <= current.updated_at {
            return None;
        }
        self.set_balance(
            asset,
            current.free + free_delta,
            current.locked + locked_delta,
            time,
        )
    }
}

/// An open order from `GET /api/v3/openOrders` (or any order query) seeds
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
        (!self.executed_qty.is_zero())
            .then(|| (self.cummulative_quote_qty / self.executed_qty).normalize())
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

    fn event(json: &str) -> UserDataEvent {
        serde_json::from_str(json).unwrap()
    }

    fn state() -> AccountState {
        let mut s = AccountState::default();
        s.balances.insert(
            "BTC".into(),
            AssetBalance {
                free: dec!(1),
                locked: dec!(0),
                updated_at: 1000,
            },
        );
        s
    }

    #[test]
    fn absolute_balances_respect_time() {
        let mut s = state();
        let old = event(
            r#"{"e":"outboundAccountPosition","E":1,"u":999,"B":[{"a":"BTC","f":"5","l":"0"}]}"#,
        );
        assert!(s.apply(&old).is_empty(), "older than the snapshot");
        let new = event(
            r#"{"e":"outboundAccountPosition","E":1,"u":1001,"B":[{"a":"BTC","f":"0.5","l":"0.5"},{"a":"USDT","f":"10","l":"0"}]}"#,
        );
        assert_eq!(s.apply(&new).len(), 2);
        assert_eq!(s.balance("BTC").unwrap().total(), dec!(1));
        assert_eq!(s.balance("USDT").unwrap().free, dec!(10));
        assert!(s.apply(&new).is_empty(), "repeat");
    }

    #[test]
    fn deltas_are_not_counted_twice() {
        let mut s = state();
        let deposit = event(r#"{"e":"balanceUpdate","E":1,"a":"BTC","d":"0.25","T":1100}"#);
        assert_eq!(s.apply(&deposit).len(), 1);
        assert_eq!(s.balance("BTC").unwrap().free, dec!(1.25));
        assert!(s.apply(&deposit).is_empty(), "repeat");
        // A delta already contained in the snapshot.
        let old = event(r#"{"e":"balanceUpdate","E":1,"a":"BTC","d":"7","T":900}"#);
        assert!(s.apply(&old).is_empty());
        // The absolute balance after the deposit, then the late delta.
        let mut s = state();
        let after = event(
            r#"{"e":"outboundAccountPosition","E":1,"u":1100,"B":[{"a":"BTC","f":"1.25","l":"0"}]}"#,
        );
        s.apply(&after);
        assert!(s.apply(&deposit).is_empty());
        assert_eq!(s.balance("BTC").unwrap().free, dec!(1.25));
    }

    #[test]
    fn external_lock_moves_free_to_locked() {
        let mut s = state();
        let lock = event(r#"{"e":"externalLockUpdate","E":1,"a":"BTC","d":"0.4","T":1200}"#);
        s.apply(&lock);
        let b = s.balance("BTC").unwrap();
        assert_eq!((b.free, b.locked), (dec!(0.6), dec!(0.4)));
    }

    #[test]
    fn final_orders_are_reported_then_dropped() {
        let doc: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/spot_user_data/event_executionReport.json"
        ))
        .unwrap();
        let new: UserDataEvent = serde_json::from_value(doc["event"].clone()).unwrap();
        let mut canceled = doc["event"].clone();
        canceled["x"] = "CANCELED".into();
        canceled["X"] = "CANCELED".into();
        canceled["T"] = 1499405658700u64.into();
        let canceled: UserDataEvent = serde_json::from_value(canceled).unwrap();

        let mut s = AccountState::default();
        assert!(
            matches!(&s.apply(&new)[..], [AccountChange::Order(o)] if o.status == OrderStatus::New)
        );
        assert_eq!(s.open_orders().count(), 1);
        assert!(s.apply(&new).is_empty(), "repeat");
        assert!(matches!(
            &s.apply(&canceled)[..],
            [AccountChange::Order(o)] if o.status == OrderStatus::Canceled
        ));
        assert_eq!(s.open_orders().count(), 0);
        assert!(s.order("ETHBTC", 4293153).is_none());
        assert!(s.apply(&new).is_empty(), "a late event doesn't reopen it");
        assert_eq!(s.open_orders().count(), 0);
    }
}
