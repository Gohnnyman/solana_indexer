use anyhow::Result;
use log::info;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use tokio::sync::{mpsc, oneshot};

use crate::{register::Register, storages::queue_storage::QueueStorage};

struct TransactionsSaver {
    receiver: mpsc::Receiver<TransactionsSaverMessage>,
    queue_storage: QueueStorage,
}

enum TransactionsSaverMessage {
    SaveTransaction {
        signature: String,
        transaction: EncodedConfirmedTransactionWithStatusMeta,
        respond_to: oneshot::Sender<String>,
    },
}

impl TransactionsSaver {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<TransactionsSaverMessage>,
    ) -> Result<Self> {
        let queue_storage =
            QueueStorage::new(&register.config.get_queue_storage_config().database_url).await?;

        Ok(TransactionsSaver {
            receiver,
            queue_storage,
        })
    }

    fn handle_message(&mut self, msg: TransactionsSaverMessage) -> Result<()> {
        match msg {
            TransactionsSaverMessage::SaveTransaction {
                signature,
                transaction,
                respond_to,
            } => {
                self.save_transaction(signature, transaction)?;
                let _ = respond_to.send(String::from("transaction saving"));
            }
        }

        Ok(())
    }

    async fn run(&mut self) {
        info!("Transaction saver started");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).unwrap();
        }
        info!("Transaction saver stopped");
    }

    fn save_transaction(
        &self,
        signature: String,
        transaction: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<()> {
        self.queue_storage
            .store_transaction(&signature, transaction)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct TransactionsSaverHandle {
    sender: mpsc::Sender<TransactionsSaverMessage>,
}

impl TransactionsSaverHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(16);
        let mut transactions_saver = TransactionsSaver::new(register, receiver).await?;
        tokio::spawn(async move { transactions_saver.run().await });

        Ok(Self { sender })
    }

    pub async fn save_transaction(
        &self,
        signature: String,
        transaction: EncodedConfirmedTransactionWithStatusMeta,
    ) -> String {
        let (sender, receiver) = oneshot::channel();
        let msg = TransactionsSaverMessage::SaveTransaction {
            signature,
            transaction,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;
        receiver
            .await
            .expect("SignaturesLoader task has been killed")
    }
}
