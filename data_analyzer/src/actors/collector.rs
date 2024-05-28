use super::main_storage_manager::MainStorageManagerHandle;
use super::transaction_parser::{Delegations, Undelegations};
use crate::metrics_update;
use crate::storages::main_storage::{Balance, Delegation, InstructionArgument};
use crate::{register::Register, storages::main_storage::Instruction};
use anyhow::Result;
use log::{error, info};
use macros::{ActorInstance, HandleInstance};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;

const BUFFER_SIZE: usize = 100_000;
const FLUSH_BUFFER_TIMEOUT: u64 = 3000;

#[derive(ActorInstance)]
struct Collector {
    instructions: Vec<Instruction>,
    balances: Vec<Balance>,
    instruction_arguments: Vec<InstructionArgument>,
    delegations: Vec<Delegation>,
    undelegations: Vec<Delegation>,
    main_storage_manager: MainStorageManagerHandle,
    receiver: mpsc::Receiver<CollectorMessage>,
    tick_receiver: mpsc::Receiver<()>,
    ticks: u8,
}

enum CollectorMessage {
    SaveInstruction {
        instruction: Instruction,
        respond_to: oneshot::Sender<()>,
    },
    SaveBalance {
        balance: Balance,
        respond_to: oneshot::Sender<()>,
    },
    SaveInstructionArgument {
        instruction_argument: InstructionArgument,
        respond_to: oneshot::Sender<()>,
    },
    SaveDelegation {
        delegation: Delegation,
        respond_to: oneshot::Sender<()>,
    },
    SaveUndelegation {
        undelegation: Delegation,
        respond_to: oneshot::Sender<()>,
    },
}

impl Collector {
    async fn new(
        register: &Register,
        receiver: mpsc::Receiver<CollectorMessage>,
        tick_receiver: mpsc::Receiver<()>,
    ) -> Result<Self> {
        let instructions = Vec::with_capacity(BUFFER_SIZE);
        let balances = Vec::with_capacity(BUFFER_SIZE);
        let instruction_arguments = Vec::with_capacity(BUFFER_SIZE);
        let delegations = Delegations::with_capacity(BUFFER_SIZE);
        let undelegations = Undelegations::with_capacity(BUFFER_SIZE);

        let main_storage_manager = MainStorageManagerHandle::new(register).await?;

        metrics_update!(inc total ACTIVE_ACTOR_INSTANCES_COUNT, &["instructions_collector"]);

        Ok(Collector {
            instructions,
            balances,
            instruction_arguments,
            delegations,
            undelegations,
            main_storage_manager,
            receiver,
            tick_receiver,
            ticks: 0,
        })
    }

    async fn handle_message(&mut self, msg: CollectorMessage) {
        match msg {
            CollectorMessage::SaveInstruction {
                instruction,
                respond_to,
            } => {
                self.collect_instruction(instruction).await;
                let _ = respond_to.send(());
            }
            CollectorMessage::SaveBalance {
                balance,
                respond_to,
            } => {
                self.collect_balance(balance).await;
                let _ = respond_to.send(());
            }
            CollectorMessage::SaveInstructionArgument {
                instruction_argument,
                respond_to,
            } => {
                self.collect_instruction_argument(instruction_argument)
                    .await;
                let _ = respond_to.send(());
            }
            CollectorMessage::SaveDelegation {
                delegation,
                respond_to,
            } => {
                self.collect_delegation(delegation).await;
                let _ = respond_to.send(());
            }
            CollectorMessage::SaveUndelegation {
                undelegation,
                respond_to,
            } => {
                self.collect_undelegation(undelegation).await;
                let _ = respond_to.send(());
            }
        }
    }

