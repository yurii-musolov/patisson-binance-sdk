//! Optional client-side rate limiter mirroring Binance's request-weight,
//! order-count, and raw-request buckets.
//!
//! Attach an [`Arc<RateLimiter>`] to a product `Config` via its
//! `rate_limiter(...)` builder to enable it. Calls that would exceed local
//! budget are rejected up front with `Error::RateLimited` (RejectFast mode)
//! before they reach Binance — the goal is to avoid burning the request on a
//! response that's already doomed and risking a 429 → 418 IP ban.
//!
//! Buckets reset on fixed windows. After every response the limiter folds
//! `X-MBX-USED-WEIGHT-*` / `X-MBX-ORDER-COUNT-*` back in via [`observe`], so
//! transient drift (lost requests, replays) self-heals on the next call.
//! 429 / 418 responses call [`freeze_until`] with the server's `Retry-After`,
//! blocking all acquires until then.
//!
//! [`observe`]: RateLimiter::observe
//! [`freeze_until`]: RateLimiter::freeze_until

use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// What an endpoint consumes from the limiter.
///
/// `weight` charges the per-IP `REQUEST_WEIGHT` bucket; `orders` charges the
/// per-UID `ORDERS` bucket. `RAW_REQUESTS` buckets always charge 1 per call,
/// independent of `Cost`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cost {
    pub weight: u32,
    pub orders: u32,
}

impl Cost {
    /// Charges nothing — useful for endpoints whose cost is fully reimbursed
    /// or for unit-testing the wiring.
    pub const FREE: Self = Self {
        weight: 0,
        orders: 0,
    };

    pub const fn weight(weight: u32) -> Self {
        Self { weight, orders: 0 }
    }

    pub const fn weight_and_orders(weight: u32, orders: u32) -> Self {
        Self { weight, orders }
    }
}

/// Which Binance bucket a [`BucketSpec`] tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketKind {
    /// IP-scoped `REQUEST_WEIGHT` bucket. Charged by `Cost::weight`.
    Weight,
    /// UID-scoped `ORDERS` bucket. Charged by `Cost::orders`.
    Orders,
    /// IP-scoped `RAW_REQUESTS` bucket. Always charged 1 per call.
    RawRequests,
}

/// One Binance limit declaration: "at most `limit` uses of `kind` per `interval`".
///
/// You can construct these from the `rateLimits` array returned by
/// `exchangeInfo` (see [`RateLimiter::new`]) or hand-roll them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketSpec {
    pub kind: BucketKind,
    pub interval: Duration,
    pub limit: u32,
}

impl BucketSpec {
    pub const fn new(kind: BucketKind, interval: Duration, limit: u32) -> Self {
        Self {
            kind,
            interval,
            limit,
        }
    }
}

/// Reason an acquire was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitSource {
    /// Local budget exhausted — the request was never sent.
    Local,
    /// Server returned 429/418 and the limiter is frozen until the
    /// `Retry-After` deadline passes.
    Server,
}

/// Outcome of a refused acquire — how long to back off and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimited {
    pub retry_after: Duration,
    pub source: RateLimitSource,
}

/// Observed usage extracted from response headers. Keys are the bucket's
/// interval (1m, 10s, 1d, …); values are the latest authoritative count.
///
/// Construct this in the HTTP layer by parsing `X-MBX-USED-WEIGHT-(NUM)(UNIT)`
/// and `X-MBX-ORDER-COUNT-(NUM)(UNIT)` and pass it to
/// [`RateLimiter::observe`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ObservedUsage {
    pub weight: BTreeMap<Duration, u32>,
    pub orders: BTreeMap<Duration, u32>,
}

#[derive(Debug)]
struct Bucket {
    spec: BucketSpec,
    used: u32,
    window_end: Instant,
}

impl Bucket {
    fn new(spec: BucketSpec, now: Instant) -> Self {
        Self {
            spec,
            used: 0,
            window_end: now + spec.interval,
        }
    }

    /// Advance to the current window. Binance uses fixed windows, so after
    /// `window_end` the count resets to 0.
    fn roll(&mut self, now: Instant) {
        if now < self.window_end {
            return;
        }
        // Advance by however many whole windows have elapsed so the boundary
        // stays aligned even after long idleness.
        let interval = self.spec.interval;
        let past = now - (self.window_end - interval);
        let windows = (past.as_nanos() / interval.as_nanos()).max(1) as u32;
        self.window_end += interval * windows;
        self.used = 0;
    }

    fn need(&self, cost: Cost) -> u32 {
        match self.spec.kind {
            BucketKind::Weight => cost.weight,
            BucketKind::Orders => cost.orders,
            BucketKind::RawRequests => 1,
        }
    }
}

