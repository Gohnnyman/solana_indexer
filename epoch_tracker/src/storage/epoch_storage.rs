use log::{debug, info};
use serde_json::json;
use solana_sdk::{
    account::Account,
    account_utils::StateMut,
    clock::{Epoch, Slot},
    epoch_info::EpochInfo,
    pubkey::Pubkey,
    stake::state::StakeState,
};
use solana_transaction_status::EncodedConfirmedBlock;
use tokio_postgres::{Client, NoTls};

use crate::{errors::EpochStorageError, register::Register};

use super::migrations::Migration;

const SCRIPTS_UP: [(&str, &str); 2] = [
    (
        "00000000000000_initial_setup",
        include_str!("./migrations/00000000000000_initial_setup/up.sql"),
    ),
    (
        "2022-12-18-013028_create_table_epochs",
        include_str!("./migrations/2022-12-18-013028_create_table_epochs/up.sql"),
    ),
];

pub struct EpochStorage {}

impl EpochStorage {
    pub async fn run_migrations() -> Result<(), EpochStorageError> {
        let migration = Migration::new();
        let mut client = Self::connect().await?;
        migration.up(&mut client, &SCRIPTS_UP).await?;
        Ok(())
    }

    pub async fn store_epoch(epoch_info: &EpochInfo) -> Result<(), EpochStorageError> {
        debug!("Trying to store Info for {:?} epoch", epoch_info.epoch);

        let client = Self::connect().await?;

        let stmt = client
            .prepare("INSERT INTO epochs (epoch, first_slot, last_slot) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .await?;

        let first_slot = epoch_info.absolute_slot - epoch_info.slot_index;
        let last_slot = first_slot + epoch_info.slots_in_epoch - 1;

        let _ = client
            .execute(
                &stmt,
                &[
                    &(epoch_info.epoch as i32),
                    &(first_slot as i32),
                    &(last_slot as i32),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_first_last_slots(
        epoch: Epoch,
    ) -> Result<(Option<Slot>, Option<Slot>), EpochStorageError> {
        debug!(
            "Trying to retrieve first and last slots for {:?} epoch",
            epoch
        );

        let client = Self::connect().await?;

        let stmt = client
            .prepare("SELECT first_slot, last_slot FROM epochs WHERE epoch = $1")
            .await?;

        let response = client.query(&stmt, &[&(epoch as i32)]).await?;

        if response.is_empty() {
            Ok((None, None))
        } else {
            let first_slot: Option<i32> = response.first().unwrap().get(0);
            let first_slot = first_slot.map(|first_slot| first_slot as u64);

            let last_slot: Option<i32> = response.first().unwrap().get(1);
            let last_slot = last_slot.map(|last_slot| last_slot as u64);

            Ok((first_slot, last_slot))
        }
    }

    pub async fn get_epoch_with_empty_first_block() -> Result<Vec<Epoch>, EpochStorageError> {
        debug!("Trying to retrieve the list of the Epoch with empty first_block field");

        let client = Self::connect().await?;

        let stmt = client
            .prepare("SELECT epoch FROM epochs WHERE first_block IS NULL ORDER BY epoch DESC")
            .await?;

        let response = client.query(&stmt, &[]).await?;

        let mut epoch_list = Vec::new();

        for row in response.iter() {
            let epoch: i32 = row.get(0);
            epoch_list.push(epoch as Epoch);
        }

        Ok(epoch_list)
    }

    pub async fn update_first_block_for_epoch(
        epoch: Epoch,
        first_block: Slot,
        first_block_raw: &Option<EncodedConfirmedBlock>,
    ) -> Result<(), EpochStorageError> {
        info!("Trying to update first_block field for {:?} epoch", epoch);

        let client = Self::connect().await?;

        let first_block_json = json!((*first_block_raw).as_ref().unwrap());

        let first_block_raw = (*first_block_raw)
            .as_ref()
            .map(|fb| serde_json::to_string(&fb).unwrap());

        let stmt = client
            .prepare("UPDATE epochs SET first_block = $1, first_block_raw = $2, first_block_json = $3 WHERE epoch = $4")
            .await?;

        let _ = client
            .execute(
                &stmt,
                &[
                    &(first_block as i32),
                    &first_block_raw,
                    &first_block_json,
                    &(epoch as i32),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_epoch_with_empty_last_block() -> Result<Vec<Epoch>, EpochStorageError> {
        debug!("Trying to retrieve the list of the Epoch with empty last_block field");

        let client = Self::connect().await?;

        let stmt = client
            .prepare("SELECT epoch FROM epochs WHERE last_block IS NULL ORDER BY epoch DESC")
            .await?;

        let response = client.query(&stmt, &[]).await?;

        let mut epoch_list = Vec::new();

        for row in response.iter() {
            let epoch: i32 = row.get(0);
            epoch_list.push(epoch as Epoch);
        }

        Ok(epoch_list)
    }

    pub async fn update_last_block_for_epoch(
        epoch: Epoch,
        last_block: Slot,
        last_block_raw: &Option<EncodedConfirmedBlock>,
    ) -> Result<(), EpochStorageError> {
        info!("Trying to update last_block field for {:?} epoch", epoch);

        let client = Self::connect().await?;

        let last_block_json = json!((*last_block_raw).as_ref().unwrap());

        let last_block_raw = (*last_block_raw)
            .as_ref()
            .map(|lb| serde_json::to_string(&lb).unwrap());

        let stmt = client
            .prepare("UPDATE epochs SET last_block = $1, last_block_raw = $2, last_block_json = $3 WHERE epoch = $4")
            .await?;

        let _ = client
            .execute(
                &stmt,
                &[
                    &(last_block as i32),
                    &last_block_raw,
                    &last_block_json,
                    &(epoch as i32),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn _stakes_loaded(epoch: Epoch) -> Result<bool, EpochStorageError> {
        let client = Self::connect().await?;

        let stmt = client
            .prepare("SELECT epoch FROM epochs WHERE epoch = $1 and stakes IS NULL")
            .await?;

        let response = client.query(&stmt, &[&(epoch as i32)]).await?;

        Ok(response.is_empty())
    }

    pub async fn _update_all_stakes(
        epoch: Epoch,
        all_stake_accounts: &Vec<(Pubkey, Account)>,
    ) -> Result<(), EpochStorageError> {
        info!("Trying to update stakes field for {:?} epoch", epoch);

        let mut stakes = Vec::new();

        for (stake_pubkey, stake_account) in all_stake_accounts {
            let stake_state: StakeState = stake_account.state().unwrap();
            if let StakeState::Stake(_, stake) = stake_state {
                stakes.push((stake_pubkey.to_string(), stake.delegation))
            }
        }

        let stakes_json = json!(stakes);

        let client = Self::connect().await?;

        let stmt = client
            .prepare("UPDATE epochs SET stakes = $1 WHERE epoch = $2")
            .await?;

        let _ = client
            .execute(&stmt, &[&stakes_json, &(epoch as i32)])
            .await?;

        Ok(())
    }

    async fn connect() -> Result<Client, EpochStorageError> {
        let register_current_state = Register::current().clone();
        let url = register_current_state.configuration.storage_url();

        let (client, connection) = tokio_postgres::connect(url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                println!("connection error: {}", e);
            }
        });

        Ok(client)
    }
}
