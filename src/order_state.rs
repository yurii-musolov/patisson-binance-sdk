//! Track orders from user data stream events.
//!
//! [`OrderState`] folds the order updates of one order (spot and margin
//! `executionReport`, USD-M and COIN-M `ORDER_TRADE_UPDATE`) into its current
//! status, filled quantity, average price, commissions and fills; [`Orders`]
//! keeps every order of an account by id and client order id.
//!
//! Streams can repeat or reorder events (reconnects, events read both from a
//! stream and from REST). Applying the same event twice, or an older event
//! after a newer one, leaves the state unchanged:
//!
//! - a fill is counted once per trade id;
//! - an event whose cumulative filled quantity is lower than the current one
//!   is stale;
//! - a final status (`FILLED`, `CANCELED`, ...) never goes back to an open
//!   one.

use std::collections::{BTreeMap, HashMap};

use rust_decimal::Decimal;

use crate::{
    Timestamp,
    derivatives::{coin_margined_futures as coinm, usds_margined_futures as usdm},
    margin, spot,
};

/// Order status, common to every product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    /// Accepted by the engine but not yet placed (spot `PENDING_NEW`).
    PendingNew,
    /// On the book (COIN-M `NEW_INSURANCE` / `NEW_ADL` liquidation orders
    /// too).
    New,
    PartiallyFilled,
    Filled,
    /// Cancel requested, not yet confirmed.
    PendingCancel,
    Canceled,
    Rejected,
    Expired,
    /// Expired by self-trade prevention.
    ExpiredInMatch,
    /// A status this SDK version doesn't know.
    Unknown,
}

impl OrderStatus {
    /// The order can no longer change.
    pub fn is_final(self) -> bool {
        matches!(
            self,
            Self::Filled | Self::Canceled | Self::Rejected | Self::Expired | Self::ExpiredInMatch
        )
    }
}

/// One trade of an order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub trade_id: u64,
    pub price: Decimal,
    pub qty: Decimal,
    pub commission: Decimal,
    pub commission_asset: Option<String>,
    pub is_maker: bool,
    pub time: Timestamp,
}

/// An order update event, whatever the product. Implemented for the user
/// data stream order events of spot, margin, USD-M and COIN-M.
pub trait OrderEvent {
    fn symbol(&self) -> &str;
    fn order_id(&self) -> u64;
    /// The id the order was placed with (for a cancel, the original order's
    /// id, not the cancel request's).
    fn client_order_id(&self) -> &str;
    fn status(&self) -> OrderStatus;
    fn original_qty(&self) -> Decimal;
    fn price(&self) -> Decimal;
    fn cumulative_filled_qty(&self) -> Decimal;
    /// Average fill price so far, if anything was filled.
    fn average_price(&self) -> Option<Decimal>;
    /// The trade this event reports, if it reports one.
    fn fill(&self) -> Option<Fill>;
    fn transaction_time(&self) -> Timestamp;
}

/// Current state of one order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderState {
    pub symbol: String,
    pub order_id: u64,
    pub client_order_id: String,
    pub status: OrderStatus,
    pub original_qty: Decimal,
    pub price: Decimal,
    pub filled_qty: Decimal,
    pub average_price: Option<Decimal>,
    /// Total commission by asset.
    pub commissions: BTreeMap<String, Decimal>,
    /// Fills in the order they were applied.
    pub fills: Vec<Fill>,
    /// Transaction time of the first and the latest applied event.
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl OrderState {
    /// State from the first event seen for an order.
    pub fn new(event: &impl OrderEvent) -> Self {
        let mut state = Self {
            symbol: event.symbol().to_owned(),
            order_id: event.order_id(),
            client_order_id: event.client_order_id().to_owned(),
            status: event.status(),
            original_qty: event.original_qty(),
            price: event.price(),
            filled_qty: event.cumulative_filled_qty(),
            average_price: event.average_price(),
            commissions: BTreeMap::new(),
            fills: Vec::new(),
            created_at: event.transaction_time(),
            updated_at: event.transaction_time(),
        };
        state.add_fill(event);
        state
    }

    /// Apply an update of this order. Returns `false` (and changes nothing)
    /// for another order, a repeated event or a stale one.
    pub fn apply(&mut self, event: &impl OrderEvent) -> bool {
        if event.order_id() != self.order_id || event.symbol() != self.symbol {
            return false;
        }
        let mut changed = self.add_fill(event);
        let status = event.status();
        let reopens = self.status.is_final() && !status.is_final();
        let stale = event.cumulative_filled_qty() < self.filled_qty;
        if !reopens && !stale {
            let before = (
                self.status,
                self.filled_qty,
                self.average_price,
                self.original_qty,
                self.price,
            );
            self.status = status;
            self.filled_qty = event.cumulative_filled_qty();
            if let Some(price) = event.average_price() {
                self.average_price = Some(price);
            }
            // Amendments change quantity and price.
            self.original_qty = event.original_qty();
            self.price = event.price();
            if !event.client_order_id().is_empty() {
                self.client_order_id = event.client_order_id().to_owned();
            }
            changed |= before
                != (
                    self.status,
                    self.filled_qty,
                    self.average_price,
                    self.original_qty,
                    self.price,
                );
        }
        if changed {
            self.updated_at = self.updated_at.max(event.transaction_time());
            self.created_at = self.created_at.min(event.transaction_time());
        }
        changed
    }

    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    /// Quantity not filled yet (zero once the order is final).
    pub fn remaining_qty(&self) -> Decimal {
        if self.is_final() {
            Decimal::ZERO
        } else {
            (self.original_qty - self.filled_qty).max(Decimal::ZERO)
        }
    }

    fn add_fill(&mut self, event: &impl OrderEvent) -> bool {
        let Some(fill) = event.fill() else {
            return false;
        };
        if self.fills.iter().any(|f| f.trade_id == fill.trade_id) {
            return false;
        }
        if let Some(asset) = &fill.commission_asset {
            *self.commissions.entry(asset.clone()).or_default() += fill.commission;
        }
        self.fills.push(fill);
        true
    }
}

