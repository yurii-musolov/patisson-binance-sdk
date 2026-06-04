use std::time::Duration;

pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(20);
pub const DEFAULT_PONG_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_CONNECTION_TTL: Duration = Duration::from_hours(24);

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
            ping_interval: DEFAULT_PING_INTERVAL,
            pong_timeout: DEFAULT_PONG_TIMEOUT,
            connection_ttl: DEFAULT_CONNECTION_TTL,
        }
    }
}

impl Config {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
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