#[derive(Debug)]
struct Inner {
    buckets: Vec<Bucket>,
    frozen_until: Option<Instant>,
}

/// Opaque, thread-safe limiter. Share one across clients with `Arc<RateLimiter>`.
#[derive(Debug)]
pub struct RateLimiter {
    inner: Mutex<Inner>,
}

impl RateLimiter {
    pub fn new(specs: impl IntoIterator<Item = BucketSpec>) -> Self {
        Self::new_at(specs, Instant::now())
    }

    pub(crate) fn new_at(specs: impl IntoIterator<Item = BucketSpec>, now: Instant) -> Self {
        let buckets = specs.into_iter().map(|s| Bucket::new(s, now)).collect();
        Self {
            inner: Mutex::new(Inner {
                buckets,
                frozen_until: None,
            }),
        }
    }

    /// Conservative spot defaults: 1200 weight/min, 50 orders/10s, 160k orders/day.
    /// Use [`RateLimiter::new`] with the `rateLimits` from `exchangeInfo` for
    /// the exact values currently in force.
    pub fn spot_defaults() -> Self {
        Self::new([
            BucketSpec::new(BucketKind::Weight, Duration::from_secs(60), 1200),
            BucketSpec::new(BucketKind::Orders, Duration::from_secs(10), 50),
            BucketSpec::new(BucketKind::Orders, Duration::from_secs(86_400), 160_000),
        ])
    }

    /// Conservative USDⓈ-M / COIN-M futures defaults: 2400 weight/min, 1200 orders/min.
    pub fn futures_defaults() -> Self {
        Self::new([
            BucketSpec::new(BucketKind::Weight, Duration::from_secs(60), 2400),
            BucketSpec::new(BucketKind::Orders, Duration::from_secs(60), 1200),
        ])
    }

    /// Try to charge `cost` against every applicable bucket atomically. On
    /// failure no buckets are mutated.
    pub fn try_acquire(&self, cost: Cost) -> Result<(), RateLimited> {
        self.try_acquire_at(cost, Instant::now())
    }

    pub(crate) fn try_acquire_at(&self, cost: Cost, now: Instant) -> Result<(), RateLimited> {
        let mut inner = self.inner.lock().expect("rate limiter mutex poisoned");

        if let Some(until) = inner.frozen_until {
            if now < until {
                return Err(RateLimited {
                    retry_after: until - now,
                    source: RateLimitSource::Server,
                });
            }
            inner.frozen_until = None;
        }

        let mut worst_wait: Option<Duration> = None;
        for bucket in &mut inner.buckets {
            bucket.roll(now);
            let need = bucket.need(cost);
            if need == 0 {
                continue;
            }
            if bucket.used.saturating_add(need) > bucket.spec.limit {
                let wait = bucket.window_end - now;
                worst_wait = Some(worst_wait.map_or(wait, |w| w.max(wait)));
            }
        }
        if let Some(retry_after) = worst_wait {
            return Err(RateLimited {
                retry_after,
                source: RateLimitSource::Local,
            });
        }

        for bucket in &mut inner.buckets {
            let need = bucket.need(cost);
            bucket.used = bucket.used.saturating_add(need);
        }
        Ok(())
    }

    /// Fold server-authoritative usage back in. Local counts can only be
    /// raised — the limiter is always conservative.
    pub fn observe(&self, headers: &ObservedUsage) {
        self.observe_at(headers, Instant::now());
    }

    pub(crate) fn observe_at(&self, headers: &ObservedUsage, now: Instant) {
        let mut inner = self.inner.lock().expect("rate limiter mutex poisoned");
        for bucket in &mut inner.buckets {
            bucket.roll(now);
            let observed = match bucket.spec.kind {
                BucketKind::Weight => headers.weight.get(&bucket.spec.interval).copied(),
                BucketKind::Orders => headers.orders.get(&bucket.spec.interval).copied(),
                BucketKind::RawRequests => None,
            };
            if let Some(obs) = observed
                && obs > bucket.used
            {
                bucket.used = obs;
            }
        }
    }

