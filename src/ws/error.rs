#[derive(Debug, thiserror::Error)]
/// Errors returned by this product's clients.
pub enum Error {
    #[error("invalid URL: {0}")]
    /// The URL is not a valid WebSocket URL.
    InvalidUrl(String),

    #[error("command queue full - backpressure limit reached")]
    /// The command queue is full (`try_*` methods).
    QueueFull,

    #[error("driver has shut down")]
    /// The driver task has stopped.
    DriverGone,

    #[error("websocket error: {0}")]
    /// WebSocket protocol or transport error.
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("io error: {0}")]
    /// I/O error.
    Io(#[from] std::io::Error),
}
