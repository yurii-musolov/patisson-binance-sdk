//! Keep a user data stream listenKey alive in the background.
//!
//! A listenKey (margin, USD-M and COIN-M futures) expires 60 minutes after
//! its last keepalive, and the stream then stops with `listenKeyExpired`.
//! [`ListenKeyKeeper`] runs the keepalive on a timer so callers don't have
//! to write that task themselves.

use std::{future::Future, time::Duration};

use tokio::{sync::mpsc, task::JoinHandle, time::MissedTickBehavior};

/// Interval Binance recommends between keepalives (the key lives 60 min).
pub const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// Background task that calls a keepalive on a fixed interval.
///
/// Failed keepalives are reported through [`ListenKeyKeeper::next_error`];
/// the task keeps running, so a transient error is retried on the next tick.
/// Dropping the keeper stops the task.
///
/// ```no_run
/// # async fn run() -> Result<(), binance::derivatives::usds_margined_futures::Error> {
/// use binance::{
///     Environment, SensitiveString,
///     derivatives::usds_margined_futures::http::{PrivateClient, PrivateConfig},
/// };
/// # let (api_key, api_secret) = (SensitiveString::from("k"), SensitiveString::from("s"));
/// let cfg = PrivateConfig::for_env(Environment::Production, api_key, api_secret).unwrap();
/// let client = PrivateClient::new(cfg)?;
/// let listen_key = client.create_listen_key().await?.result.listen_key;
/// let mut keeper = client.keep_listen_key_alive();
/// // ... connect to the user data stream with `listen_key` ...
/// while let Some(error) = keeper.next_error().await {
///     eprintln!("listenKey keepalive failed: {error}");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ListenKeyKeeper<E> {
    task: JoinHandle<()>,
    errors: mpsc::Receiver<E>,
}

impl<E: Send + 'static> ListenKeyKeeper<E> {
    /// Call `keepalive` every `interval`, starting one `interval` from now
    /// (a freshly created key needs no immediate keepalive).
    pub fn spawn<F, Fut, T>(interval: Duration, mut keepalive: F) -> Self
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send,
    {
        // Errors are rare (one per interval at most); a small buffer is
        // enough, and if nobody reads them the oldest are kept, not the task.
        let (tx, errors) = mpsc::channel(16);
        let task = tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval_at(tokio::time::Instant::now() + interval, interval);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                if let Err(error) = keepalive().await {
                    tracing::warn!("listenKey keepalive failed");
                    let _ = tx.try_send(error);
                }
            }
        });
        Self { task, errors }
    }

    /// The next failed keepalive. Waits until one happens; returns `None`
    /// only if the task has stopped.
    pub async fn next_error(&mut self) -> Option<E> {
        self.errors.recv().await
    }

    /// Stop the keepalive task.
    pub fn stop(self) {}
}

impl<E> Drop for ListenKeyKeeper<E> {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    };

    #[tokio::test(start_paused = true)]
    async fn calls_keepalive_on_every_interval_and_reports_errors() {
        let calls = Arc::new(AtomicU32::new(0));
        let counter = calls.clone();
        let mut keeper = ListenKeyKeeper::spawn(Duration::from_secs(60), move || {
            let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
            async move {
                if n == 2 {
                    Err(format!("call {n}"))
                } else {
                    Ok(())
                }
            }
        });

        tokio::time::sleep(Duration::from_secs(59)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 0, "no keepalive right away");
        assert_eq!(keeper.next_error().await.as_deref(), Some("call 2"));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        tokio::time::sleep(Duration::from_secs(61)).await;
        assert_eq!(
            calls.load(Ordering::SeqCst),
            3,
            "keeps running after an error"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn dropping_the_keeper_stops_the_task() {
        let calls = Arc::new(AtomicU32::new(0));
        let counter = calls.clone();
        let keeper = ListenKeyKeeper::<()>::spawn(Duration::from_secs(60), move || {
            counter.fetch_add(1, Ordering::SeqCst);
            async { Ok(()) }
        });
        tokio::time::sleep(Duration::from_secs(61)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        keeper.stop();
        tokio::time::sleep(Duration::from_secs(600)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
