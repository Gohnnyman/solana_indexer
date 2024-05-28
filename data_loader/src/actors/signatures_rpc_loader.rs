use std::str::FromStr;

use crate::solana_client::*;
use log::{error, info};
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::{mpsc, oneshot};

use super::saved_state_manager::SavedState;

struct SignaturesRpcLoader {
    receiver: mpsc::Receiver<SignaturesRpcLoaderMessage>,
    rpc_client: Box<dyn SolanaClient>,
    account_key: String,
}

enum SignaturesRpcLoaderMessage {
    LoadSignatures {
        respond_to: oneshot::Sender<Vec<RpcConfirmedTransactionStatusWithSignature>>,
        saved_state: SavedState,
    },
}

impl SignaturesRpcLoader {
    async fn new(
        client_type: &ClientType,
        receiver: mpsc::Receiver<SignaturesRpcLoaderMessage>,
        url: &str,
        account_key: &str,
    ) -> Self {
        SignaturesRpcLoader {
            receiver,
            rpc_client: crate::solana_client::new_with_url(client_type, url).await,
            account_key: account_key.to_string(),
        }
    }

    async fn handle_message(&mut self, msg: SignaturesRpcLoaderMessage) {
        match msg {
            SignaturesRpcLoaderMessage::LoadSignatures {
                respond_to,
                saved_state,
            } => {
                let _ = respond_to.send(self.process_load_signatures(saved_state).await);
            }
        }
    }

    async fn run(&mut self) {
        info!("Signatures rpc loader started");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
        info!("Signatures rpc loader stopped");
    }

    async fn process_load_signatures(
        &self,
        saved_state: SavedState,
    ) -> Vec<RpcConfirmedTransactionStatusWithSignature> {
        info!("Signatures loading - request sent");

        let signatures = self
            .rpc_client
            .load_signatures_batch(
                &Pubkey::from_str(&self.account_key).unwrap(),
                saved_state.before,
                None,
            )
            .await;

        info!("Signatures loading - response received");

        match signatures {
            Ok(res_vector) => res_vector,
            Err(e) => {
                error!("Error during signatures request: {:?}", e);
                [].to_vec()
            }
        }
    }
}

#[derive(Clone)]
pub struct SignaturesRpcLoaderHandle {
    sender: mpsc::Sender<SignaturesRpcLoaderMessage>,
}

impl SignaturesRpcLoaderHandle {
    pub async fn new(client_type: &ClientType, url: &str, account_key: &str) -> Self {
        let (sender, receiver) = mpsc::channel(16);
        let mut signatures_rpc_loader =
            SignaturesRpcLoader::new(client_type, receiver, url, account_key).await;
        tokio::spawn(async move { signatures_rpc_loader.run().await });

        Self { sender }
    }

    pub async fn signatures_rpc_load(
        &self,
        saved_state: SavedState,
    ) -> Vec<RpcConfirmedTransactionStatusWithSignature> {
        let (sender, receiver) = oneshot::channel();
        let msg = SignaturesRpcLoaderMessage::LoadSignatures {
            respond_to: sender,
            saved_state,
        };

        let _ = self.sender.send(msg).await;
        receiver
            .await
            .expect("SignaturesRpcLoader task has been killed")
    }
}