/// Orders of an account, by `(symbol, orderId)` and by client order id.
#[derive(Debug, Clone, Default)]
pub struct Orders {
    orders: HashMap<(String, u64), OrderState>,
    by_client_id: HashMap<String, (String, u64)>,
}

impl Orders {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply an order event, creating the order on its first event.
    pub fn apply(&mut self, event: &impl OrderEvent) -> &OrderState {
        let key = (event.symbol().to_owned(), event.order_id());
        let state = self
            .orders
            .entry(key.clone())
            .and_modify(|state| {
                state.apply(event);
            })
            .or_insert_with(|| OrderState::new(event));
        self.by_client_id.insert(state.client_order_id.clone(), key);
        state
    }

    pub fn get(&self, symbol: &str, order_id: u64) -> Option<&OrderState> {
        self.orders.get(&(symbol.to_owned(), order_id))
    }

    pub fn get_by_client_id(&self, client_order_id: &str) -> Option<&OrderState> {
        self.by_client_id
            .get(client_order_id)
            .and_then(|key| self.orders.get(key))
    }

    /// Orders that can still change.
    pub fn open(&self) -> impl Iterator<Item = &OrderState> {
        self.orders.values().filter(|o| !o.is_final())
    }

    pub fn iter(&self) -> impl Iterator<Item = &OrderState> {
        self.orders.values()
    }

