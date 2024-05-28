use log::{error, warn};
use solana_transaction_status::Reward;
use std::time::Duration;
use tokio::{
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::{
    errors::RewardsCollectorError,
    storage::{
        epoch_storage::Epoch,
        main_storage::{connect_main_storage, MainStorage},
    },
};

const BUFFER_SIZE: usize = 10000;
const FLUSH_BUFFER_TIMEOUT: u64 = 5000;

struct RewardsCollector {
    rewards: Vec<(String, Epoch, Option<u64>, Reward, i64)>,
    main_storage: Box<dyn MainStorage>,
    receiver: mpsc::Receiver<RewardsCollectorMessage>,
    tick_receiver: mpsc::Receiver<()>,
    ticks: u8,
}

enum RewardsCollectorMessage {
    SaveReward {
        vote_account: String,
        epoch: Epoch,
        first_block_slot: Option<u64>,
        reward: Reward,
        block_time: i64,
        respond_to: oneshot::Sender<()>,
    },
}

impl RewardsCollector {
    async fn new(
        receiver: mpsc::Receiver<RewardsCollectorMessage>,
        tick_receiver: mpsc::Receiver<()>,
    ) -> Result<Self, RewardsCollectorError> {
        let rewards = Vec::with_capacity(BUFFER_SIZE);
        let main_storage = connect_main_storage().await?;

        Ok(RewardsCollector {
            rewards,
            main_storage,
            receiver,
            tick_receiver,
            ticks: 0,
        })
    }

    async fn handle_message(&mut self, msg: RewardsCollectorMessage) {
        match msg {
            RewardsCollectorMessage::SaveReward {
                vote_account,
                epoch,
                first_block_slot,
                reward,
                block_time,
                respond_to,
            } => {
                self.collect_reward((vote_account, epoch, first_block_slot, reward, block_time))
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
            warn!("Flushed rewards buffer because timeout expired");
        }
    }

    async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    self.handle_message(msg).await;
                },
                Some(_) = self.tick_receiver.recv() => {
                    self.handle_tick_message().await;
                },
                else => break,
            }
        }
    }

    async fn collect_reward(&mut self, reward_record: (String, Epoch, Option<u64>, Reward, i64)) {
        self.rewards.push(reward_record);
        self.ticks = 0;

        if self.rewards.len() >= BUFFER_SIZE {
            self.flush_buffer().await;
            warn!("1. Flushed rewards buffer because a threshold is reached");
        }
    }

    async fn flush_buffer(&mut self) {
        if !self.rewards.is_empty() {
            let result = self
                .main_storage
                .store_rewards_block(self.rewards.as_slice().to_vec())
                .await;

            match result {
                Ok(..) => {
                    warn!("2. Stored {} rewards", self.rewards.len());
                    self.rewards.clear();
                }
                Err(err) => error!("Rewards were not stored: {:#?}", err),
            }
        }
    }
}

pub struct RewardsCollectorHandle {
    sender: mpsc::Sender<RewardsCollectorMessage>,
}

impl RewardsCollectorHandle {
    pub async fn new() -> Result<Self, RewardsCollectorError> {
        let (sender, receiver) = mpsc::channel(100);
        let (tick_sender, tick_receiver) = mpsc::channel(1);
        let mut rewards_collector = RewardsCollector::new(receiver, tick_receiver).await?;

        tokio::spawn(async move { rewards_collector.run().await });

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(FLUSH_BUFFER_TIMEOUT)).await;
                tick_sender.send(()).await.unwrap();
            }
        });

        Ok(Self { sender })
    }

    pub async fn save_reward(
        &mut self,
        vote_account: String,
        epoch: Epoch,
        first_block_slot: Option<u64>,
        reward: Reward,
        block_time: i64,
    ) {
        let (sender, receiver) = oneshot::channel();
        let msg = RewardsCollectorMessage::SaveReward {
            vote_account,
            epoch,
            first_block_slot,
            reward,
            block_time,
            respond_to: sender,
        };

        let _ = self.sender.send(msg).await;

        receiver
            .await
            .expect("RewardsCollector task has been killed")
    }
}
