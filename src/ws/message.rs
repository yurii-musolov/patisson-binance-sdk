pub trait ReceivedMessage {
    fn server_shutdown_event_time(&self) -> Option<u64>;
}

/// Builds the messages to send right after every (re)connect; see
/// [`crate::ws::Handle::on_connect`].
pub type OnConnect<T> = Box<dyn Fn() -> Vec<T> + Send>;

pub enum Command<T> {
    Connect,
    Send(T),
    Disconnect,
    /// Replace (or with `None`, clear) the on-connect messages builder.
    OnConnect(Option<OnConnect<T>>),
}

impl<T: std::fmt::Debug> std::fmt::Debug for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect => f.write_str("Connect"),
            Self::Send(msg) => f.debug_tuple("Send").field(msg).finish(),
            Self::Disconnect => f.write_str("Disconnect"),
            Self::OnConnect(builder) => f
                .debug_tuple("OnConnect")
                .field(&builder.as_ref().map(|_| "<fn>"))
                .finish(),
        }
    }
}

#[derive(Debug)]
pub enum Event<T> {
    Message(T),
    /// A connection was established (on-connect messages, see
    /// `Handle::on_connect`, have already been sent).
    Connected,
    /// A WebSocket text frame arrived but could not be deserialized.
    /// The connection stays open — the raw error description is included.
    ParseError(String),
    /// The event queue was full and `dropped` data events (`Message` /
    /// `ParseError`) were discarded. Lifecycle events are never dropped. For
    /// stateful consumers (order books, user data) treat this like a gap and
    /// resynchronize.
    Lagged {
        dropped: u64,
    },
    /// An outgoing message could not be serialized and was dropped.
    SendFailed {
        error: String,
    },
    /// An established connection ended without being asked to. The driver
    /// reconnects on its own (see `Reconnecting`); subscriptions made with
    /// `SUBSCRIBE` are restored only if registered with `Handle::on_connect`. Stateful consumers should resynchronize.
    ConnectionLost {
        reason: DisconnectReason,
    },
    /// The driver is waiting `delay_ms` before connection attempt `attempt`.
    Reconnecting {
        attempt: u32,
        delay_ms: u64,
    },
    /// The driver has stopped and is idle: either `disconnect()` was called
    /// (`Requested`) or every reconnect attempt failed (`Error`). Call
    /// `connect()` to start again.
    Disconnected {
        reason: DisconnectReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    /// The user called `disconnect()` or dropped every `Handle`.
    Requested,
    /// The server closed the connection (close frame or end of stream).
    RemoteClosed,
    /// No ping from the server within the configured heartbeat deadline.
    PongTimeout,
    /// The server announced maintenance (`serverShutdown` event).
    ServerShutdown,
    /// `connection_ttl` elapsed; the connection is renewed proactively.
    ConnectionTtl,
    /// Transport error, or every reconnect attempt failed.
    Error(String),
}
