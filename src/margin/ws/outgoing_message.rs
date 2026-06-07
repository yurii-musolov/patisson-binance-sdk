use serde::Serialize;

/// Margin user data streams are server-push only: the client establishes the
/// connection with `listenKey` in the URL path and never sends application
/// messages. This empty enum encodes that at the type level — any
/// `Command::Send` call against a margin stream is unrepresentable.
///
/// Keep-alive is performed out-of-band via REST
/// (`PUT /sapi/v1/userDataStream`), not over the WebSocket.
#[derive(Serialize, Debug)]
pub enum OutgoingMessage {}