    async fn handle_tick_message(&mut self) {
        self.ticks += 1;

        if self.ticks >= 2 {
            self.flush_buffer().await;
            self.ticks = 0;
            info!("Flushed collector's buffer because timeout expired");
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

    async fn collect_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
        self.ticks = 0;

        if self.instructions.len() >= BUFFER_SIZE {
            self.flush_instructions().await;
            info!("1. Flushed instructions buffer because a threshold is reached");
        }
    }

    async fn collect_balance(&mut self, balance: Balance) {
        self.balances.push(balance);
        self.ticks = 0;

        if self.balances.len() >= BUFFER_SIZE {
            self.flush_balances().await;
            info!("1. Flushed balances buffer because a threshold is reached");
        }
    }

    async fn collect_instruction_argument(&mut self, instruction_argument: InstructionArgument) {
        self.instruction_arguments.push(instruction_argument);
        self.ticks = 0;

        if self.instruction_arguments.len() >= BUFFER_SIZE {
            self.flush_instruction_arguments().await;
            info!("1. Flushed instruction arguments buffer because a threshold is reached");
        }
    }

    async fn collect_delegation(&mut self, delegation: Delegation) {
        self.delegations.push(delegation);
        self.ticks = 0;

        if self.delegations.len() >= BUFFER_SIZE {
            self.flush_delegations().await;
            info!("1. Flushed delegations buffer because a threshold is reached");
        }
    }

    async fn collect_undelegation(&mut self, undelegation: Delegation) {
        self.undelegations.push(undelegation);
        self.ticks = 0;

        if self.undelegations.len() >= BUFFER_SIZE {
            self.flush_undelegations().await;
            info!("1. Flushed undelegations buffer because a threshold is reached");
        }
    }

    async fn flush_buffer(&mut self) {
        self.flush_instructions().await;
        self.flush_balances().await;
        self.flush_instruction_arguments().await;
        self.flush_delegations().await;
        self.flush_undelegations().await;
    }

    async fn flush_instructions(&mut self) {
        if !self.instructions.is_empty() {
            let result = self
                .main_storage_manager
                .store_instructions_block(self.instructions.as_slice())
                .await;

            match result {
                Ok(..) => {
                    info!("2. Stored {} instructions", self.instructions.len());
                    self.instructions.clear();
                }
                Err(err) => error!("Instructions were not stored: {:#?}", err),
            }
        }
    }

    async fn flush_balances(&mut self) {
        if !self.balances.is_empty() {
            let result = self
                .main_storage_manager
                .store_balances_block(self.balances.as_slice())
                .await;
            match result {
                Ok(..) => {
                    info!("2. Stored {} balances", self.balances.len());
                    self.balances.clear();
                }
                Err(err) => error!("Balances were not stored: {:#?}", err),
            }
        }
    }

    async fn flush_instruction_arguments(&mut self) {
        if !self.instruction_arguments.is_empty() {
            let result = self
                .main_storage_manager
                .store_instruction_arguments_block(self.instruction_arguments.as_slice())
                .await;

            match result {
                Ok(..) => {
                    info!(
                        "2. Stored {} instruction arguments",
                        self.instruction_arguments.len()
                    );
                    self.instruction_arguments.clear();
                }
                Err(err) => error!("Instruction arguments were not stored: {:#?}", err),
            }
        }
    }

    async fn flush_delegations(&mut self) {
        if !self.delegations.is_empty() {
            let result = self
                .main_storage_manager
                .store_delegations_block(self.delegations.clone())
                .await;

            match result {
                Ok(..) => {
                    info!("2. Stored {} delegations", self.delegations.len());
                    self.delegations.clear();
                }
                Err(err) => error!("Delegations were not stored: {:#?}", err),
            }
        }
    }

    async fn flush_undelegations(&mut self) {
        if !self.undelegations.is_empty() {
            let result = self
                .main_storage_manager
                .store_undelegations_block(self.undelegations.clone())
                .await;

            match result {
                Ok(..) => {
                    info!("2. Stored {} undelegations", self.undelegations.len());
                    self.undelegations.clear();
                }
                Err(err) => error!("Unelegations were not stored: {:#?}", err),
            }
        }
    }
}

#[derive(HandleInstance)]
pub struct CollectorHandle {
    sender: mpsc::Sender<CollectorMessage>,
}

impl CollectorHandle {
    pub async fn new(register: &Register) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(100);
        let (tick_sender, tick_receiver) = mpsc::channel(1);
        let mut instructions_collector = Collector::new(register, receiver, tick_receiver).await?;

        tokio::spawn(async move { instructions_collector.run().await });

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(FLUSH_BUFFER_TIMEOUT)).await;
                tick_sender.send(()).await.unwrap();
            }
        });

        metrics_update!(inc total ACTIVE_HANDLE_INSTANCES_COUNT, &["instructions_collector_handle"]);

        Ok(Self { sender })
    }

    pub async fn save_instruction(&mut self, instruction: Instruction) {
        let (sender, receiver) = oneshot::channel();
        let msg = CollectorMessage::SaveInstruction {
            instruction,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver.await.expect("Collector task has been killed")
    }

    pub async fn save_balance(&mut self, balance: Balance) {
        let (sender, receiver) = oneshot::channel();
        let msg = CollectorMessage::SaveBalance {
            balance,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver.await.expect("Collector task has been killed")
    }

    pub async fn save_instruction_argument(&mut self, instruction_argument: InstructionArgument) {
        let (sender, receiver) = oneshot::channel();
        let msg = CollectorMessage::SaveInstructionArgument {
            instruction_argument,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver.await.expect("Collector task has been killed")
    }

    pub async fn save_delegation(&mut self, delegation: Delegation) {
        let (sender, receiver) = oneshot::channel();
        let msg = CollectorMessage::SaveDelegation {
            delegation,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver.await.expect("Collector task has been killed")
    }

    pub async fn save_undelegation(&mut self, undelegation: Delegation) {
        let (sender, receiver) = oneshot::channel();
        let msg = CollectorMessage::SaveUndelegation {
            undelegation,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver.await.expect("Collector task has been killed")
    }
}
