use crate::{repeat_until_ok, solana_client::*};
use log::info;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use tokio::sync::{mpsc, oneshot};

struct TransactionsRpcLoader {
    receiver: mpsc::Receiver<TransactionsRpcLoaderMessage>,
    rpc_client: Box<dyn SolanaClient>,
}

enum TransactionsRpcLoaderMessage {
    LoadTransaction {
        signature: String,
        respond_to: oneshot::Sender<EncodedConfirmedTransactionWithStatusMeta>,
    },
}

impl TransactionsRpcLoader {
    async fn new(
        client_type: &ClientType,
        receiver: mpsc::Receiver<TransactionsRpcLoaderMessage>,
        url: &str,
    ) -> Self {
        TransactionsRpcLoader {
            receiver,
            rpc_client: crate::solana_client::new_with_url(client_type, url).await,
        }
    }

    async fn handle_message(&mut self, msg: TransactionsRpcLoaderMessage) {
        match msg {
            TransactionsRpcLoaderMessage::LoadTransaction {
                signature,
                respond_to,
            } => {
                let _ = respond_to.send(self.process_load_transaction(&signature).await);
            }
        }
    }

    async fn run(&mut self) {
        info!("Transaction rpc loader started");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
        info!("Transaction rpc loader stopped");
    }

    async fn process_load_transaction(
        &self,
        signature: &str,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        repeat_until_ok!(self.rpc_client.load_transaction_info(signature).await, 5)
    }
}

#[derive(Clone)]
pub struct TransactionsRpcLoaderHandle {
    sender: mpsc::Sender<TransactionsRpcLoaderMessage>,
}

impl TransactionsRpcLoaderHandle {
    pub async fn new(client_type: &ClientType, url: &str) -> Self {
        let (sender, receiver) = mpsc::channel(3);
        let mut transactions_rpc_loader =
            TransactionsRpcLoader::new(client_type, receiver, url).await;
        tokio::spawn(async move { transactions_rpc_loader.run().await });

        Self { sender }
    }

    pub async fn transaction_rpc_load(
        &self,
        signature: String,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        let (sender, receiver) = oneshot::channel();
        let msg = TransactionsRpcLoaderMessage::LoadTransaction {
            signature,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;
        receiver
            .await
            .expect("SignaturesRpcLoader task has been killed")
    }
}
