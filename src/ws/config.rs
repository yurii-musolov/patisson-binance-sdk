use crate::SensitiveString;
use std::time::Duration;

/// Spot / margin streams: the server pings every 20s.
pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(20);
/// Spot / margin streams: the server drops the connection if no pong
/// arrives within a minute.
pub const DEFAULT_PONG_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_CONNECTION_TTL: Duration = Duration::from_hours(24);

/// USD-M / COIN-M futures streams: the server pings every 3 minutes. The
/// deadline for the first ping leaves room for one late ping.
pub const FUTURES_PING_INTERVAL: Duration = Duration::from_mins(5);
/// USD-M / COIN-M futures streams: the server drops the connection if no
/// pong arrives within 10 minutes.
pub const FUTURES_PONG_TIMEOUT: Duration = Duration::from_mins(10);

#[derive(Debug, Clone)]
pub struct Config {
    /// WebSocket server URL
    pub url: String,

    /// Maximum number of pending commands (backpressure)
    pub command_queue_size: usize,

    /// Maximum number of buffered events (backpressure)
    pub event_queue_size: usize,

    /// Maximum reconnect attempts (0 = no reconnect)
    pub max_reconnect_attempts: u32,

    /// Base delay between reconnect attempts
    pub reconnect_base_delay: Duration,

    /// Maximum delay cap for exponential back-off
    pub reconnect_max_delay: Duration,

    /// How long to wait for a clean close handshake
    pub close_timeout: Duration,

    /// Time budget for one connection attempt (TCP + TLS + WebSocket handshake)
    pub connect_timeout: Duration,

    /// HTTP proxy (`http://[user:password@]host:port`) to tunnel the
    /// connection through with `CONNECT`. Kept as a [`SensitiveString`]
    /// since it may carry credentials.
    pub proxy: Option<SensitiveString>,

    /// Expected interval between heartbeat pings from the server.
    /// Per Binance Spot docs, the server sends a ping every 20s. Used as the
    /// initial deadline for the first server ping; if exceeded, the connection
    /// is assumed dead and a reconnect is scheduled.
    pub ping_interval: Duration,

    /// Maximum time to wait for the next ping after we last responded with a
    /// pong before declaring the connection dead. Per Binance Spot docs, the
    /// server disconnects if no pong is received within 60s.
    pub pong_timeout: Duration,

    /// Maximum lifetime of a single websocket connection before a proactive
    /// reconnect. Per Binance Spot docs, a connection is only valid for 24h.
    pub connection_ttl: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: String::new(),
            command_queue_size: 64,
            event_queue_size: 256,
            max_reconnect_attempts: 5,
            reconnect_base_delay: Duration::from_millis(500),
            reconnect_max_delay: Duration::from_secs(30),
            close_timeout: Duration::from_secs(5),
            connect_timeout: Duration::from_secs(10),
            proxy: None,
            ping_interval: DEFAULT_PING_INTERVAL,
            pong_timeout: DEFAULT_PONG_TIMEOUT,
            connection_ttl: DEFAULT_CONNECTION_TTL,
        }
    }
}

impl Config {
    /// Config with Spot heartbeat defaults (same as [`Config::spot`]). Use
    /// [`Config::futures`] for USD-M / COIN-M futures streams: with Spot
    /// timings a futures connection is declared dead long before the first
    /// server ping arrives.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Config for Spot and margin streams (`stream.binance.com`).
    pub fn spot(url: impl Into<String>) -> Self {
        Self::new(url)
    }

    /// Config for USD-M and COIN-M futures streams (`fstream.binance.com`,
    /// `dstream.binance.com`), whose server pings only every 3 minutes.
    pub fn futures(url: impl Into<String>) -> Self {
        Self::new(url)
            .ping_interval(FUTURES_PING_INTERVAL)
            .pong_timeout(FUTURES_PONG_TIMEOUT)
    }

    pub fn command_queue_size(mut self, n: usize) -> Self {
        self.command_queue_size = n;
        self
    }

    pub fn event_queue_size(mut self, n: usize) -> Self {
        self.event_queue_size = n;
        self
    }

    pub fn max_reconnect_attempts(mut self, n: u32) -> Self {
        self.max_reconnect_attempts = n;
        self
    }

    pub fn reconnect_base_delay(mut self, d: Duration) -> Self {
        self.reconnect_base_delay = d;
        self
    }

    pub fn reconnect_max_delay(mut self, d: Duration) -> Self {
        self.reconnect_max_delay = d;
        self
    }

    pub fn close_timeout(mut self, d: Duration) -> Self {
        self.close_timeout = d;
        self
    }

    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = d;
        self
    }

    /// Connect through an HTTP proxy (`CONNECT` tunnel), e.g.
    /// `http://user:password@proxy:8080`.
    pub fn proxy(mut self, url: impl Into<SensitiveString>) -> Self {
        self.proxy = Some(url.into());
        self
    }

    pub fn ping_interval(mut self, d: Duration) -> Self {
        self.ping_interval = d;
        self
    }

    pub fn pong_timeout(mut self, d: Duration) -> Self {
        self.pong_timeout = d;
        self
    }

    pub fn connection_ttl(mut self, d: Duration) -> Self {
        self.connection_ttl = d;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spot_preset_matches_spot_heartbeat() {
        let cfg = Config::spot("wss://example.com");
        assert_eq!(cfg.ping_interval, DEFAULT_PING_INTERVAL);
        assert_eq!(cfg.pong_timeout, DEFAULT_PONG_TIMEOUT);
    }

    #[test]
    fn futures_preset_tolerates_three_minute_pings() {
        let cfg = Config::futures("wss://example.com");
        assert_eq!(cfg.url, "wss://example.com");
        assert!(cfg.ping_interval > Duration::from_mins(3));
        assert!(cfg.pong_timeout > Duration::from_mins(3));
        assert_eq!(cfg.connection_ttl, DEFAULT_CONNECTION_TTL);
    }
}
