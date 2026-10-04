use super::{
    Command, Config, DisconnectReason, Event, Handle,
    state::{FrameResult, HeartbeatState, Sink, State},
};
use crate::{
    serde::{deserialize_json, serialize_json},
    ws::ReceivedMessage,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::time::Duration;
use tokio::{
    sync::mpsc,
    time::{Instant, sleep, sleep_until, timeout},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// General WSS information
/// A single connection to stream.binance.com is only valid for 24 hours; expect to be disconnected at the 24 hour mark.
/// A serverShutdown event will be sent 10 minutes before disconnection. Please establish a new connection as soon as possible to prevent interruption.
///     The WebSocket server will send a ping frame every 20 seconds.
///     If the WebSocket server does not receive a pong frame back from the connection within a minute the connection will be disconnected.
///     When you receive a ping, you must send a pong with a copy of ping's payload as soon as possible.
///     Unsolicited pong frames are allowed, but will not prevent disconnection. It is recommended that the payload for these pong frames are empty.
///
/// # Connection lifecycle
///
/// The driver answers server pings, reconnects with exponential back-off
/// after a lost connection (`Event::ConnectionLost` followed by
/// `Event::Reconnecting`) and renews the connection before the 24h limit.
/// `Event::Disconnected` is only emitted once the driver has stopped.
///
/// Subscriptions are **not** restored after a reconnect: send the
/// `SUBSCRIBE` requests again on every `Event::Connected` (or use a URL that
/// carries the stream names). Messages sent while a reconnect is pending are
/// queued and delivered once the connection is back.
pub struct Stream<C, M>
where
    C: Serialize + Send + Debug + 'static,
    M: ReceivedMessage + DeserializeOwned + Send + Debug + 'static,
{
    config: Config,
    cmd_rx: mpsc::Receiver<Command<C>>,
    evt_tx: mpsc::Sender<Event<M>>,
    /// Outgoing messages accepted while a reconnect was pending; flushed in
    /// order right after the next successful connect.
    pending: VecDeque<C>,
    /// Data events dropped since the last `Event::Lagged` was delivered.
    lagged: u64,
}

impl<C, M> Stream<C, M>
where
    C: Serialize + Send + Debug + 'static,
    M: ReceivedMessage + DeserializeOwned + Send + Debug + 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: Config) -> (Handle<C>, mpsc::Receiver<Event<M>>) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command<C>>(config.command_queue_size);
        let (evt_tx, evt_rx) = mpsc::channel::<Event<M>>(config.event_queue_size);

        let stream = Self {
            config,
            cmd_rx,
            evt_tx,
            pending: VecDeque::new(),
            lagged: 0,
        };

        tokio::spawn(stream.run());

        (Handle::<C>::new(cmd_tx), evt_rx)
    }

    async fn run(mut self) {
        info!("stream started");
        let mut state = State::Idle;

        loop {
            state = match state {
                State::Idle => self.step_idle().await,
                State::Connecting { attempt } => self.step_connecting(attempt).await,
                State::Connected {
                    frame_rx,
                    read_task,
                    sink,
                } => self.step_connected(frame_rx, read_task, sink).await,
                State::Reconnecting { attempt, delay_ms } => {
                    self.step_reconnecting(attempt, delay_ms).await
                }
                State::Closing {
                    frame_rx,
                    read_task,
                    sink,
                } => self.step_closing(frame_rx, read_task, sink).await,
                State::Done => break,
            };
        }

        info!("stream shut down");
    }

    /// Deliver a lifecycle event (`Connected`, `Reconnecting`,
    /// `Disconnected`). These are never dropped: the driver waits for room in
    /// the event queue, so a consumer that stops reading also stalls the
    /// driver.
    async fn emit(&mut self, event: Event<M>) {
        if self.lagged > 0 {
            let dropped = std::mem::take(&mut self.lagged);
            if self.evt_tx.send(Event::Lagged { dropped }).await.is_err() {
                return;
            }
        }
        if self.evt_tx.send(event).await.is_err() {
            debug!("event receiver dropped");
        }
    }

    /// Deliver a data event (`Message`, `ParseError`) without blocking the
    /// socket. When the queue is full the event is dropped and counted; the
    /// count is reported as `Event::Lagged` as soon as there is room again.
    fn emit_data(&mut self, event: Event<M>) {
        if self.lagged > 0 {
            match self.evt_tx.try_send(Event::Lagged {
                dropped: self.lagged,
            }) {
                Ok(()) => self.lagged = 0,
                Err(mpsc::error::TrySendError::Full(_)) => {
                    self.lagged += 1;
                    return;
                }
                Err(mpsc::error::TrySendError::Closed(_)) => return,
            }
        }
        match self.evt_tx.try_send(event) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                if self.lagged == 0 {
                    warn!("event queue full, dropping events until the consumer catches up");
                }
                self.lagged += 1;
            }
            Err(mpsc::error::TrySendError::Closed(_)) => debug!("event receiver dropped"),
        }
    }

    async fn step_idle(&mut self) -> State {
        loop {
            match self.cmd_rx.recv().await {
                Some(Command::Connect) => {
                    return State::Connecting { attempt: 1 };
                }
                None => return State::Done,
                Some(Command::Disconnect) => {
                    warn!("Command disconnect ignored - not connected");
                }
                Some(Command::Send(_)) => {
                    warn!("Send ignored - not connected");
                }
            }
        }
    }

    async fn step_connecting(&mut self, attempt: u32) -> State {
        debug!(attempt, "connecting");

        let ws_stream =
            match timeout(self.config.connect_timeout, connect_async(&self.config.url)).await {
                Ok(Ok((ws_stream, _))) => ws_stream,
                Ok(Err(e)) => {
                    error!(error = %e, attempt, "connection failed");
                    return self.next_reconnect_state(attempt + 1, e.to_string()).await;
                }
                Err(_) => {
                    error!(attempt, "connection attempt timed out");
                    return self
                        .next_reconnect_state(attempt + 1, "connect timed out".into())
                        .await;
                }
            };

        info!("websocket connected");
        self.emit(Event::Connected).await;

        let (sink, stream) = ws_stream.split();
        let mut sink: Sink = Box::new(sink);
        let (frame_tx, frame_rx) = mpsc::channel::<FrameResult>(self.config.event_queue_size);
        let read_task = tokio::spawn(async move {
            let mut stream = stream;
            while let Some(msg) = stream.next().await {
                if frame_tx.send(msg).await.is_err() {
                    break;
                }
            }
        });

        if let Err(e) = self.flush_pending(&mut sink).await {
            error!(error = %e, "sending queued messages failed");
            return self
                .connection_lost(read_task, DisconnectReason::Error(e.to_string()))
                .await;
        }

        State::Connected {
            frame_rx,
            read_task,
            sink,
        }
    }

    async fn step_connected(
        &mut self,
        mut frame_rx: mpsc::Receiver<FrameResult>,
        read_task: tokio::task::JoinHandle<()>,
        mut sink: Sink,
    ) -> State {
        let ping_interval = self.config.ping_interval;
        let pong_timeout_dur = self.config.pong_timeout;
        let ttl_dur = self.config.connection_ttl;

        // ping_timer is active in HeartbeatState::Idle and tracks the deadline
        // for the *first* server ping. pong_timeout is active in
        // HeartbeatState::PongSent (i.e. after we have responded to at least
        // one ping) and tracks the deadline for the *next* server ping.
        // The inactive timer is parked at FAR_FUTURE so it never fires.
        let mut ping_timer = Box::pin(sleep(ping_interval));
        let mut pong_timeout = Box::pin(sleep(FAR_FUTURE));
        let mut ttl_timer = Box::pin(sleep(ttl_dur));
        let mut hb = HeartbeatState::Idle;

        let lost = loop {
            tokio::select! {
                biased;

                frame = frame_rx.recv() => match frame {
                    None => {
                        info!("remote closed the connection");
                        break DisconnectReason::RemoteClosed;
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "websocket read error");
                        break DisconnectReason::Error(e.to_string());
                    }
                    Some(Ok(msg)) => match msg {
                        Message::Ping(bytes) => {
                            debug!("protocol ping received ({}B)", bytes.len());
                            if let Err(e) = sink.send(Message::Pong(bytes)).await {
                                error!(error = %e, "send protocol pong failed");
                                break DisconnectReason::Error(e.to_string());
                            }
                            hb = HeartbeatState::PongSent;
                            ping_timer.as_mut().reset(far_future_instant());
                            pong_timeout.as_mut().reset(Instant::now() + pong_timeout_dur);
                        }
                        Message::Text(json) => match deserialize_json::<M>(&json) {
                            Ok(msg) => {
                                let shutdown = msg.server_shutdown_event_time().is_some();
                                self.emit_data(Event::Message(msg));
                                if shutdown {
                                    info!("server shutdown notice received, initiating reconnect");
                                    break DisconnectReason::ServerShutdown;
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, "parsing IncomingMessage failed");
                                self.emit_data(Event::ParseError(e.to_string()));
                            }
                        },
                        Message::Pong(bytes) => debug!("pong received ({}B)", bytes.len()),
                        Message::Binary(bytes) => debug!("binary message received ({}B)", bytes.len()),
                        Message::Close(close_frame) => {
                            debug!(?close_frame, "close frame received");
                            break DisconnectReason::RemoteClosed;
                        }
                        Message::Frame(frame) => debug!("frame received ({}B)", frame.len()),
                    },
                },

                cmd = self.cmd_rx.recv() => match cmd {
                    None | Some(Command::Disconnect) => {
                        info!("disconnect requested");
                        // Keep the reader alive: it delivers the server's
                        // close frame that completes the handshake.
                        return State::Closing { frame_rx, read_task, sink };
                    }
                    Some(Command::Send(msg)) => {
                        let frame = match encode(&msg) {
                            Ok(frame) => frame,
                            Err(error) => {
                                self.emit(Event::SendFailed { error }).await;
                                continue;
                            }
                        };
                        if let Err(e) = sink.send(frame).await {
                            error!(error = %e, "send error");
                            // The message never left: retry it after reconnect.
                            self.pending.push_front(msg);
                            break DisconnectReason::Error(e.to_string());
                        }
                    }
                    Some(Command::Connect) => warn!("Connect ignored - already connected")
                },

                _ = ping_timer.as_mut(), if matches!(hb, HeartbeatState::Idle) => {
                    warn!("no ping received within ping_interval - connection assumed dead");
                    break DisconnectReason::PongTimeout;
                }

                _ = pong_timeout.as_mut(), if matches!(hb, HeartbeatState::PongSent) => {
                    warn!("no ping received within pong_timeout after last pong - connection assumed dead");
                    break DisconnectReason::PongTimeout;
                }

                _ = ttl_timer.as_mut() => {
                    info!("connection TTL reached, reconnecting proactively");
                    break DisconnectReason::ConnectionTtl;
                }
            }
        };

        self.connection_lost(read_task, lost).await
    }

    /// An established connection ended without the user asking for it:
    /// report it and schedule a reconnect.
    async fn connection_lost(
        &mut self,
        read_task: tokio::task::JoinHandle<()>,
        reason: DisconnectReason,
    ) -> State {
        read_task.abort();
        let detail = format!("{reason:?}");
        self.emit(Event::ConnectionLost { reason }).await;
        self.next_reconnect_state(1, detail).await
    }

    async fn step_reconnecting(&mut self, attempt: u32, delay_ms: u64) -> State {
        warn!(attempt, delay_ms, "waiting before reconnect");
        self.emit(Event::Reconnecting { attempt, delay_ms }).await;

        // Keep serving commands for the whole back-off: a Send must not cut
        // the delay short nor be lost, it is queued and flushed once the
        // connection is back.
        let wake_at = sleep_until(Instant::now() + Duration::from_millis(delay_ms));
        tokio::pin!(wake_at);
        loop {
            tokio::select! {
                _ = &mut wake_at => return State::Connecting { attempt },
                cmd = self.cmd_rx.recv() => match cmd {
                    None | Some(Command::Disconnect) => {
                        self.pending.clear();
                        self.emit(Event::Disconnected {
                            reason: DisconnectReason::Requested,
                        }).await;
                        return State::Idle;
                    }
                    Some(Command::Connect) => debug!("Connect ignored - reconnect already scheduled"),
                    Some(Command::Send(msg)) => self.queue_pending(msg),
                },
            }
        }
    }

    /// Remember an outgoing message until the connection is re-established.
    /// The queue is bounded by `command_queue_size`; the oldest message is
    /// dropped when it overflows.
    fn queue_pending(&mut self, msg: C) {
        if self.pending.len() >= self.config.command_queue_size.max(1)
            && let Some(dropped) = self.pending.pop_front()
        {
            warn!(
                ?dropped,
                "pending queue full, dropping oldest outgoing message"
            );
        }
        self.pending.push_back(msg);
    }

    /// Send every queued message in order. On failure the unsent message is
    /// put back at the front so it is retried after the next reconnect.
    async fn flush_pending(
        &mut self,
        sink: &mut Sink,
    ) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        while let Some(msg) = self.pending.pop_front() {
            let frame = match encode(&msg) {
                Ok(frame) => frame,
                Err(error) => {
                    self.emit(Event::SendFailed { error }).await;
                    continue;
                }
            };
            if let Err(e) = sink.send(frame).await {
                self.pending.push_front(msg);
                return Err(e);
            }
        }
        Ok(())
    }

    /// Send a close frame and wait (up to `close_timeout`) for the server to
    /// answer with its own close frame. Commands are deliberately not read
    /// here: anything queued meanwhile (e.g. a `Connect` right after
    /// `Disconnect`) is handled once the driver is back in `Idle`.
    async fn step_closing(
        &mut self,
        mut frame_rx: mpsc::Receiver<FrameResult>,
        read_task: tokio::task::JoinHandle<()>,
        mut sink: Sink,
    ) -> State {
        match sink.send(Message::Close(None)).await {
            Err(e) => error!(error = %e, "send close message failed"),
            Ok(()) => {
                let handshake = async {
                    while let Some(frame) = frame_rx.recv().await {
                        if matches!(frame, Ok(Message::Close(_)) | Err(_)) {
                            break;
                        }
                    }
                };
                if timeout(self.config.close_timeout, handshake).await.is_err() {
                    warn!("no close frame from the server within close_timeout");
                }
            }
        }
        read_task.abort();

        self.emit(Event::Disconnected {
            reason: DisconnectReason::Requested,
        })
        .await;
        State::Idle
    }

    async fn next_reconnect_state(&mut self, next_attempt: u32, reason: String) -> State {
        if self.config.max_reconnect_attempts == 0
            || next_attempt > self.config.max_reconnect_attempts
        {
            self.pending.clear();
            self.emit(Event::Disconnected {
                reason: DisconnectReason::Error(String::from(
                    "all reconnection attempts have failed",
                )),
            })
            .await;
            return State::Idle;
        }

        let base_ms = self.config.reconnect_base_delay.as_millis() as u64;
        let max_ms = self.config.reconnect_max_delay.as_millis() as u64;
        let delay_ms = (base_ms.saturating_mul(1u64 << (next_attempt - 1).min(10))).min(max_ms);

        debug!(next_attempt, delay_ms, reason, "scheduling reconnect");
        State::Reconnecting {
            attempt: next_attempt,
            delay_ms,
        }
    }
}

