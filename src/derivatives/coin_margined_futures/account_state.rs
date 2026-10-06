//! COIN-M Futures positions, balances and open orders,
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
//!   never reopen).

use std::collections::{BTreeMap, HashMap};

use rust_decimal::Decimal;

use crate::{
    OrderEvent, OrderState, OrderStatus, Timestamp,
    derivatives::coin_margined_futures::{
        Error, MarginType, PositionSide,
        http::{
            GetAccountInformationParams, GetOpenOrdersParams, GetPositionInformationParams, Order,
            PrivateClient,
        },
        ws::{AccountUpdateEvent, UserDataMessage},
    },
    order_state::OpenOrders,
};

/// Wallet balance of one margin asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalletBalance {
    /// Wallet balance.
    pub wallet_balance: Decimal,
    /// Cross wallet balance.
    pub cross_wallet_balance: Decimal,
    /// Time of the last update applied.
    pub updated_at: Timestamp,
}

/// One position (per symbol and position side).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionState {
    /// Symbol.
    pub symbol: String,
    /// `BOTH` in one-way mode, `LONG` / `SHORT` in hedge mode.
    pub position_side: PositionSide,
    /// Signed: negative for a short in one-way mode. `0` once closed.
    pub amount: Decimal,
    /// Average entry price.
    pub entry_price: Decimal,
    /// Break-even price.
    pub breakeven_price: Option<Decimal>,
    /// Unrealized PnL.
    pub unrealized_pnl: Decimal,
    /// Isolated or crossed.
    pub margin_type: Option<MarginType>,
    /// Unknown until the first `ACCOUNT_UPDATE` (the position endpoint
    /// doesn't report it).
    pub isolated_wallet: Option<Decimal>,
    /// Time of the last update applied.
    pub updated_at: Timestamp,
}

impl PositionState {
    /// Amount is not zero.
    pub fn is_open(&self) -> bool {
        !self.amount.is_zero()
    }
}

/// What a message changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountChange {
    /// New balance of an asset.
    Balance {
        /// Asset.
        asset: String,
        /// New balance.
        balance: WalletBalance,
    },
    /// New state of a position (`amount` 0: closed).
    Position(PositionState),
    /// New state of an order; final orders are reported once, then dropped.
    Order(OrderState),
}

/// COIN-M Futures account: balances, positions and open orders.
#[derive(Debug, Clone, Default)]
pub struct AccountState {
    balances: BTreeMap<String, WalletBalance>,
    positions: HashMap<(String, PositionSide), PositionState>,
    orders: OpenOrders,
}

impl AccountState {
    /// Snapshot: `GET /dapi/v1/account`, `/dapi/v1/positionRisk` and
    /// `/dapi/v1/openOrders` (all symbols).
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
                    breakeven_price: Some(p.break_even_price),
                    unrealized_pnl: p.un_realized_profit,
                    margin_type: Some(p.margin_type),
                    isolated_wallet: None,
                    updated_at: p.update_time,
                },
            );
        }
        for order in &open_orders {
            state.orders.apply(order);
        }
        Ok(state)
    }

    /// Replace the state with a fresh snapshot (after a reconnect or a
    /// `Lagged` gap). Finished orders stay remembered.
    pub async fn reload(&mut self, client: &PrivateClient) -> Result<(), Error> {
        let fresh = Self::load(client).await?;
        self.balances = fresh.balances;
        self.positions = fresh.positions;
        self.orders.replace_with(fresh.orders);
        Ok(())
    }

    /// Balance of an asset.
    pub fn balance(&self, asset: &str) -> Option<&WalletBalance> {
        self.balances.get(asset)
    }

    /// Every asset, sorted by name.
    pub fn balances(&self) -> impl Iterator<Item = (&str, &WalletBalance)> {
        self.balances.iter().map(|(a, b)| (a.as_str(), b))
    }

    /// An open position by symbol and position side.
    pub fn position(&self, symbol: &str, side: PositionSide) -> Option<&PositionState> {
        self.positions
            .get(&(symbol.to_owned(), side))
            .filter(|p| p.is_open())
    }

    /// Open positions (amount not zero).
    pub fn positions(&self) -> impl Iterator<Item = &PositionState> {
        self.positions.values().filter(|p| p.is_open())
    }

    /// Open orders.
    pub fn open_orders(&self) -> impl Iterator<Item = &OrderState> {
        self.orders.orders().open()
    }

    /// An order by symbol and order id.
    pub fn order(&self, symbol: &str, order_id: u64) -> Option<&OrderState> {
        self.orders.orders().get(symbol, order_id)
    }

    /// An order by client order id.
    pub fn order_by_client_id(&self, client_order_id: &str) -> Option<&OrderState> {
        self.orders.orders().get_by_client_id(client_order_id)
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
                isolated_wallet: Some(p.isolated_wallet),
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
}

/// An order from `GET /dapi/v1/openOrders` (or any order query) seeds
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
            "{}/tests/fixtures/coinm_user_data/{name}",
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
            s.position("BTCUSD_200925", PositionSide::Long)
                .unwrap()
                .amount,
            dec!(20)
        );
        assert_eq!(
            s.balance("BTC").unwrap().wallet_balance,
            dec!(122624.12345678)
        );
        assert!(s.apply(&message(update.clone())).is_empty(), "repeat");

        // Close the long, then a stale update that would reopen it.
        let mut close = update.clone();
        close["T"] = json!(1564745799000u64);
        close["a"]["B"] = json!([]);
        close["a"]["P"] = json!([{"s":"BTCUSD_200925","pa":"0","ep":"0","bep":"0","cr":"0","up":"0","mt":"isolated","iw":"0","ps":"LONG"}]);
        assert_eq!(s.apply(&message(close)).len(), 1);
        assert!(s.position("BTCUSD_200925", PositionSide::Long).is_none());
        assert!(s.apply(&message(update)).is_empty(), "stale");
        assert!(s.position("BTCUSD_200925", PositionSide::Long).is_none());
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
