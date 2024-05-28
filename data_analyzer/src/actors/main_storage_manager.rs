use crate::{metrics_update, register::Register, storages::main_storage::*};
use anyhow::Result;
use macros::{ActorInstance, HandleInstance};
use tokio::sync::{mpsc, oneshot};

#[derive(ActorInstance)]
struct MainStorageManager {
    receiver: mpsc::Receiver<MainStorageManagerMessage>,
    storage: Box<dyn MainStorage>,
}

#[allow(clippy::enum_variant_names)]
enum MainStorageManagerMessage {
    StoreInstructionsBlock {
        instructions: Vec<Instruction>,
        respond_to: oneshot::Sender<Result<()>>,
    },
    StoreInstructionArgumentsBlock {
        instruction_arguments: Vec<InstructionArgument>,
        respond_to: oneshot::Sender<Result<()>>,
    },
    StoreBalancesBlock {
        balances: Vec<Balance>,
        respond_to: oneshot::Sender<Result<()>>,
    },
    StoreErroneousTransactionBlock {
        erroneous_transactions: Vec<ErroneousTransaction>,
        respond_to: oneshot::Sender<Result<()>>,
    },
    StoreDelegationsBlock {
        delegations: Vec<Delegation>,
        respond_to: oneshot::Sender<Result<()>>,
    },
    StoreUndelegationsBlock {
        undelegations: Vec<Delegation>,
        respond_to: oneshot::Sender<Result<()>>,
    },
}

impl MainStorageManager {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<MainStorageManagerMessage>,
    ) -> Result<Self> {
        metrics_update!(inc total ACTIVE_ACTOR_INSTANCES_COUNT, &["main_storage_manager"]);

        let storage =
            connect_main_storage(&register.config.get_main_storage_config().database_url).await?;

        Ok(MainStorageManager { receiver, storage })
    }

    async fn handle_message(&mut self, msg: MainStorageManagerMessage) {
        match msg {
            MainStorageManagerMessage::StoreInstructionsBlock {
                respond_to,
                instructions,
            } => {
                let result = self.storage.store_instructions_block(instructions).await;
                let _ = respond_to.send(result);
            }
            MainStorageManagerMessage::StoreInstructionArgumentsBlock {
                respond_to,
                instruction_arguments,
            } => {
                let result = self
                    .storage
                    .store_instruction_arguments_block(instruction_arguments)
                    .await;
                let _ = respond_to.send(result);
            }
            MainStorageManagerMessage::StoreBalancesBlock {
                respond_to,
                balances,
            } => {
                let result = self.storage.store_balances_block(balances).await;
                let _ = respond_to.send(result);
            }
            MainStorageManagerMessage::StoreErroneousTransactionBlock {
                respond_to,
                erroneous_transactions,
            } => {
                let result = self
                    .storage
                    .store_erroneous_transaction_block(erroneous_transactions)
                    .await;
                let _ = respond_to.send(result);
            }
            MainStorageManagerMessage::StoreDelegationsBlock {
                respond_to,
                delegations,
            } => {
                let result = self.storage.store_delegations_block(delegations).await;
                let _ = respond_to.send(result);
            }
            MainStorageManagerMessage::StoreUndelegationsBlock {
                respond_to,
                undelegations,
            } => {
                let result = self.storage.store_undelegations_block(undelegations).await;
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
pub struct MainStorageManagerHandle {
    sender: mpsc::Sender<MainStorageManagerMessage>,
}

impl MainStorageManagerHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(100);
        let mut main_storage_manager = MainStorageManager::new(register, receiver).await?;
        tokio::spawn(async move { main_storage_manager.run().await });

        metrics_update!(inc total ACTIVE_HANDLE_INSTANCES_COUNT, &["main_storage_manager_handle"]);

        Ok(Self { sender })
    }

    pub async fn store_instructions_block(&mut self, instructions: &[Instruction]) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        let msg = MainStorageManagerMessage::StoreInstructionsBlock {
            instructions: instructions.to_vec(),
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("MainStorageManager task has been killed")
    }

    pub async fn store_instruction_arguments_block(
        &mut self,
        instruction_arguments: &[InstructionArgument],
    ) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        let msg = MainStorageManagerMessage::StoreInstructionArgumentsBlock {
            instruction_arguments: instruction_arguments.to_vec(),
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("MainStorageManager task has been killed")
    }

    pub async fn store_balances_block(&mut self, balances: &[Balance]) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        let msg = MainStorageManagerMessage::StoreBalancesBlock {
            balances: balances.to_vec(),
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("MainStorageManager task has been killed")
    }

    pub async fn store_delegations_block(&mut self, delegations: Vec<Delegation>) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        let msg = MainStorageManagerMessage::StoreDelegationsBlock {
            delegations,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("MainStorageManager task has been killed")
    }

    pub async fn store_undelegations_block(
        &mut self,
        undelegations: Vec<Delegation>,
    ) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        let msg = MainStorageManagerMessage::StoreUndelegationsBlock {
            undelegations,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("MainStorageManager task has been killed")
    }

    pub async fn store_erroneous_transactions_block(
        &mut self,
        erroneous_transactions: &[ErroneousTransaction],
    ) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        let msg = MainStorageManagerMessage::StoreErroneousTransactionBlock {
            erroneous_transactions: erroneous_transactions.to_vec(),
            respond_to: sender,
        };
        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("MainStorageManager task has been killed")
    }
}
