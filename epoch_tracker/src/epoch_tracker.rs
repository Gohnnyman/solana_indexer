use std::{
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use clap::error;
use futures::executor;
use log::{debug, error, info};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcBlockConfig, RpcProgramAccountsConfig},
    rpc_filter,
};
use solana_sdk::{
    account::Account,
    clock::{Epoch, Slot},
    commitment_config::CommitmentConfig,
    epoch_info::EpochInfo,
    pubkey::Pubkey,
    stake,
};
use solana_transaction_status::EncodedConfirmedBlock;
use tokio::time::sleep;

use crate::{
    configuration::get_matches, errors::EpochTrackerError, register::Register,
    storage::epoch_storage::EpochStorage,
};

struct CurrentEpoch {
    pub epoch: Epoch,
}

pub struct EpochTracker {}

impl EpochTracker {
    pub async fn run() -> Result<Self, EpochTrackerError> {
        info!("Starting epoch_tracker");

        let url = Register::current().configuration.endpoint().to_string();
        let rpc_client = RpcClient::new(url.clone());

        let current_epoch: Arc<Mutex<CurrentEpoch>> = Arc::new(Mutex::new(CurrentEpoch {
            epoch: Default::default(),
        }));

        loop {
            if let Ok(epoch_info) = executor::block_on(rpc_client.get_epoch_info()) {
                debug!("Current Epoch is {}", epoch_info.epoch);

                // update current_epoch
                let mut lock: MutexGuard<CurrentEpoch> = current_epoch.lock().unwrap();
                lock.epoch = epoch_info.epoch;
                break;
            }

            sleep(Duration::from_secs(1)).await;
        }

        let c_current_epoch = current_epoch.clone();

        let mut epochs_setup_completed = false;
        let setup_epochs = get_matches().is_present("setup-epochs");

        // Track an epoch
        tokio::spawn(async move {
            loop {
                if let Ok(epoch_info) = rpc_client.get_epoch_info().await {
                    {
                        // update current_epoch
                        let mut lock: MutexGuard<CurrentEpoch> = c_current_epoch.lock().unwrap();
                        lock.epoch = epoch_info.epoch;
                    }

                    if setup_epochs && !epochs_setup_completed {
                        Self::setup_epochs(&mut epoch_info.clone()).await.unwrap();
                        epochs_setup_completed = true;
                    }

                    tokio::spawn(
                        async move { EpochStorage::store_epoch(&epoch_info).await.unwrap() },
                    );
                }

                sleep(Duration::from_secs(5)).await;
            }
        });

        let rpc_client = RpcClient::new(url.clone());

        // Track first blocks
        tokio::spawn(async move {
            loop {
                let epoch_list = EpochStorage::get_epoch_with_empty_first_block()
                    .await
                    .unwrap();

                for epoch in epoch_list.iter() {
                    if let Ok((first_block, first_block_raw)) =
                        Self::get_first_block(&rpc_client, *epoch).await
                    {
                        let epoch = *epoch;
                        if let Some(first_block) = first_block {
                            tokio::spawn(async move {
                                EpochStorage::update_first_block_for_epoch(
                                    epoch,
                                    first_block,
                                    &first_block_raw,
                                )
                                .await
                                .unwrap();
                            });
                        }
                    }
                }

                sleep(Duration::from_secs(10)).await;
            }
        });

        let rpc_client = RpcClient::new(url.clone());
        let c_current_epoch = current_epoch.clone();

        // Track last blocks
        tokio::spawn(async move {
            loop {
                let epoch_list = EpochStorage::get_epoch_with_empty_last_block()
                    .await
                    .unwrap();

                let current_epoch = {
                    let lock: MutexGuard<CurrentEpoch> = c_current_epoch.lock().unwrap();
                    lock.epoch
                };

                for epoch in epoch_list.iter() {
                    if *epoch != current_epoch {
                        if let Ok((last_block, last_block_raw)) =
                            Self::get_last_block(&rpc_client, *epoch).await
                        {
                            let epoch = *epoch;
                            if let Some(last_block) = last_block {
                                tokio::spawn(async move {
                                    EpochStorage::update_last_block_for_epoch(
                                        epoch,
                                        last_block,
                                        &last_block_raw,
                                    )
                                    .await
                                    .unwrap();
                                });
                            }
                        }
                    }
                }
                sleep(Duration::from_secs(10)).await;
            }
        });

        // let rpc_client = RpcClient::new(url);
        // let c_current_epoch = current_epoch.clone();

        // // Track stakes
        // tokio::spawn(async move {
        //     let validator = Register::current()
        //         .configuration
        //         .validator_vote_account()
        //         .to_string();

        //     loop {
        //         let current_epoch = {
        //             let lock: MutexGuard<CurrentEpoch> = c_current_epoch.lock().unwrap();
        //             lock.epoch
        //         };

        //         if let Ok(stakes_loaded) = EpochStorage::stakes_loaded(current_epoch).await {
        //             if !stakes_loaded {
        //                 info!("Current epoch: {}", current_epoch);
        //                 let all_stake_accounts = Self::get_stake_accounts(&rpc_client, &validator)
        //                     .await
        //                     .unwrap();

        //                 EpochStorage::update_all_stakes(current_epoch, &all_stake_accounts)
        //                     .await
        //                     .unwrap();
        //             }
        //         }

        //         sleep(Duration::from_secs(120)).await;
        //     }
        // });

        // Show current epoch
        tokio::spawn(async move {
            loop {
                let current_epoch = {
                    let lock: MutexGuard<CurrentEpoch> = current_epoch.lock().unwrap();
                    lock.epoch
                };

                info!("Current epoch: {}", current_epoch);

                sleep(Duration::from_secs(3600)).await;
            }
        });

        Ok(Self {})
    }

