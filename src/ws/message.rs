pub trait ReceivedMessage {
    fn server_shutdown_event_time(&self) -> Option<u64>;
}

#[derive(Debug)]
pub enum Command<T> {
    Connect,
    Send(T),
    Disconnect,
}

#[derive(Debug)]
pub enum Event<T> {
    Connected,
    Message(T),
    /// A WebSocket text frame arrived but could not be deserialized.
    /// The connection stays open — the raw error description is included.
    ParseError(String),
    Reconnecting {
        attempt: u32,
        delay_ms: u64,
    },
    Disconnected {
        reason: DisconnectReason,
    },
}

#[derive(Debug, Clone)]
pub enum DisconnectReason {
    Requested,
    RemoteClosed,
    PongTimeout,
    Error(String),
}
