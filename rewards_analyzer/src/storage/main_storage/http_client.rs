use super::{
    super::epoch_storage::Epoch, LookupVoteAccRec, MainStorage, RewardRec, RewardRecResult,
};
use crate::errors::MainStorageError;
use anyhow::Result;
use async_trait::async_trait;
use clickhouse_http::Client;
use dsn::DSN;
use log::info;
use solana_transaction_status::{Reward, RewardType};

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub async fn new(db_creds: DSN) -> Result<Self, MainStorageError> {
        let protocol = db_creds.driver;
        let address = db_creds.address;

        let mut client = if protocol == "https" {
            Client::with_https_client().with_url(format!("{protocol}://{address}"))
        } else {
            Client::default().with_url(format!("{protocol}://{address}"))
        };

        if let Some(user_name) = db_creds.username {
            client = client.with_user(user_name);
        }
        if let Some(password) = db_creds.password {
            client = client.with_password(password);
        }

        if let Some(db) = db_creds.database {
            client = client.with_database(db);
        }

        Ok(Self { client })
    }
}

#[async_trait]
impl MainStorage for HttpClient {
    async fn execute(&mut self, ddl: &str) -> Result<(), MainStorageError> {
        let query = self.client.query(ddl);
        query.execute().await?;
        Ok(())
    }

    async fn migration_exists(&mut self, version: &str) -> Result<bool, MainStorageError> {
        let mut cursor = self
            .client
            .query("SELECT COUNT(*) AS count FROM __schema_migrations WHERE version = ?")
            .bind(version)
            .fetch::<u64>()?;

        if let Some(count) = cursor.next().await? {
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    #[cfg(feature = "on_ch_cluster")]
    async fn clean_unfinished(&mut self, epoch: Epoch) -> Result<(), MainStorageError> {
        let ddl = format!(
            "ALTER TABLE rewards ON CLUSTER '{{cluster}}' DELETE WHERE epoch = {}",
            epoch - 1
        );
        self.client.query(&ddl).execute().await?;

        Ok(())
    }

    #[cfg(not(feature = "on_ch_cluster"))]
    async fn clean_unfinished(&mut self, epoch: Epoch) -> Result<(), MainStorageError> {
        let ddl = format!("ALTER TABLE rewards DELETE WHERE epoch = {}", epoch - 1);
        self.client.query(&ddl).execute().await?;

        Ok(())
    }

    async fn lookup_vote_acc(
        &mut self,
        slot: u64,
        stake_acc: &str,
    ) -> Result<Option<String>, MainStorageError> {
        let mut cursor = self
            .client
            .query(
                "
                SELECT * FROM (
                    SELECT 
                        slot, raw_instruction_idx, vote_acc, 1 as is_delegation
                    FROM delegations
                    WHERE stake_acc = ? AND slot <= ?
                    ORDER BY slot DESC, raw_instruction_idx DESC
                    LIMIT 1
                    UNION ALL 
                    SELECT 
                        slot, raw_instruction_idx, vote_acc, 0 as is_delegation
                    FROM undelegations
                    WHERE stake_acc = ? AND slot <= ?
                    ORDER BY slot DESC, raw_instruction_idx DESC
                    LIMIT 1
                ) ORDER BY slot DESC, raw_instruction_idx DESC LIMIT 1
                ",
            )
            .bind(stake_acc.clone())
            .bind(slot)
            .bind(stake_acc.clone())
            .bind(slot)
            .fetch::<LookupVoteAccRec>()?;

        if let Some(row) = cursor.next().await? {
            if row.is_delegation {
                Ok(row.vote_acc)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn store_rewards_block(
        &mut self,
        rewards: Vec<(String, Epoch, Option<u64>, Reward, i64)>,
    ) -> Result<(), MainStorageError> {
        let mut insert = self.client.insert("rewards")?;

        for reward in rewards {
            let reward_type = reward.3.reward_type.map(|rew_type| match rew_type {
                RewardType::Fee => "fee",
                RewardType::Rent => "rent",
                RewardType::Staking => "staking",
                RewardType::Voting => "voting",
            });

            insert
                .write(&RewardRec {
                    vote_account: reward.0,
                    epoch: reward.1 - 1,
                    pubkey: reward.3.pubkey,
                    lamports: reward.3.lamports,
                    post_balance: reward.3.post_balance,
                    reward_type,
                    commission: reward.3.commission,
                    first_block_slot: reward.2,
                    block_time: reward.4 as u32,
                })
                .await?;
        }

        insert.end().await?;

        Ok(())
    }

    async fn get_rewards_with_empty_vote_acc(
        &mut self,
    ) -> Result<Vec<RewardRecResult>, MainStorageError> {
        let mut cursor = self
            .client
            .query(
                "
        SELECT
            vote_account,
            epoch,
            pubkey,
            lamports,
            post_balance,
            reward_type,
            commission,
            first_block_slot,
            block_time
        FROM rewards
        WHERE
            vote_account = ''
            and reward_type = 'staking'",
            )
            .fetch::<RewardRecResult>()?;

        let mut reward_records: Vec<RewardRecResult> = Vec::new();

        while let Some(row) = cursor.next().await? {
            let reward_record = RewardRecResult {
                vote_account: row.vote_account,
                epoch: row.epoch,
                pubkey: row.pubkey,
                lamports: row.lamports,
                post_balance: row.post_balance,
                reward_type: row.reward_type,
                commission: row.commission,
                first_block_slot: row.first_block_slot,
                block_time: row.block_time,
            };

            reward_records.push(reward_record);
        }

        Ok(reward_records)
    }

    #[cfg(feature = "on_ch_cluster")]
    async fn update_reward(
        &mut self,
        vote_acc: &str,
        epoch: Epoch,
        pubkey: &str,
    ) -> Result<(), MainStorageError> {
        let request = format!("
        ALTER TABLE rewards ON CLUSTER '{{cluster}}' UPDATE vote_account = '{}' WHERE epoch = {} AND pubkey = '{}'",
        vote_acc,
        epoch,
        pubkey);

        let ddl = request.as_str();
        self.client.query(ddl).execute().await?;
        info!(
            "Updated reward recird: epoch: {}, pubkey: {}, vote_account: {}",
            epoch, pubkey, vote_acc
        );

        Ok(())
    }

    #[cfg(not(feature = "on_ch_cluster"))]
    async fn update_reward(
        &mut self,
        vote_acc: &str,
        epoch: Epoch,
        pubkey: &str,
    ) -> Result<(), MainStorageError> {
        let request = format!(
            "
        ALTER TABLE rewards UPDATE vote_account = '{}' WHERE epoch = {} AND pubkey = '{}'",
            vote_acc, epoch, pubkey
        );

        let ddl = request.as_str();
        self.client.query(ddl).execute().await?;
        info!(
            "Updated reward recird: epoch: {}, pubkey: {}, vote_account: {}",
            epoch, pubkey, vote_acc
        );

        Ok(())
    }
}
