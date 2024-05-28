use log::info;
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    errors::VoteAccountResolverError, repeat_until_ok, storage::main_storage::connect_main_storage,
};

#[allow(dead_code)]
pub(crate) struct VoteAccountResolver {}

impl VoteAccountResolver {
    #[allow(dead_code)]
    pub async fn run() -> Result<(), VoteAccountResolverError> {
        info!("Starting vote_account_resolver");
        let mut main_storage = connect_main_storage().await?;

        tokio::spawn(async move {
            loop {
                let rewards = main_storage.get_rewards_with_empty_vote_acc().await;
                if let Ok(rewards) = rewards {
                    for reward in rewards {
                        let vote_account = repeat_until_ok!(
                            main_storage
                                .lookup_vote_acc(
                                    reward.first_block_slot.unwrap(),
                                    reward.pubkey.as_str(),
                                )
                                .await,
                            5
                        )
                        .unwrap_or_default();

                        repeat_until_ok!(
                            main_storage
                                .update_reward(&vote_account, reward.epoch, &reward.pubkey)
                                .await,
                            5
                        );
                    }
                }

                sleep(Duration::from_secs(10)).await;
            }
        });

        Ok(())
    }
}
