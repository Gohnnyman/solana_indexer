use anyhow::Result;
use log::info;
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::{mpsc, oneshot};

use crate::{register::Register, storages::queue_storage::QueueStorage};

use super::saved_state_manager::SavedState;

struct SignaturesSaver {
    receiver: mpsc::Receiver<SignaturesSaverMessage>,
    queue_storage: QueueStorage,
}

enum SignaturesSaverMessage {
    SaveSignaturesAndState {
        signatures: Vec<RpcConfirmedTransactionStatusWithSignature>,
        program_address: Pubkey,
        saved_state: Box<SavedState>,
        respond_to: oneshot::Sender<usize>,
    },
}

impl SignaturesSaver {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<SignaturesSaverMessage>,
    ) -> Result<Self> {
        let queue_storage =
            QueueStorage::new(&register.config.get_queue_storage_config().database_url).await?;
        Ok(SignaturesSaver {
            receiver,
            queue_storage,
        })
    }

    fn handle_message(&mut self, msg: SignaturesSaverMessage) -> Result<()> {
        match msg {
            SignaturesSaverMessage::SaveSignaturesAndState {
                signatures,
                program_address,
                saved_state,
                respond_to,
            } => {
                let signatures_stored =
                    self.save_signatures_and_state(signatures, program_address, *saved_state)?;
                let _ = respond_to.send(signatures_stored);
            }
        }

        Ok(())
    }

    async fn run(&mut self) {
        info!("Signatures saver started");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).unwrap();
        }
        info!("Signatures saver stopped");
    }

    fn save_signatures_and_state(
        &self,
        signatures: Vec<RpcConfirmedTransactionStatusWithSignature>,
        program_address: Pubkey,
        saved_state: SavedState,
    ) -> Result<usize> {
        let signatures_stored = self.queue_storage.store_signatures_and_state(
            &signatures,
            &program_address.to_string(),
            &serde_json::to_string(&saved_state)?,
        )?;

        Ok(signatures_stored)
    }
}

#[derive(Clone)]
pub struct SignaturesSaverHandle {
    sender: mpsc::Sender<SignaturesSaverMessage>,
}

impl SignaturesSaverHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(16);
        let mut signatures_saver = SignaturesSaver::new(register, receiver).await?;
        tokio::spawn(async move { signatures_saver.run().await });

        Ok(Self { sender })
    }

    pub async fn store_signatures_and_state(
        &self,
        signatures: Vec<RpcConfirmedTransactionStatusWithSignature>,
        program_address: Pubkey,
        saved_state: SavedState,
    ) -> usize {
        let (sender, receiver) = oneshot::channel();
        let msg = SignaturesSaverMessage::SaveSignaturesAndState {
            signatures,
            program_address,
            saved_state: Box::new(saved_state),
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;
        receiver
            .await
            .expect("SignaturesSaver task has been killed")
    }
}
