use crate::{register::Register, storages::queue_storage::*};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio::sync::{mpsc, oneshot};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SavedState {
    pub newest_transaction: Option<Signature>,
    pub before: Option<Signature>,
    pub until: Option<Signature>,
}

struct SavedStateManager {
    receiver: mpsc::Receiver<SavedStateManagerMessage>,
    queue_storage: QueueStorage,
}

enum SavedStateManagerMessage {
    LoadState {
        program_address: Pubkey,
        respond_to: oneshot::Sender<SavedState>,
    },
}

impl SavedStateManager {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<SavedStateManagerMessage>,
    ) -> Result<SavedStateManager> {
        let queue_storage =
            QueueStorage::new(&register.config.get_queue_storage_config().database_url).await?;
        Ok(SavedStateManager {
            receiver,
            queue_storage,
        })
    }

    fn handle_message(&mut self, msg: SavedStateManagerMessage) {
        match msg {
            SavedStateManagerMessage::LoadState {
                program_address,
                respond_to,
            } => {
                let saved_state = self.load_state(program_address);
                let _ = respond_to.send(saved_state);
            }
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }

    fn load_state(&self, program_address: Pubkey) -> SavedState {
        let downloading_status = self
            .queue_storage
            .load_downloading_status(&program_address.to_string());

        match downloading_status {
            Some(downloading_status) => {
                if let Ok(result) = serde_json::from_str(&downloading_status) {
                    result
                } else {
                    SavedState {
                        newest_transaction: None,
                        before: None,
                        until: None,
                    }
                }
            }
            None => SavedState {
                newest_transaction: None,
                before: None,
                until: None,
            },
        }
    }
}

#[derive(Clone)]
pub struct SavedStateManagerHandle {
    sender: mpsc::Sender<SavedStateManagerMessage>,
}

impl SavedStateManagerHandle {
    pub async fn new(register: &Register) -> Result<SavedStateManagerHandle> {
        let (sender, receiver) = mpsc::channel(16);
        let mut saved_state_manager = SavedStateManager::new(register, receiver).await?;
        tokio::spawn(async move { saved_state_manager.run().await });

        Ok(Self { sender })
    }

    pub async fn load_state(&self, program_address: Pubkey) -> SavedState {
        let (sender, receiver) = oneshot::channel();
        let msg = SavedStateManagerMessage::LoadState {
            program_address,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;
        receiver
            .await
            .expect("SavedStateManager task has been killed")
    }
}
