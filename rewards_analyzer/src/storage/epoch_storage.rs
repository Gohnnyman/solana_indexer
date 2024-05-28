use solana_transaction_status::{Reward, RewardType, Rewards};
use tokio_postgres::{types::Json, Client, NoTls};

use crate::{errors::EpochStorageError, register::Register};

pub type Epoch = u64;

pub struct EpochStorage {}

impl EpochStorage {
    pub async fn get_not_parsed_first_block_epoch(
    ) -> Result<(Option<Epoch>, Option<u64>), EpochStorageError> {
        let client = Self::connect().await?;

        let stmt = client
            .prepare(
                "
            select
                epoch,
                first_block
            from
                epochs
            where
                first_block_json is not null and
                rewards_parsing_status=0 and
                epoch <= (
                    select  
                        least(max(slot/432000), 99999999999)
                    from
                        signatures s,
                        (
                        select
                            first_slot
                        from
                            epochs
                        where
                            first_block_json is not null and
                            rewards_parsing_status=0
                        order by
                            epoch DESC LIMIT 1
                        ) e
                    where
                        err='' and
                        loading_status != 2 and
                        slot < e.first_slot
                )
            LIMIT 1
            ",
            )
            .await?;

        let response = client.query(&stmt, &[]).await?;

        if response.is_empty() {
            Ok((None, None))
        } else {
            let epoch: Option<i32> = response.first().unwrap().get(0);
            let epoch = epoch.map(|epoch| epoch as u64);

            let first_block: Option<i32> = response.first().unwrap().get(1);
            let first_block = first_block.map(|first_block| first_block as u64);
            Ok((epoch, first_block))
        }
    }

    pub async fn mark_rewards_parsed(epoch: Epoch) -> Result<(), EpochStorageError> {
        let client = Self::connect().await?;

        let stmt = client
            .prepare("UPDATE epochs SET rewards_parsing_status = 1 WHERE epoch = $1")
            .await?;

        client.query(&stmt, &[&(epoch as i32)]).await?;

        Ok(())
    }

    pub async fn get_rewards_records(epoch: Epoch) -> Result<(i64, Rewards), EpochStorageError> {
        let client = Self::connect().await?;

        // retrieve block_time
        let mut stmt = client
            .prepare(
                "
            SELECT 
                (first_block_json->'blockTime')::bigint as blockTime
            FROM
                epochs
            WHERE
                epoch = $1
          ",
            )
            .await?;

        let mut response = client.query(&stmt, &[&(epoch as i32)]).await?;

        let block_time = if let Some(first_row) = response.first() {
            first_row.get::<_, i64>(0)
        } else {
            0
        };

        // retrieve rewards
        stmt = client
            .prepare(
                "
            SELECT 
                json_array_elements((first_block_json->'rewards')::json)
            FROM
                epochs
            WHERE
                epoch = $1
          ",
            )
            .await?;

        response = client.query(&stmt, &[&(epoch as i32)]).await?;

        let mut rewards = Rewards::new();

        for reward in response {
            let rec = reward.get::<_, Json<Reward>>(0);
            if rec.0.reward_type == Some(RewardType::Staking)
                || rec.0.reward_type == Some(RewardType::Voting)
            {
                rewards.push(rec.0);
            }
        }

        Ok((block_time, rewards))
    }

    async fn connect() -> Result<Client, EpochStorageError> {
        let register_current_state = Register::current().clone();
        let url = register_current_state.configuration.epoch_storage_url();

        let (client, connection) = tokio_postgres::connect(url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                println!("connection error: {}", e);
            }
        });

        Ok(client)
    }
}
