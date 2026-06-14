//! Local L2 order book for USDⓈ-M Futures.
//!
//! [`OrderBookState`] encapsulates the full synchronization protocol from the
//! Binance USDⓈ-M docs (snapshot + diff stream, `pu`/`u` chain) behind a
//! state machine that accepts a REST snapshot and live diff events in any
//! order. Callers feed events as they arrive and inspect the returned
//! [`ApplyOutcome`] — on [`ApplyOutcome::ResyncRequired`], fetch a new
//! snapshot.

use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};

use crate::derivatives::usds_margined_futures::{http::OrderBook, ws::DepthUpdateMsg};

/// Outcome of [`OrderBookState::apply_snapshot`] / [`OrderBookState::apply_diff`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Data was stored but the book is not yet synchronized; more input is
    /// needed (e.g., snapshot received but no bridging diff has arrived yet,
    /// or diffs received before the snapshot).
    Buffered,
    /// State has just transitioned to fully synchronized: snapshot + bridging
    /// diffs have been applied and the local book is now live.
    Synced,
    /// Diff was applied to an already-synchronized live book.
    Applied,
    /// Data was stale relative to current state and was dropped.
    Ignored,
    /// State is unrecoverable from current data — the snapshot is too old to
    /// bridge the diff stream, or the live diff chain has a gap. The caller
    /// must fetch a fresh snapshot and feed it back via [`OrderBookState::apply_snapshot`].
    /// Buffered diffs are preserved so the new snapshot has a chance to bridge.
    ResyncRequired,
}

/// Local L2 order book for a USDⓈ-M Futures symbol.
///
/// Built from a REST depth snapshot plus the `<symbol>@depth` diff stream.
/// Snapshots and diffs may be fed in any order; the state machine handles
/// buffering, draining stale events, bridging, and live-chain verification
/// (`pu == previous u`) internally.
#[derive(Debug, Default)]
pub struct OrderBookState {
    inner: Inner,
}

#[derive(Debug)]
enum Inner {
    /// No snapshot yet. Diffs accumulate in the buffer.
    NoSnapshot { buffered: VecDeque<DepthUpdateMsg> },
    /// Snapshot received but not yet bridged. Diffs continue to accumulate.
    Pending {
        snapshot_last_id: i64,
        bids: BTreeMap<Decimal, Decimal>,
        asks: BTreeMap<Decimal, Decimal>,
        buffered: VecDeque<DepthUpdateMsg>,
    },
    /// Fully synchronized live book.
    Synced {
        last_update_id: i64,
        bids: BTreeMap<Decimal, Decimal>,
        asks: BTreeMap<Decimal, Decimal>,
    },
}

impl Default for Inner {
    fn default() -> Self {
        Self::NoSnapshot {
            buffered: VecDeque::new(),
        }
    }
}

