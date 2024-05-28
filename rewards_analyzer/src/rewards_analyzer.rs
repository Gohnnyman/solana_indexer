use std::time::Duration;

use log::{error, info};
use solana_transaction_status::RewardType;
use tokio::time::sleep;

use crate::{
    errors::RewardsAnalyzerError,
    repeat_until_ok,
    rewards_collector::RewardsCollectorHandle,
    storage::{epoch_storage::EpochStorage, main_storage::connect_main_storage},
};

pub struct RewardsAnalyzer {}

impl RewardsAnalyzer {
    pub async fn run() -> Result<Self, RewardsAnalyzerError> {
        info!("Starting rewards_analyzer");

        let mut main_storage = connect_main_storage().await?;
        let mut rewards_collector = RewardsCollectorHandle::new().await?;

        tokio::spawn(async move {
            loop {
                if let (Some(epoch), first_block_slot) =
                    repeat_until_ok!(EpochStorage::get_not_parsed_first_block_epoch().await, 5)
                {
                    info!("Start analyze the rewards of {} epoch", epoch);
                    let (block_time, rewards) =
                        repeat_until_ok!(EpochStorage::get_rewards_records(epoch).await, 5);
                    info!("The number of rewards is: {}", rewards.len());

                    info!("Call prepare_clean_unfinished");
                    repeat_until_ok!(main_storage.clean_unfinished(epoch).await, 5);

                    for reward in rewards {
                        match reward.reward_type {
                            Some(RewardType::Staking) => {
                                let vote_acc = repeat_until_ok!(
                                    main_storage
                                        .lookup_vote_acc(first_block_slot.unwrap(), &reward.pubkey)
                                        .await,
                                    5
                                );

                                rewards_collector
                                    .save_reward(
                                        vote_acc.unwrap_or_default(),
                                        epoch,
                                        first_block_slot,
                                        reward,
                                        block_time,
                                    )
                                    .await;
                            }
                            Some(RewardType::Voting) => {
                                rewards_collector
                                    .save_reward(
                                        String::from(""),
                                        epoch,
                                        first_block_slot,
                                        reward,
                                        block_time,
                                    )
                                    .await;
                            }
                            _ => {}
                        }
                    }

                    info!("Complete analyze the rewards of {} epoch", epoch);

                    repeat_until_ok!(EpochStorage::mark_rewards_parsed(epoch).await, 5);
                }

                sleep(Duration::from_secs(60)).await;
            }
        });

        Ok(Self {})
    }
}
