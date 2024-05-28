use super::main_storage_manager::MainStorageManagerHandle;
use crate::errors::ParseInstructionError;
use crate::metrics_update;
use crate::{register::Register, storages::main_storage::ErroneousTransaction};
use anyhow::Result;
use log::{error, info};
use macros::{ActorInstance, HandleInstance};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;

const ERRONEOUS_TRANSACTIONS_BUFFER_SIZE: usize = 100;
const FLUSH_BUFFER_TIMEOUT: u64 = 3000;

#[derive(ActorInstance)]
struct ErroneousTransactionsCollector {
    erroneous_transactions: Vec<ErroneousTransaction>,
    main_storage_manager: MainStorageManagerHandle,
    receiver: mpsc::Receiver<ErroneousTransactionsCollectorMessage>,
    tick_receiver: mpsc::Receiver<()>,
    ticks: u8,
}

enum ErroneousTransactionsCollectorMessage {
    SaveErroneousTransaction {
        erroneous_transaction: ErroneousTransaction,
        respond_to: oneshot::Sender<()>,
    },
}

impl ErroneousTransactionsCollector {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<ErroneousTransactionsCollectorMessage>,
        tick_receiver: mpsc::Receiver<()>,
    ) -> Result<Self> {
        let erroneous_transactions = Vec::with_capacity(ERRONEOUS_TRANSACTIONS_BUFFER_SIZE);
        let main_storage_manager = MainStorageManagerHandle::new(register).await?;

        metrics_update!(inc total ACTIVE_ACTOR_INSTANCES_COUNT, &["erroneous_transactions_collector"]);

        Ok(ErroneousTransactionsCollector {
            erroneous_transactions,
            main_storage_manager,
            receiver,
            tick_receiver,
            ticks: 0,
        })
    }

    async fn handle_message(&mut self, msg: ErroneousTransactionsCollectorMessage) {
        match msg {
            ErroneousTransactionsCollectorMessage::SaveErroneousTransaction {
                erroneous_transaction,
                respond_to,
            } => {
                self.collect_erroneous_transaction(erroneous_transaction)
                    .await;
                let _ = respond_to.send(());
            }
        }
    }

    async fn handle_tick_message(&mut self) {
        self.ticks += 1;

        if self.ticks >= 2 {
            self.flush_buffer().await;
            self.ticks = 0;
            info!("Flushed erroneous_transactions buffer because timeout expired");
        }
    }

    async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    self.handle_message(msg).await;
                },
                Some(_msg) = self.tick_receiver.recv() => {
                    self.handle_tick_message().await;
                },
                else => break,
            }
        }
    }

    async fn collect_erroneous_transaction(&mut self, erroneous_transaction: ErroneousTransaction) {
        self.erroneous_transactions.push(erroneous_transaction);
        self.ticks = 0;

        if self.erroneous_transactions.len() >= ERRONEOUS_TRANSACTIONS_BUFFER_SIZE {
            self.flush_buffer().await;
            info!("1. Flushed erroneous_transactions buffer because a threshold is reached");
        }
    }

    async fn flush_buffer(&mut self) {
        if !self.erroneous_transactions.is_empty() {
            let result = self
                .main_storage_manager
                .store_erroneous_transactions_block(self.erroneous_transactions.as_slice())
                .await;

            match result {
                Ok(..) => {
                    info!(
                        "2. Stored {} erroneous transactions",
                        self.erroneous_transactions.len()
                    );
                    self.erroneous_transactions.clear();
                }
                Err(err) => error!("Erroneous transactions were not stored: {:#?}", err),
            }
        }
    }
}

#[derive(HandleInstance)]
pub struct ErroneousTransactionsCollectorHandle {
    sender: mpsc::Sender<ErroneousTransactionsCollectorMessage>,
}

impl ErroneousTransactionsCollectorHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(100);
        let (tick_sender, tick_receiver) = mpsc::channel(1);
        let mut erroneous_transactions_collector =
            ErroneousTransactionsCollector::new(register, receiver, tick_receiver).await?;

        tokio::spawn(async move { erroneous_transactions_collector.run().await });

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(FLUSH_BUFFER_TIMEOUT)).await;
                tick_sender.send(()).await.unwrap();
            }
        });

        metrics_update!(inc total ACTIVE_HANDLE_INSTANCES_COUNT, &["erroneous_transactions_collector_handle"]);

        Ok(Self { sender })
    }

    pub async fn save_erroneous_transaction(
        &mut self,
        erroneous_transaction: ErroneousTransaction,
    ) {
        let (sender, receiver) = oneshot::channel();
        let msg = ErroneousTransactionsCollectorMessage::SaveErroneousTransaction {
            erroneous_transaction,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("ErroneousTransactionsCollector task has been killed")
    }

    pub async fn handle_error(
        &mut self,
        encoded_transaction: EncodedConfirmedTransactionWithStatusMeta,
        err: ParseInstructionError,
    ) -> Result<()> {
        let err_tx =
            ErroneousTransaction::try_from_transactions_with_error(encoded_transaction, err)?;

        log::error!(
            "Erroneous transaction found: {:#?}, tx_hash: {}",
            err_tx.cause,
            err_tx.tx_signature
        );
        self.save_erroneous_transaction(err_tx).await;

        Ok(())
    }
}