    pub fn len(&self) -> usize {
        self.orders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Drop final orders and return them.
    pub fn remove_final(&mut self) -> Vec<OrderState> {
        let keys: Vec<_> = self
            .orders
            .iter()
            .filter(|(_, o)| o.is_final())
            .map(|(k, _)| k.clone())
            .collect();
        let removed: Vec<_> = keys.iter().filter_map(|k| self.orders.remove(k)).collect();
        self.by_client_id
            .retain(|_, key| self.orders.contains_key(key));
        removed
    }
}

fn average(quote: Decimal, base: Decimal) -> Option<Decimal> {
    (!base.is_zero()).then(|| (quote / base).normalize())
}

fn nonzero(value: Decimal) -> Option<Decimal> {
    (!value.is_zero()).then_some(value)
}

// ===== spot =====

impl From<&spot::OrderStatus> for OrderStatus {
    fn from(s: &spot::OrderStatus) -> Self {
        use spot::OrderStatus as S;
        match s {
            S::PendingNew => Self::PendingNew,
            S::New => Self::New,
            S::PartiallyFilled => Self::PartiallyFilled,
            S::Filled => Self::Filled,
            S::PendingCancel => Self::PendingCancel,
            S::Canceled => Self::Canceled,
            S::Rejected => Self::Rejected,
            S::Expired => Self::Expired,
            S::ExpiredInMatch => Self::ExpiredInMatch,
            S::Unknown => Self::Unknown,
        }
    }
}

impl OrderEvent for spot::ws_api::ExecutionReport {
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn order_id(&self) -> u64 {
        self.order_id as u64
    }
    fn client_order_id(&self) -> &str {
        // On a cancel `c` is the cancel request's id and `C` the order's.
        if self.orig_client_order_id.is_empty() {
            &self.client_order_id
        } else {
            &self.orig_client_order_id
        }
    }
    fn status(&self) -> OrderStatus {
        (&self.order_status).into()
    }
    fn original_qty(&self) -> Decimal {
        self.quantity
    }
    fn price(&self) -> Decimal {
        self.price
    }
    fn cumulative_filled_qty(&self) -> Decimal {
        self.cumulative_filled_qty
    }
    fn average_price(&self) -> Option<Decimal> {
        average(self.cumulative_quote_qty, self.cumulative_filled_qty)
    }
    fn fill(&self) -> Option<Fill> {
        (self.trade_id >= 0 && !self.last_executed_qty.is_zero()).then(|| Fill {
            trade_id: self.trade_id as u64,
            price: self.last_executed_price,
            qty: self.last_executed_qty,
            commission: self.commission,
            commission_asset: self.commission_asset.clone(),
            is_maker: self.is_maker,
            time: self.transaction_time,
        })
    }
    fn transaction_time(&self) -> Timestamp {
        self.transaction_time
    }
}

// ===== margin =====

impl From<&margin::OrderStatus> for OrderStatus {
    fn from(s: &margin::OrderStatus) -> Self {
        use margin::OrderStatus as S;
        match s {
            S::New => Self::New,
            S::PartiallyFilled => Self::PartiallyFilled,
            S::Filled => Self::Filled,
            S::PendingCancel => Self::PendingCancel,
            S::Canceled => Self::Canceled,
            S::Rejected => Self::Rejected,
            S::Expired => Self::Expired,
            S::ExpiredInMatch => Self::ExpiredInMatch,
            S::Unknown => Self::Unknown,
        }
    }
}

impl OrderEvent for margin::ws::ExecutionReportEvent {
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn order_id(&self) -> u64 {
        self.order_id as u64
    }
    fn client_order_id(&self) -> &str {
        if self.orig_client_order_id.is_empty() {
            &self.client_order_id
        } else {
            &self.orig_client_order_id
        }
    }
    fn status(&self) -> OrderStatus {
        (&self.order_status).into()
    }
    fn original_qty(&self) -> Decimal {
        self.orig_qty
    }
    fn price(&self) -> Decimal {
        self.price
    }
    fn cumulative_filled_qty(&self) -> Decimal {
        self.cumulative_filled_qty
    }
    fn average_price(&self) -> Option<Decimal> {
        average(self.cumulative_quote_qty, self.cumulative_filled_qty)
    }
    fn fill(&self) -> Option<Fill> {
        (self.trade_id >= 0 && !self.last_executed_qty.is_zero()).then(|| Fill {
            trade_id: self.trade_id as u64,
            price: self.last_executed_price,
            qty: self.last_executed_qty,
            commission: self.commission,
            commission_asset: self.commission_asset.clone(),
            is_maker: self.is_maker,
            time: self.transaction_time,
        })
    }
    fn transaction_time(&self) -> Timestamp {
        self.transaction_time
    }
}

// ===== USD-M and COIN-M futures =====

impl From<&usdm::OrderStatus> for OrderStatus {
    fn from(s: &usdm::OrderStatus) -> Self {
        use usdm::OrderStatus as S;
        match s {
            S::New => Self::New,
            S::PartiallyFilled => Self::PartiallyFilled,
            S::Filled => Self::Filled,
            S::Canceled => Self::Canceled,
            S::Rejected => Self::Rejected,
            S::Expired => Self::Expired,
            S::ExpiredInMatch => Self::ExpiredInMatch,
            S::Unknown => Self::Unknown,
        }
    }
}

impl From<&coinm::OrderStatus> for OrderStatus {
    fn from(s: &coinm::OrderStatus) -> Self {
        use coinm::OrderStatus as S;
        match s {
            S::New | S::NewInsurance | S::NewAdl => Self::New,
            S::PartiallyFilled => Self::PartiallyFilled,
            S::Filled => Self::Filled,
            S::Canceled => Self::Canceled,
            S::Rejected => Self::Rejected,
            S::Expired => Self::Expired,
            S::Unknown => Self::Unknown,
        }
    }
}

macro_rules! futures_order_event {
    ($event:ty) => {
        impl OrderEvent for $event {
            fn symbol(&self) -> &str {
                &self.order.symbol
            }
            fn order_id(&self) -> u64 {
                self.order.order_id
            }
            fn client_order_id(&self) -> &str {
                &self.order.client_order_id
            }
            fn status(&self) -> OrderStatus {
                (&self.order.order_status).into()
            }
            fn original_qty(&self) -> Decimal {
                self.order.original_quantity
            }
            fn price(&self) -> Decimal {
                self.order.original_price
            }
            fn cumulative_filled_qty(&self) -> Decimal {
                self.order.cumulative_filled_quantity
            }
            fn average_price(&self) -> Option<Decimal> {
                nonzero(self.order.average_price)
            }
            fn fill(&self) -> Option<Fill> {
                let o = &self.order;
                (!o.last_filled_quantity.is_zero()).then(|| Fill {
                    trade_id: o.trade_id,
                    price: o.last_filled_price,
                    qty: o.last_filled_quantity,
                    commission: o.commission.unwrap_or_default(),
                    commission_asset: o.commission_asset.clone(),
                    is_maker: o.is_maker,
                    time: o.trade_time,
                })
            }
            fn transaction_time(&self) -> Timestamp {
                self.transaction_time
            }
        }
    };
}

futures_order_event!(usdm::ws::OrderTradeUpdateEvent);
futures_order_event!(coinm::ws::OrderTradeUpdateEvent);

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::dec;

