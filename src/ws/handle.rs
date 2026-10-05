use serde::Serialize;
use std::fmt::Debug;
use tokio::sync::mpsc;

use crate::ws;

pub struct Handle<C>
where
    C: Serialize + Send + Debug + 'static,
{
    cmd_tx: mpsc::Sender<ws::Command<C>>,
}

// Not derived: `#[derive(Clone)]` would require `C: Clone`, but only the
// channel sender is cloned.
impl<C> Clone for Handle<C>
where
    C: Serialize + Send + Debug + 'static,
{
    fn clone(&self) -> Self {
        Self {
            cmd_tx: self.cmd_tx.clone(),
        }
    }
}

impl<C> Handle<C>
where
    C: Serialize + Send + Debug + 'static,
{
    pub fn new(cmd_tx: mpsc::Sender<ws::Command<C>>) -> Self {
        Self { cmd_tx }
    }

    pub async fn connect(&self) -> Result<(), ws::Error> {
        let cmd = ws::Command::Connect;
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| ws::Error::DriverGone)
    }

    pub fn try_connect(&self) -> Result<(), ws::Error> {
        let cmd = ws::Command::Connect;
        self.cmd_tx.try_send(cmd).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => ws::Error::QueueFull,
            mpsc::error::TrySendError::Closed(_) => ws::Error::DriverGone,
        })
    }

    pub async fn disconnect(&self) -> Result<(), ws::Error> {
        let cmd = ws::Command::Disconnect;
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| ws::Error::DriverGone)
    }

    pub fn try_disconnect(&self) -> Result<(), ws::Error> {
        let cmd = ws::Command::Disconnect;
        self.cmd_tx.try_send(cmd).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => ws::Error::QueueFull,
            mpsc::error::TrySendError::Closed(_) => ws::Error::DriverGone,
        })
    }

    /// Send the messages built by `builder` right after every (re)connect,
    /// and immediately if already connected. Use it for subscriptions, which
    /// don't survive a reconnect; the builder runs on each connect, so it can
    /// sign requests with a fresh timestamp. Replaces any previous builder.
    pub async fn on_connect<F>(&self, builder: F) -> Result<(), ws::Error>
    where
        F: Fn() -> Vec<C> + Send + 'static,
    {
        self.cmd_tx
            .send(ws::Command::OnConnect(Some(Box::new(builder))))
            .await
            .map_err(|_| ws::Error::DriverGone)
    }

    /// Stop sending on-connect messages.
    pub async fn clear_on_connect(&self) -> Result<(), ws::Error> {
        self.cmd_tx
            .send(ws::Command::OnConnect(None))
            .await
            .map_err(|_| ws::Error::DriverGone)
    }

    pub async fn send_command(&self, msg: C) -> Result<(), ws::Error> {
        let cmd = ws::Command::Send(msg);
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| ws::Error::DriverGone)
    }

    pub fn try_send_command(&self, msg: C) -> Result<(), ws::Error> {
        let cmd = ws::Command::Send(msg);
        self.cmd_tx.try_send(cmd).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => ws::Error::QueueFull,
            mpsc::error::TrySendError::Closed(_) => ws::Error::DriverGone,
        })
    }
}