impl OrderBookState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a REST depth snapshot into the state machine.
    pub fn apply_snapshot(&mut self, snapshot: OrderBook) -> ApplyOutcome {
        let new_id = snapshot.last_update_id;

        match &self.inner {
            Inner::Synced { last_update_id, .. } => {
                if new_id <= *last_update_id {
                    return ApplyOutcome::Ignored;
                }
                // Newer snapshot during live operation: replace the book.
                let (bids, asks) = sides_from_snapshot(snapshot);
                self.inner = Inner::Synced {
                    last_update_id: new_id,
                    bids,
                    asks,
                };
                return ApplyOutcome::Applied;
            }
            Inner::Pending {
                snapshot_last_id, ..
            } if new_id <= *snapshot_last_id => {
                return ApplyOutcome::Ignored;
            }
            _ => {}
        }

        // Transition to Pending, preserving any buffered diffs, then try to bridge.
        let buffered = match std::mem::take(&mut self.inner) {
            Inner::NoSnapshot { buffered } | Inner::Pending { buffered, .. } => buffered,
            Inner::Synced { .. } => unreachable!("handled above"),
        };
        let (bids, asks) = sides_from_snapshot(snapshot);
        self.inner = Inner::Pending {
            snapshot_last_id: new_id,
            bids,
            asks,
            buffered,
        };
        self.try_bridge()
    }

    /// Feed a live diff event into the state machine.
    pub fn apply_diff(&mut self, diff: DepthUpdateMsg) -> ApplyOutcome {
        match &mut self.inner {
            Inner::NoSnapshot { buffered } => {
                buffered.push_back(diff);
                ApplyOutcome::Buffered
            }
            Inner::Pending { buffered, .. } => {
                buffered.push_back(diff);
                self.try_bridge()
            }
            Inner::Synced {
                last_update_id,
                bids,
                asks,
            } => {
                if diff.final_update_id < *last_update_id {
                    return ApplyOutcome::Ignored;
                }
                if diff.previous_final_update_id != *last_update_id {
                    // Sequence broken: reset to NoSnapshot, preserve this diff
                    // so the next snapshot has a chance to bridge it.
                    let mut buffered = VecDeque::new();
                    buffered.push_back(diff);
                    self.inner = Inner::NoSnapshot { buffered };
                    return ApplyOutcome::ResyncRequired;
                }
                apply_diff_to_sides(bids, asks, &diff);
                *last_update_id = diff.final_update_id;
                ApplyOutcome::Applied
            }
        }
    }

    /// `true` if the state machine has reached the synchronized state and the
    /// book is live.
    pub fn is_synced(&self) -> bool {
        matches!(self.inner, Inner::Synced { .. })
    }

    /// Last applied update id once synced, otherwise `None`.
    pub fn last_update_id(&self) -> Option<i64> {
        match &self.inner {
            Inner::Synced { last_update_id, .. } => Some(*last_update_id),
            _ => None,
        }
    }

    /// Live bid side once synced, otherwise `None`.
    pub fn bids(&self) -> Option<&BTreeMap<Decimal, Decimal>> {
        match &self.inner {
            Inner::Synced { bids, .. } => Some(bids),
            _ => None,
        }
    }

    /// Live ask side once synced, otherwise `None`.
    pub fn asks(&self) -> Option<&BTreeMap<Decimal, Decimal>> {
        match &self.inner {
            Inner::Synced { asks, .. } => Some(asks),
            _ => None,
        }
    }

    /// Best bid (highest price + qty) once synced.
    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        self.bids()
            .and_then(|b| b.iter().next_back().map(|(p, q)| (*p, *q)))
    }

    /// Best ask (lowest price + qty) once synced.
    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        self.asks()
            .and_then(|a| a.iter().next().map(|(p, q)| (*p, *q)))
    }

    /// Attempt to bridge the held snapshot with buffered diffs.
    ///
    /// Per Binance USDⓈ-M docs:
    /// - Drop buffered events with `u < lastUpdateId` (stale).
    /// - First processed event must satisfy `U <= lastUpdateId AND u >= lastUpdateId`.
    /// - Subsequent events must chain: `pu == previous u`.
    fn try_bridge(&mut self) -> ApplyOutcome {
        if !matches!(self.inner, Inner::Pending { .. }) {
            return ApplyOutcome::Buffered;
        }

        let Inner::Pending {
            snapshot_last_id,
            bids,
            asks,
            mut buffered,
        } = std::mem::take(&mut self.inner)
        else {
            unreachable!("checked above");
        };

        while let Some(front) = buffered.front() {
            if front.final_update_id < snapshot_last_id {
                buffered.pop_front();
            } else {
                break;
            }
        }

        let Some(first) = buffered.front() else {
            // Snapshot held, waiting for diffs.
            self.inner = Inner::Pending {
                snapshot_last_id,
                bids,
                asks,
                buffered,
            };
            return ApplyOutcome::Buffered;
        };

        if first.first_update_id > snapshot_last_id {
            // Snapshot is older than the diff stream — gap. Drop the
            // snapshot, keep buffered diffs for the next snapshot attempt.
            self.inner = Inner::NoSnapshot { buffered };
            return ApplyOutcome::ResyncRequired;
        }

        let mut bids = bids;
        let mut asks = asks;
        let mut last_id = snapshot_last_id;
        let mut prev_u: Option<i64> = None;
        while let Some(diff) = buffered.pop_front() {
            if let Some(p) = prev_u
                && diff.previous_final_update_id != p
            {
                // Chain broken inside buffered diffs — preserve this diff
                // and the rest for the next snapshot.
                let mut remaining = VecDeque::with_capacity(buffered.len() + 1);
                remaining.push_back(diff);
                remaining.extend(buffered);
                self.inner = Inner::NoSnapshot {
                    buffered: remaining,
                };
                return ApplyOutcome::ResyncRequired;
            }
            apply_diff_to_sides(&mut bids, &mut asks, &diff);
            last_id = diff.final_update_id;
            prev_u = Some(diff.final_update_id);
        }

        self.inner = Inner::Synced {
            last_update_id: last_id,
            bids,
            asks,
        };
        ApplyOutcome::Synced
    }
}

fn sides_from_snapshot(
    snapshot: OrderBook,
) -> (BTreeMap<Decimal, Decimal>, BTreeMap<Decimal, Decimal>) {
    let bids = snapshot
        .bids
        .into_iter()
        .map(|l| (l.price(), l.qty()))
        .collect();
    let asks = snapshot
        .asks
        .into_iter()
        .map(|l| (l.price(), l.qty()))
        .collect();
    (bids, asks)
}

fn apply_diff_to_sides(
    bids: &mut BTreeMap<Decimal, Decimal>,
    asks: &mut BTreeMap<Decimal, Decimal>,
    diff: &DepthUpdateMsg,
) {
    for level in &diff.bids {
        apply_side(bids, level.price(), level.qty());
    }
    for level in &diff.asks {
        apply_side(asks, level.price(), level.qty());
    }
}