    /// Minimal event for the state machine tests.
    #[derive(Clone)]
    struct Ev {
        status: OrderStatus,
        cum: Decimal,
        avg: Option<Decimal>,
        fill: Option<(u64, Decimal, Decimal)>,
        time: Timestamp,
    }

    impl OrderEvent for Ev {
        fn symbol(&self) -> &str {
            "BTCUSDT"
        }
        fn order_id(&self) -> u64 {
            7
        }
        fn client_order_id(&self) -> &str {
            "my-order"
        }
        fn status(&self) -> OrderStatus {
            self.status
        }
        fn original_qty(&self) -> Decimal {
            dec!(3)
        }
        fn price(&self) -> Decimal {
            dec!(100)
        }
        fn cumulative_filled_qty(&self) -> Decimal {
            self.cum
        }
        fn average_price(&self) -> Option<Decimal> {
            self.avg
        }
        fn fill(&self) -> Option<Fill> {
            self.fill.map(|(trade_id, price, qty)| Fill {
                trade_id,
                price,
                qty,
                commission: dec!(0.1),
                commission_asset: Some("USDT".into()),
                is_maker: true,
                time: self.time,
            })
        }
        fn transaction_time(&self) -> Timestamp {
            self.time
        }
    }

    fn new() -> Ev {
        Ev {
            status: OrderStatus::New,
            cum: dec!(0),
            avg: None,
            fill: None,
            time: 1,
        }
    }
    fn partial() -> Ev {
        Ev {
            status: OrderStatus::PartiallyFilled,
            cum: dec!(1),
            avg: Some(dec!(100)),
            fill: Some((11, dec!(100), dec!(1))),
            time: 2,
        }
    }
    fn filled() -> Ev {
        Ev {
            status: OrderStatus::Filled,
            cum: dec!(3),
            avg: Some(dec!(99)),
            fill: Some((12, dec!(98.5), dec!(2))),
            time: 3,
        }
    }

    #[test]
    fn folds_the_lifecycle() {
        let mut o = OrderState::new(&new());
        assert_eq!(o.remaining_qty(), dec!(3));
        assert!(o.apply(&partial()));
        assert!(o.apply(&filled()));
        assert_eq!(o.status, OrderStatus::Filled);
        assert_eq!(o.filled_qty, dec!(3));
        assert_eq!(o.average_price, Some(dec!(99)));
        assert_eq!(o.fills.len(), 2);
        assert_eq!(o.commissions["USDT"], dec!(0.2));
        assert_eq!(o.remaining_qty(), dec!(0));
        assert_eq!((o.created_at, o.updated_at), (1, 3));
    }

    #[test]
    fn repeated_and_reordered_events_change_nothing() {
        let mut o = OrderState::new(&new());
        o.apply(&partial());
        o.apply(&filled());
        let done = o.clone();
        // Repeats.
        assert!(!o.apply(&partial()));
        assert!(!o.apply(&filled()));
        // Late events after the final state.
        assert!(!o.apply(&new()));
        assert_eq!(o, done);

        // Out of order: the fill arrives before the partial update.
        let mut o = OrderState::new(&new());
        o.apply(&filled());
        o.apply(&partial()); // stale state, but its trade is new
        assert_eq!(o.status, OrderStatus::Filled);
        assert_eq!(o.filled_qty, dec!(3));
        assert_eq!(o.fills.len(), 2);
    }

    #[test]
    fn registry_by_id_and_client_id() {
        let mut orders = Orders::new();
        orders.apply(&new());
        assert_eq!(orders.open().count(), 1);
        orders.apply(&filled());
        assert_eq!(
            orders.get("BTCUSDT", 7).unwrap().status,
            OrderStatus::Filled
        );
        assert_eq!(orders.get_by_client_id("my-order").unwrap().order_id, 7);
        assert_eq!(orders.open().count(), 0);
        assert_eq!(orders.remove_final().len(), 1);
        assert!(orders.is_empty() && orders.get_by_client_id("my-order").is_none());
    }
}