    async fn get_first_block(
        rpc_client: &RpcClient,
        epoch: Epoch,
    ) -> Result<(Option<Slot>, Option<EncodedConfirmedBlock>), EpochTrackerError> {
        let (epoch_first_slot, _) = EpochStorage::get_first_last_slots(epoch).await?;

        if epoch_first_slot.is_none() {
            panic!("Epoch first slot is none");
        }

        let slot = rpc_client
            .get_blocks(
                epoch_first_slot.unwrap(),
                Some(epoch_first_slot.unwrap() + 100),
            )
            .await?
            .first()
            .cloned();

        if slot.is_none() {
            panic!("Slot is none");
        }

        let block_raw;
        loop {
            match rpc_client
                .get_block_with_config(
                    slot.unwrap(),
                    RpcBlockConfig {
                        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
                        transaction_details: Some(
                            solana_transaction_status::TransactionDetails::Full,
                        ),
                        rewards: Some(true),
                        commitment: Some(CommitmentConfig::finalized()),
                        max_supported_transaction_version: Some(0),
                    },
                )
                .await
            {
                Ok(block) => {
                    block_raw = block;
                    break;
                }
                Err(e) => {
                    error!("Error while trying to get epoch: {:?}", e);
                }
            }

            sleep(Duration::from_secs(1)).await;
        }

        Ok((slot, Some(block_raw.into())))
    }

    async fn get_last_block(
        rpc_client: &RpcClient,
        epoch: Epoch,
    ) -> Result<(Option<Slot>, Option<EncodedConfirmedBlock>), EpochTrackerError> {
        let (_, epoch_last_slot) = EpochStorage::get_first_last_slots(epoch).await?;

        if epoch_last_slot.is_none() {
            return Ok((None, None));
        }

        let slot = rpc_client
            .get_blocks(
                epoch_last_slot.unwrap() - 100,
                Some(epoch_last_slot.unwrap()),
            )
            .await?
            .last()
            .cloned();

        if slot.is_none() {
            return Ok((None, None));
        }

        let block_raw;

        loop {
            if let Ok(block) = rpc_client
                .get_block_with_config(
                    slot.unwrap(),
                    RpcBlockConfig {
                        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
                        transaction_details: Some(
                            solana_transaction_status::TransactionDetails::Full,
                        ),
                        rewards: Some(true),
                        commitment: Some(CommitmentConfig::finalized()),
                        max_supported_transaction_version: Some(0),
                    },
                )
                .await
            {
                block_raw = block.into();
                break;
            }

            sleep(Duration::from_secs(1)).await;
        }

        Ok((slot, Some(block_raw)))
    }

    async fn setup_epochs(epoch_info: &mut EpochInfo) -> Result<(), EpochTrackerError> {
        for _ in 0..epoch_info.epoch {
            epoch_info.epoch -= 1;
            epoch_info.absolute_slot -= epoch_info.slots_in_epoch;

            EpochStorage::store_epoch(epoch_info).await?;
        }

        Ok(())
    }

    async fn _get_stake_accounts(
        rpc_client: &RpcClient,
        validator: &str,
    ) -> Result<Vec<(Pubkey, Account)>, EpochTrackerError> {
        let mut program_accounts_config = RpcProgramAccountsConfig {
            account_config: RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                ..RpcAccountInfoConfig::default()
            },
            ..RpcProgramAccountsConfig::default()
        };

        let pubkey = Pubkey::from_str(validator).unwrap();
        let key_arr = [pubkey];
        let vote_account_pubkeys = key_arr.as_ref();

        program_accounts_config.filters = Some(vec![
            // Filter by `StakeState::Stake(_, _)`
            rpc_filter::RpcFilterType::Memcmp(rpc_filter::Memcmp::new_base58_encoded(
                0,
                &[2, 0, 0, 0],
            )),
            // Filter by `Delegation::voter_pubkey`, which begins at byte offset 124
            rpc_filter::RpcFilterType::Memcmp(rpc_filter::Memcmp::new_base58_encoded(
                124,
                vote_account_pubkeys[0].as_ref(),
            )),
        ]);

        let all_stake_accounts = rpc_client
            .get_program_accounts_with_config(&stake::program::id(), program_accounts_config)
            .await?;

        Ok(all_stake_accounts)
    }
}