fn apply_side(side: &mut BTreeMap<Decimal, Decimal>, price: Decimal, qty: Decimal) {
    if qty.is_zero() {
        side.remove(&price);
    } else {
        side.insert(price, qty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derivatives::usds_margined_futures::http::OrderLevel;

    fn snapshot(last_id: i64) -> OrderBook {
        OrderBook {
            last_update_id: last_id,
            event_time: 0,
            transaction_time: 0,
            bids: vec![level("100", "1")],
            asks: vec![level("101", "2")],
        }
    }

    fn level(price: &str, qty: &str) -> OrderLevel {
        // OrderLevel is a tuple-struct deserialized from `[price, qty]`.
        // Construct via JSON round-trip to avoid needing a public constructor.
        let json = format!("[\"{price}\", \"{qty}\"]");
        serde_json::from_str(&json).unwrap()
    }

    fn diff(prev_u: i64, u_first: i64, u_last: i64) -> DepthUpdateMsg {
        DepthUpdateMsg {
            event_time: 0,
            transaction_time: 0,
            symbol: "BTCUSDT".into(),
            first_update_id: u_first,
            final_update_id: u_last,
            previous_final_update_id: prev_u,
            bids: vec![],
            asks: vec![],
        }
    }

    #[test]
    fn snapshot_then_bridging_diff() {
        let mut book = OrderBookState::new();
        assert_eq!(book.apply_snapshot(snapshot(100)), ApplyOutcome::Buffered);
        assert!(!book.is_synced());
        // Diff spans the snapshot: U <= 100 <= u
        assert_eq!(book.apply_diff(diff(99, 95, 105)), ApplyOutcome::Synced);
        assert!(book.is_synced());
        assert_eq!(book.last_update_id(), Some(105));
    }

    #[test]
    fn diffs_then_snapshot() {
        let mut book = OrderBookState::new();
        assert_eq!(book.apply_diff(diff(89, 90, 95)), ApplyOutcome::Buffered);
        assert_eq!(book.apply_diff(diff(95, 96, 105)), ApplyOutcome::Buffered);
        // Snapshot at 100 lands inside the second buffered diff.
        assert_eq!(book.apply_snapshot(snapshot(100)), ApplyOutcome::Synced);
        assert_eq!(book.last_update_id(), Some(105));
    }

    #[test]
    fn snapshot_older_than_buffered_stream_triggers_resync() {
        let mut book = OrderBookState::new();
        // Buffered events have U > snapshot.last_update_id — gap.
        assert_eq!(book.apply_diff(diff(199, 200, 210)), ApplyOutcome::Buffered);
        assert_eq!(
            book.apply_snapshot(snapshot(100)),
            ApplyOutcome::ResyncRequired
        );
        // Buffered diff preserved for the next snapshot.
        assert!(!book.is_synced());

        // A fresher snapshot bridges.
        assert_eq!(book.apply_snapshot(snapshot(205)), ApplyOutcome::Synced);
        assert_eq!(book.last_update_id(), Some(210));
    }

    #[test]
    fn live_chain_break_triggers_resync() {
        let mut book = OrderBookState::new();
        book.apply_snapshot(snapshot(100));
        book.apply_diff(diff(99, 95, 105));
        assert!(book.is_synced());
        // Next diff has pu=106 != last=105 — chain broken.
        assert_eq!(
            book.apply_diff(diff(106, 110, 115)),
            ApplyOutcome::ResyncRequired
        );
        assert!(!book.is_synced());
    }

    #[test]
    fn stale_diff_after_sync_is_ignored() {
        let mut book = OrderBookState::new();
        book.apply_snapshot(snapshot(100));
        book.apply_diff(diff(99, 95, 105));
        assert_eq!(book.apply_diff(diff(89, 90, 95)), ApplyOutcome::Ignored);
        assert_eq!(book.last_update_id(), Some(105));
    }

    #[test]
    fn duplicate_snapshot_is_ignored() {
        let mut book = OrderBookState::new();
        book.apply_snapshot(snapshot(100));
        assert_eq!(book.apply_snapshot(snapshot(100)), ApplyOutcome::Ignored);
        assert_eq!(book.apply_snapshot(snapshot(50)), ApplyOutcome::Ignored);
    }

    #[test]
    fn newer_snapshot_replaces_synced_book() {
        let mut book = OrderBookState::new();
        book.apply_snapshot(snapshot(100));
        book.apply_diff(diff(99, 95, 105));
        assert!(book.is_synced());
        // User refetches and gets a fresher snapshot during live operation.
        assert_eq!(book.apply_snapshot(snapshot(200)), ApplyOutcome::Applied);
        assert_eq!(book.last_update_id(), Some(200));
    }
}
