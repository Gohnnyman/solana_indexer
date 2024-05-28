use anyhow::Result;
use log::info;

use crate::{
    actors::{
        queue_manager::QueueManagerHandle, transactions_rpc_loader::TransactionsRpcLoaderHandle,
        transactions_saver::TransactionsSaverHandle,
    },
    register::Register,
};

pub struct TransactionsLoadingCtx;

impl TransactionsLoadingCtx {
    pub async fn setup_and_run(register: &Register) -> Result<Self> {
        let primary_queue_manager = QueueManagerHandle::new(register).await?;

        for tx_loader_idx in 0..register.config.get_tx_loaders_num() {
            let queue_manager = primary_queue_manager.clone();
            let rpc_loader = TransactionsRpcLoaderHandle::new(
                register.config.get_solana_client_type(),
                &register.config.get_endpoint_url(),
            )
            .await;
            let transaction_saver = TransactionsSaverHandle::new(register).await?;
            let load_only_successful_transactions = register
                .config
                .get_load_only_successful_transactions_status();

            tokio::spawn(async move {
                loop {
                    if let Some(signature) = queue_manager
                        .get_signature_from_queue(load_only_successful_transactions)
                        .await
                    {
                        info!("TxLoader {} scheduled {:?}", &tx_loader_idx, &signature);

                        let sign = signature.clone();

                        let transaction = rpc_loader.transaction_rpc_load(signature.clone()).await;

                        info!("TxLoader {} success - {:?}", &tx_loader_idx, &sign);
                        transaction_saver.save_transaction(sign, transaction).await;
                        queue_manager
                            .mark_signature_as_loaded(signature.clone())
                            .await;
                    }
                }
            });
            info!("Transaction loader {} spawned", tx_loader_idx);
        }

        Ok(Self {})
    }
}
