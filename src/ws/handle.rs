use serde::Serialize;
use std::fmt::Debug;
use tokio::sync::mpsc;

use crate::ws;

#[derive(Clone)]
pub struct Handle<C>
where
    C: Serialize + Send + Debug + 'static,
{
    cmd_tx: mpsc::Sender<ws::Command<C>>,
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
