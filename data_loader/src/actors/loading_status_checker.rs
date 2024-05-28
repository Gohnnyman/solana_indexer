use anyhow::Result;
use log::info;
use tokio::sync::mpsc;

use crate::{register::Register, storages::queue_storage::QueueStorage};

struct LoadingStatusChecker {
    receiver: mpsc::Receiver<LoadingStatusCheckerMessage>,
    queue_storage: QueueStorage,
}

enum LoadingStatusCheckerMessage {
    ResetLoadingStatus,
}

impl LoadingStatusChecker {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<LoadingStatusCheckerMessage>,
    ) -> Result<Self> {
        let queue_storage =
            QueueStorage::new(&register.config.get_queue_storage_config().database_url).await?;
        Ok(LoadingStatusChecker {
            receiver,
            queue_storage,
        })
    }

    fn handle_message(&mut self, msg: LoadingStatusCheckerMessage) -> Result<()> {
        match msg {
            LoadingStatusCheckerMessage::ResetLoadingStatus => {
                self.reset_loading_status()?;
            }
        }

        Ok(())
    }

    async fn run(&mut self) {
        info!("Loading status checker started");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).unwrap();
        }
        info!("Loading status checker stopped");
    }

    fn reset_loading_status(&self) -> Result<()> {
        self.queue_storage.reset_loading_status()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct LoadingStatusCheckerHandle {
    sender: mpsc::Sender<LoadingStatusCheckerMessage>,
}

impl LoadingStatusCheckerHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(16);
        let mut loading_status_checker = LoadingStatusChecker::new(register, receiver).await?;
        tokio::spawn(async move { loading_status_checker.run().await });

        Ok(Self { sender })
    }

    pub async fn reset_loading_status(&self) {
        let msg = LoadingStatusCheckerMessage::ResetLoadingStatus;

        let _ = self.sender.send(msg).await;
    }
}
