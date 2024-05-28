use crate::{
    errors::QueueManagerError, metrics_update, register::Register,
    storages::postgre_storage::models::Delegation, storages::postgre_storage::*,
    storages::QueueStorage,
};
use anyhow::Result;
use macros::{ActorInstance, HandleInstance};
use serde::Deserialize;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use tokio::sync::{mpsc, oneshot};

#[derive(ActorInstance)]
struct QueueManager {
    receiver: mpsc::Receiver<QueueManagerMessage>,
    storage: Box<dyn QueueStorage>,
}

#[derive(Debug)]
enum QueueManagerMessage {
    GetTransactions {
        respond_to: oneshot::Sender<Vec<EncodedConfirmedTransactionWithStatusMeta>>,
    },
    GetDelegations {
        respond_to: oneshot::Sender<Result<Vec<Delegation>>>,
        stake_accs: Vec<String>,
    },
    SaveDelegations {
        respond_to: oneshot::Sender<Result<()>>,
        delegations: Vec<Delegation>,
    },
    MarkTransactionAsParsed {
        respond_to: oneshot::Sender<Result<()>>,
        transaction: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub enum StorageType {
    RabbitMQ,
    PostgreSQL,
}

impl QueueManager {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<QueueManagerMessage>,
    ) -> Result<Self> {
        let storage_type = register.config.get_storage_type();
        let storage: Box<dyn QueueStorage> = match storage_type {
            StorageType::RabbitMQ => {
                unreachable!()
            }
            StorageType::PostgreSQL => {
                let storage =
                    PostgreStorage::new(&register.config.get_queue_storage_config().storage_url)
                        .await?;
                Box::new(storage)
            }
        };

        metrics_update!(inc total ACTIVE_ACTOR_INSTANCES_COUNT, &["queue_manager"]);

        Ok(QueueManager { receiver, storage })
    }

    async fn handle_message(&mut self, msg: QueueManagerMessage) {
        match msg {
            QueueManagerMessage::GetTransactions { respond_to } => {
                let transaction = self.storage.get_transactions().await;

                let _ = respond_to.send(transaction);
            }
            QueueManagerMessage::GetDelegations {
                respond_to,
                stake_accs,
            } => {
                let delegations = self.storage.get_delegations(stake_accs).await;
                let _ = respond_to.send(delegations);
            }
            QueueManagerMessage::SaveDelegations {
                respond_to,
                delegations,
            } => {
                let result = self.storage.save_delegations(delegations).await;
                let _ = respond_to.send(result);
            }
            QueueManagerMessage::MarkTransactionAsParsed {
                respond_to,
                transaction,
            } => {
                let result = self.storage.mark_transaction_as_parsed(transaction).await;
                let _ = respond_to.send(result);
            }
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }
}

#[derive(HandleInstance)]
pub struct QueueManagerHandle {
    sender: mpsc::Sender<QueueManagerMessage>,
}

impl QueueManagerHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(100);
        let mut queue_manager = QueueManager::new(register, receiver).await?;
        tokio::spawn(async move { queue_manager.run().await });
        metrics_update!(inc total ACTIVE_HANDLE_INSTANCES_COUNT, &["queue_manager_handle"]);

        Ok(Self { sender })
    }

    pub async fn get_delegations(
        &mut self,
        stake_accs: Vec<String>,
    ) -> Result<Result<Vec<Delegation>>, QueueManagerError> {
        let (sender, receiver) = oneshot::channel();
        let msg = QueueManagerMessage::GetDelegations {
            respond_to: sender,
            stake_accs,
        };

        let _ = self.sender.send(msg).await;
        Ok(receiver.await?)
    }

    pub async fn save_delegations(
        &mut self,
        delegations: Vec<(String, Option<String>)>,
    ) -> Result<(), QueueManagerError> {
        let (sender, receiver) = oneshot::channel();
        let delegations = delegations
            .into_iter()
            .map(|(stake_acc, vote_acc)| Delegation {
                stake_acc,
                vote_acc,
            })
            .collect();

        let msg = QueueManagerMessage::SaveDelegations {
            respond_to: sender,
            delegations,
        };

        let _ = self.sender.send(msg).await;
        receiver.await??;
        Ok(())
    }

    pub async fn get_transactions(
        &mut self,
    ) -> Result<Vec<EncodedConfirmedTransactionWithStatusMeta>, QueueManagerError> {
        let (sender, receiver) = oneshot::channel();
        let msg = QueueManagerMessage::GetTransactions { respond_to: sender };

        let _ = self.sender.send(msg).await;
        Ok(receiver.await?)
    }

    pub async fn mark_transaction_as_parsed(
        &mut self,
        transaction: String,
    ) -> Result<(), QueueManagerError> {
        let (sender, receiver) = oneshot::channel();
        let msg = QueueManagerMessage::MarkTransactionAsParsed {
            respond_to: sender,
            transaction,
        };

        let _ = self.sender.send(msg).await;
        Ok(receiver.await??)
    }
}