    /// Block all acquires until `until`. Subsequent freezes only extend, never
    /// shorten, the deadline.
    pub fn freeze_until(&self, until: Instant) {
        let mut inner = self.inner.lock().expect("rate limiter mutex poisoned");
        inner.frozen_until = Some(match inner.frozen_until {
            Some(prev) => prev.max(until),
            None => until,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limiter() -> (RateLimiter, Instant) {
        let now = Instant::now();
        let rl = RateLimiter::new_at(
            [
                BucketSpec::new(BucketKind::Weight, Duration::from_secs(60), 100),
                BucketSpec::new(BucketKind::Orders, Duration::from_secs(10), 5),
            ],
            now,
        );
        (rl, now)
    }

    #[test]
    fn acquire_charges_weight_and_orders() {
        let (rl, now) = limiter();
        rl.try_acquire_at(Cost::weight_and_orders(10, 1), now)
            .unwrap();
        rl.try_acquire_at(Cost::weight_and_orders(10, 1), now)
            .unwrap();

        let inner = rl.inner.lock().unwrap();
        assert_eq!(inner.buckets[0].used, 20);
        assert_eq!(inner.buckets[1].used, 2);
    }

    #[test]
    fn acquire_rejects_when_weight_exhausted() {
        let (rl, now) = limiter();
        rl.try_acquire_at(Cost::weight(90), now).unwrap();
        let err = rl
            .try_acquire_at(Cost::weight(20), now)
            .expect_err("budget should be exceeded");
        assert_eq!(err.source, RateLimitSource::Local);
        assert!(err.retry_after <= Duration::from_secs(60));
    }

    #[test]
    fn acquire_rejects_when_orders_exhausted() {
        let (rl, now) = limiter();
        rl.try_acquire_at(Cost::weight_and_orders(1, 5), now)
            .unwrap();
        let err = rl
            .try_acquire_at(Cost::weight_and_orders(1, 1), now)
            .unwrap_err();
        assert_eq!(err.source, RateLimitSource::Local);
    }

    #[test]
    fn rejected_acquire_does_not_charge_any_bucket() {
        let (rl, now) = limiter();
        // Weight would fit but orders would overflow → both must stay untouched.
        rl.try_acquire_at(Cost::weight_and_orders(1, 5), now)
            .unwrap();
        let _ = rl.try_acquire_at(Cost::weight_and_orders(1, 1), now);

        let inner = rl.inner.lock().unwrap();
        assert_eq!(
            inner.buckets[0].used, 1,
            "weight must not be charged on failure"
        );
        assert_eq!(inner.buckets[1].used, 5, "orders must not be charged twice");
    }

    #[test]
    fn window_rollover_resets_used() {
        let (rl, now) = limiter();
        rl.try_acquire_at(Cost::weight(100), now).unwrap();
        // Fresh window — should be allowed again.
        rl.try_acquire_at(Cost::weight(100), now + Duration::from_secs(61))
            .unwrap();
    }

    #[test]
    fn observe_raises_local_count() {
        let (rl, now) = limiter();
        rl.try_acquire_at(Cost::weight(10), now).unwrap();

        let mut obs = ObservedUsage::default();
        obs.weight.insert(Duration::from_secs(60), 80);
        rl.observe_at(&obs, now);

        // After observation we should now fail a 30-weight call (10 used + 80 observed = 80 > 70).
        let err = rl.try_acquire_at(Cost::weight(30), now).unwrap_err();
        assert_eq!(err.source, RateLimitSource::Local);
    }

    #[test]
    fn observe_never_lowers_local_count() {
        let (rl, now) = limiter();
        rl.try_acquire_at(Cost::weight(50), now).unwrap();

        let mut obs = ObservedUsage::default();
        obs.weight.insert(Duration::from_secs(60), 10);
        rl.observe_at(&obs, now);

        let inner = rl.inner.lock().unwrap();
        assert_eq!(
            inner.buckets[0].used, 50,
            "observation must not roll back local state"
        );
    }

    #[test]
    fn freeze_blocks_all_acquires() {
        let (rl, now) = limiter();
        rl.freeze_until(now + Duration::from_secs(30));
        let err = rl.try_acquire_at(Cost::weight(1), now).unwrap_err();
        assert_eq!(err.source, RateLimitSource::Server);
        assert!(err.retry_after <= Duration::from_secs(30));
    }

    #[test]
    fn freeze_clears_after_deadline() {
        let (rl, now) = limiter();
        rl.freeze_until(now + Duration::from_secs(30));
        rl.try_acquire_at(Cost::weight(1), now + Duration::from_secs(31))
            .expect("freeze should have lifted");
    }

    #[test]
    fn freeze_only_extends_never_shortens() {
        let (rl, now) = limiter();
        rl.freeze_until(now + Duration::from_secs(30));
        rl.freeze_until(now + Duration::from_secs(5));
        // The earlier deadline must still be in force.
        let err = rl
            .try_acquire_at(Cost::weight(1), now + Duration::from_secs(10))
            .unwrap_err();
        assert_eq!(err.source, RateLimitSource::Server);
    }

    #[test]
    fn zero_cost_passes_when_not_frozen() {
        let (rl, now) = limiter();
        rl.try_acquire_at(Cost::weight(100), now).unwrap();
        // Bucket is now full but Cost::FREE charges nothing.
        rl.try_acquire_at(Cost::FREE, now).unwrap();
    }
}
