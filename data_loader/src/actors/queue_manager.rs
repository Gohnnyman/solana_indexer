use crate::{register::Register, storages::queue_storage::*};
use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

struct QueueManager {
    receiver: mpsc::Receiver<QueueManagerMessage>,
    queue_storage: QueueStorage,
}

enum QueueManagerMessage {
    GetSignature {
        respond_to: oneshot::Sender<Option<String>>,
        load_only_successful_transactions: bool,
    },
    MarkSignatureAsLoaded {
        signature: String,
    },
    MarkSignatureLoadingFault {
        signature: String,
    },
}

impl QueueManager {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<QueueManagerMessage>,
    ) -> Result<Self> {
        Ok(QueueManager {
            receiver,
            queue_storage: QueueStorage::new(
                &register.config.get_queue_storage_config().database_url,
            )
            .await?,
        })
    }

    fn handle_message(&mut self, msg: QueueManagerMessage) -> Result<()> {
        match msg {
            QueueManagerMessage::GetSignature {
                respond_to,
                load_only_successful_transactions,
            } => {
                let signature = self
                    .queue_storage
                    .get_signature_from_queue(load_only_successful_transactions);
                let _ = respond_to.send(signature);
            }
            QueueManagerMessage::MarkSignatureAsLoaded { signature } => {
                self.queue_storage.mark_signature_as_loaded(signature)?;
            }
            QueueManagerMessage::MarkSignatureLoadingFault { signature } => {
                self.queue_storage.mark_signature_loading_fault(signature)?;
            }
        }

        Ok(())
    }

    async fn run(&mut self) {
        self.reset_status_loading_in_progress().unwrap();

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).unwrap();
        }
    }

    fn reset_status_loading_in_progress(&self) -> Result<()> {
        self.queue_storage.reset_status_loading_in_progress()?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct QueueManagerHandle {
    sender: mpsc::Sender<QueueManagerMessage>,
}

impl QueueManagerHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(100);
        let mut queue_manager = QueueManager::new(register, receiver).await?;
        tokio::spawn(async move { queue_manager.run().await });

        Ok(Self { sender })
    }

    pub async fn get_signature_from_queue(
        &self,
        load_only_successful_transactions: bool,
    ) -> Option<String> {
        let (sender, receiver) = oneshot::channel();
        let msg = QueueManagerMessage::GetSignature {
            respond_to: sender,
            load_only_successful_transactions,
        };

        let _ = self.sender.send(msg).await;
        receiver.await.expect("QueueManager task has been killed")
    }

    pub async fn mark_signature_as_loaded(&self, signature: String) {
        let msg = QueueManagerMessage::MarkSignatureAsLoaded { signature };
        let _ = self.sender.send(msg).await;
    }

    pub async fn mark_signature_loading_fault(&self, signature: String) {
        let msg = QueueManagerMessage::MarkSignatureLoadingFault { signature };
        let _ = self.sender.send(msg).await;
    }
}