/// Serialize an outgoing message. A message that can't be serialized can
/// never be sent; callers report it as `Event::SendFailed` and drop it
/// instead of bringing the driver down.
fn encode<C: Serialize + Debug>(msg: &C) -> Result<Message, String> {
    serialize_json(msg)
        .map(|json| Message::Text(json.into()))
        .map_err(|e| {
            error!(error = %e, ?msg, "serialize outgoing message failed");
            e.to_string()
        })
}

const FAR_FUTURE: Duration = Duration::from_secs(u64::MAX / 4);

#[inline]
fn far_future_instant() -> Instant {
    Instant::now() + FAR_FUTURE
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::{Value, json};
    use tokio::net::TcpListener;

    #[derive(Debug, Deserialize)]
    struct TestMsg(#[allow(dead_code)] Value);

    impl ReceivedMessage for TestMsg {
        fn server_shutdown_event_time(&self) -> Option<u64> {
            None
        }
    }

    /// What the test server does with the n-th accepted connection.
    #[derive(Clone, Copy)]
    enum Behaviour {
        /// Complete the handshake, then close the connection at once.
        CloseImmediately,
        /// Read frames until the client goes away, reporting each one.
        Record,
        /// Send this many text messages, then behave like `Record`.
        Burst(usize),
    }

    #[derive(Debug)]
    enum ServerEvent {
        Accepted(usize, Instant),
        Text(usize, String),
        #[allow(dead_code)]
        Closed(usize),
    }

    /// Spawn a local WebSocket server. Connection `n` follows `behaviours[n]`
    /// (the last entry repeats). Returns the `ws://` URL and the event feed.
    async fn spawn_server(
        behaviours: Vec<Behaviour>,
    ) -> (String, mpsc::UnboundedReceiver<ServerEvent>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut n = 0;
            while let Ok((tcp, _)) = listener.accept().await {
                let behaviour = behaviours[n.min(behaviours.len() - 1)];
                let tx = tx.clone();
                let Ok(mut ws) = tokio_tungstenite::accept_async(tcp).await else {
                    continue;
                };
                let _ = tx.send(ServerEvent::Accepted(n, Instant::now()));
                let id = n;
                tokio::spawn(async move {
                    if let Behaviour::CloseImmediately = behaviour {
                        let _ = ws.close(None).await;
                    }
                    if let Behaviour::Burst(count) = behaviour {
                        for i in 0..count {
                            let text = json!({ "i": i }).to_string();
                            if ws.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    if let Behaviour::Record | Behaviour::Burst(_) = behaviour {
                        while let Some(Ok(msg)) = ws.next().await {
                            if let Message::Text(text) = msg {
                                let _ = tx.send(ServerEvent::Text(id, text.to_string()));
                            }
                        }
                    }
                    drop(ws);
                    let _ = tx.send(ServerEvent::Closed(id));
                });
                n += 1;
            }
        });
        (url, rx)
    }

    fn test_config(url: String) -> Config {
        Config::new(url)
            .reconnect_base_delay(Duration::from_millis(150))
            .close_timeout(Duration::from_millis(500))
    }

    async fn next_event(rx: &mut mpsc::Receiver<Event<TestMsg>>) -> Event<TestMsg> {
        timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("timed out waiting for a driver event")
            .expect("driver event channel closed")
    }

    async fn next_server_event(rx: &mut mpsc::UnboundedReceiver<ServerEvent>) -> ServerEvent {
        timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("timed out waiting for a server event")
            .expect("server event channel closed")
    }

    #[tokio::test]
    async fn send_during_backoff_is_delivered_after_reconnect() {
        let (url, mut server) =
            spawn_server(vec![Behaviour::CloseImmediately, Behaviour::Record]).await;
        let (handle, mut events) = Stream::<Value, TestMsg>::new(test_config(url));
        handle.connect().await.unwrap();

        assert!(matches!(next_event(&mut events).await, Event::Connected));
        let closed_at = loop {
            if let Event::Reconnecting { .. } = next_event(&mut events).await {
                break Instant::now();
            }
        };

        // Arrives while the driver is sleeping before the reconnect.
        let subscribe = json!({"method": "SUBSCRIBE", "params": ["btcusdt@trade"], "id": 1});
        handle.send_command(subscribe.clone()).await.unwrap();
        // A Connect during back-off must not cut the delay short either.
        handle.connect().await.unwrap();

        let mut second_accept = None;
        let text = loop {
            match next_server_event(&mut server).await {
                ServerEvent::Accepted(1, at) => second_accept = Some(at),
                ServerEvent::Text(1, text) => break text,
                _ => {}
            }
        };
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), subscribe);
        let waited = second_accept.unwrap() - closed_at;
        assert!(
            waited >= Duration::from_millis(100),
            "back-off was cut short: reconnected after {waited:?}"
        );

        handle.disconnect().await.unwrap();
    }

    #[tokio::test]
    async fn connect_right_after_disconnect_reconnects() {
        let (url, mut server) = spawn_server(vec![Behaviour::Record]).await;
        let (handle, mut events) = Stream::<Value, TestMsg>::new(test_config(url));
        handle.connect().await.unwrap();
        assert!(matches!(next_event(&mut events).await, Event::Connected));

        let started = Instant::now();
        handle.disconnect().await.unwrap();
        handle.connect().await.unwrap();

        assert!(matches!(
            next_event(&mut events).await,
            Event::Disconnected {
                reason: DisconnectReason::Requested
            }
        ));
        // The server answered the close frame, so the driver must not have
        // sat out the whole close_timeout.
        assert!(started.elapsed() < Duration::from_millis(400));
        assert!(matches!(next_event(&mut events).await, Event::Connected));
        loop {
            if let ServerEvent::Accepted(1, _) = next_server_event(&mut server).await {
                break;
            }
        }

        handle.disconnect().await.unwrap();
    }

    #[tokio::test]
    async fn overflow_is_reported_as_lagged_and_lifecycle_events_survive() {
        const SENT: u64 = 50;
        let (url, _server) = spawn_server(vec![Behaviour::Burst(SENT as usize)]).await;
        let cfg = test_config(url).event_queue_size(4);
        let (handle, mut events) = Stream::<Value, TestMsg>::new(cfg);
        handle.connect().await.unwrap();

        // Let the burst overflow the queue while nobody reads it, then ask
        // for a disconnect: its lifecycle event must still get through.
        sleep(Duration::from_millis(300)).await;
        handle.disconnect().await.unwrap();

        let (mut received, mut dropped) = (0, 0);
        loop {
            match next_event(&mut events).await {
                Event::Connected => {}
                Event::Message(_) => received += 1,
                Event::Lagged { dropped: n } => dropped += n,
                Event::Disconnected {
                    reason: DisconnectReason::Requested,
                } => break,
                other => panic!("unexpected event {other:?}"),
            }
        }
        assert!(dropped > 0, "the burst should have overflowed the queue");
        assert_eq!(received + dropped, SENT);
    }

    #[tokio::test]
    async fn remote_close_reports_connection_lost_and_reconnects() {
        let (url, _server) =
            spawn_server(vec![Behaviour::CloseImmediately, Behaviour::Record]).await;
        let (handle, mut events) = Stream::<Value, TestMsg>::new(test_config(url));
        handle.connect().await.unwrap();

        assert!(matches!(next_event(&mut events).await, Event::Connected));
        let lost = next_event(&mut events).await;
        assert!(
            matches!(
                lost,
                Event::ConnectionLost {
                    reason: DisconnectReason::RemoteClosed
                }
            ),
            "got {lost:?}"
        );
        assert!(matches!(
            next_event(&mut events).await,
            Event::Reconnecting { attempt: 1, .. }
        ));
        // A transient loss must not look like the terminal Disconnected.
        assert!(matches!(next_event(&mut events).await, Event::Connected));

        handle.disconnect().await.unwrap();
    }

    #[tokio::test]
    async fn stalled_handshake_hits_connect_timeout() {
        // Accepts TCP connections but never answers the WebSocket handshake.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let mut held = Vec::new();
            while let Ok((tcp, _)) = listener.accept().await {
                held.push(tcp);
            }
        });

        let cfg = test_config(url)
            .connect_timeout(Duration::from_millis(200))
            .max_reconnect_attempts(1);
        let (handle, mut events) = Stream::<Value, TestMsg>::new(cfg);
        handle.connect().await.unwrap();

        match next_event(&mut events).await {
            Event::Disconnected {
                reason: DisconnectReason::Error(_),
            } => {}
            other => panic!("unexpected event {other:?}"),
        }
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(untagged)]
    enum Outgoing {
        Good(Value),
        /// serde_json rejects non-string map keys.
        Bad(std::collections::BTreeMap<Vec<u8>, u8>),
    }

    #[tokio::test]
    async fn unserializable_message_is_reported_not_fatal() {
        let (url, mut server) = spawn_server(vec![Behaviour::Record]).await;
        let (handle, mut events) = Stream::<Outgoing, TestMsg>::new(test_config(url));
        handle.connect().await.unwrap();
        assert!(matches!(next_event(&mut events).await, Event::Connected));

        let bad = Outgoing::Bad([(vec![1], 1)].into_iter().collect());
        handle.send_command(bad).await.unwrap();
        assert!(matches!(
            next_event(&mut events).await,
            Event::SendFailed { .. }
        ));

        // The driver is still alive and keeps sending.
        handle
            .send_command(Outgoing::Good(json!({"id": 2})))
            .await
            .unwrap();
        loop {
            if let ServerEvent::Text(0, text) = next_server_event(&mut server).await {
                assert_eq!(text, r#"{"id":2}"#);
                break;
            }
        }

        handle.disconnect().await.unwrap();
    }
}
