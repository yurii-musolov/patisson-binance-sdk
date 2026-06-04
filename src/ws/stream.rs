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
use std::fmt::Debug;
use std::time::Duration;
use tokio::{
    sync::mpsc,
    time::{Instant, sleep, timeout},
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
pub struct Stream<C, M>
where
    C: Serialize + Send + Debug + 'static,
    M: ReceivedMessage + DeserializeOwned + Send + Debug + 'static,
{
    config: Config,
    cmd_rx: mpsc::Receiver<Command<C>>,
    evt_tx: mpsc::Sender<Event<M>>,
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
                State::Closing { sink } => self.step_closing(sink).await,
                State::Done => break,
            };
        }

        info!("stream shut down");
    }

    fn emit(&self, event: Event<M>) {
        if let Err(e) = self.evt_tx.try_send(event) {
            match e {
                mpsc::error::TrySendError::Full(dropped) => {
                    warn!("event queue full, dropping event: {:?}", dropped);
                }
                mpsc::error::TrySendError::Closed(_) => {
                    debug!("event receiver dropped");
                }
            }
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
        debug!(attempt, "connecting…");

        match connect_async(&self.config.url).await {
            Ok((ws_stream, _)) => {
                info!("websocket connected");
                self.emit(Event::Connected);

                let (sink, stream) = ws_stream.split();
                let (frame_tx, frame_rx) =
                    mpsc::channel::<FrameResult>(self.config.event_queue_size);

                let read_task = tokio::spawn(async move {
                    let mut stream = stream;
                    while let Some(msg) = stream.next().await {
                        if frame_tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                });

                State::Connected {
                    frame_rx,
                    read_task,
                    sink: Box::new(sink),
                }
            }
            Err(e) => {
                error!(error = %e, attempt, "connection failed");
                self.next_reconnect_state(attempt + 1, e.to_string())
            }
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

        loop {
            tokio::select! {
                biased;

                frame = frame_rx.recv() => match frame {
                    None => {
                        info!("remote closed the connection");
                        read_task.abort();
                        return self.next_reconnect_state(1, "remote closed".into());
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "websocket read error");
                        read_task.abort();
                        return self.next_reconnect_state(1, e.to_string());
                    }
                    Some(Ok(msg)) => match msg {
                        Message::Ping(bytes) => {
                            debug!("protocol ping received ({}B)", bytes.len());
                            if let Err(e) = sink.send(Message::Pong(bytes)).await {
                                error!(error = %e, "send protocol pong failed");
                                read_task.abort();
                                return self.next_reconnect_state(1, e.to_string());
                            }
                            hb = HeartbeatState::PongSent;
                            ping_timer.as_mut().reset(far_future_instant());
                            pong_timeout.as_mut().reset(Instant::now() + pong_timeout_dur);
                        }
                        Message::Text(json) => {
                            match deserialize_json::<M>(&json) {
                                Ok(msg) => {
                                    if let Some(_) = msg.server_shutdown_event_time() {
                                        info!("server shutdown notice received, initiating reconnect");
                                        self.emit(Event::Message(msg));
                                        read_task.abort();
                                        return self.next_reconnect_state(1, "server shutdown".into());
                                    }else{
                                        self.emit(Event::Message(msg))
                                    }
                                }
                                Err(e) => {
                                    warn!(error = %e, "parsing IncomingMessage failed");
                                    self.emit(Event::ParseError(e.to_string()));
                                }
                            }
                        }
                        Message::Pong(bytes) => debug!("pong received ({}B)", bytes.len()),
                        Message::Binary(bytes) => debug!("binary message received ({}B)", bytes.len()),
                        Message::Close(close_frame) => {
                            debug!(?close_frame, "close frame received");
                            read_task.abort();
                            return self.next_reconnect_state(1, "remote close frame".into());
                        }
                        Message::Frame(frame) => debug!("frame received ({}B)", frame.len()),
                    },
                },

                cmd = self.cmd_rx.recv() => match cmd {
                    None | Some(Command::Disconnect) => {
                        info!("disconnect requested");
                        read_task.abort();
                        return State::Closing { sink };
                    }
                    Some(Command::Send(msg)) => {
                        let json = serialize_json(&msg).expect("serialize outgoing message failed");
                        let msg = Message::Text(json.into());
                        if let Err(e) = sink.send(msg).await {
                            error!(error = %e, "send error");
                            read_task.abort();
                            return self.next_reconnect_state(1, e.to_string());
                        }
                    }
                    Some(Command::Connect) => warn!("Connect ignored - already connected")
                },

                _ = ping_timer.as_mut(), if matches!(hb, HeartbeatState::Idle) => {
                    warn!("no ping received within ping_interval - connection assumed dead");
                    self.emit(Event::Disconnected { reason: DisconnectReason::PongTimeout });
                    read_task.abort();
                    return self.next_reconnect_state(1, "ping interval exceeded".into());
                }

                _ = pong_timeout.as_mut(), if matches!(hb, HeartbeatState::PongSent) => {
                    warn!("no ping received within pong_timeout after last pong - connection assumed dead");
                    self.emit(Event::Disconnected { reason: DisconnectReason::PongTimeout });
                    read_task.abort();
                    return self.next_reconnect_state(1, "pong timeout".into());
                }

                _ = ttl_timer.as_mut() => {
                    info!("connection TTL reached, reconnecting proactively");
                    read_task.abort();
                    return self.next_reconnect_state(1, "connection TTL reached".into());
                }
            }
        }
    }

    async fn step_reconnecting(&mut self, attempt: u32, delay_ms: u64) -> State {
        warn!(attempt, delay_ms, "waiting before reconnect");
        self.emit(Event::Reconnecting { attempt, delay_ms });

        let cancelled = tokio::select! {
            _ = sleep(Duration::from_millis(delay_ms)) => false,
            cmd = self.cmd_rx.recv() => matches!(cmd, None | Some(Command::Disconnect)),
        };

        if cancelled {
            self.emit(Event::Disconnected {
                reason: DisconnectReason::Requested,
            });
            State::Idle
        } else {
            State::Connecting { attempt }
        }
    }

    async fn step_closing(&mut self, mut sink: Sink) -> State {
        if let Err(e) = sink.send(Message::Close(None)).await {
            error!(error = %e, "send close message failed");
        }
        if let Err(e) = timeout(self.config.close_timeout, self.cmd_rx.recv()).await {
            error!(error = %e, "waiting for a clean close handshake failed");
        }

        self.emit(Event::Disconnected {
            reason: DisconnectReason::Requested,
        });
        State::Idle
    }

    fn next_reconnect_state(&self, next_attempt: u32, reason: String) -> State {
        if self.config.max_reconnect_attempts == 0
            || next_attempt > self.config.max_reconnect_attempts
        {
            self.emit(Event::Disconnected {
                reason: DisconnectReason::Error(String::from(
                    "all reconnection attempts have failed",
                )),
            });
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

const FAR_FUTURE: Duration = Duration::from_secs(u64::MAX / 4);

#[inline]
fn far_future_instant() -> Instant {
    Instant::now() + FAR_FUTURE
}
