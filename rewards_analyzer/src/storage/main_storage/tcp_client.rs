use super::{super::epoch_storage::Epoch, MainStorage, RewardRecResult};
use crate::errors::MainStorageError;
use async_trait::async_trait;
use chrono_tz::Tz;
use clickhouse_rs::{
    row,
    types::{Block, Value},
    ClientHandle, Pool,
};
use dsn::DSN;
use futures::future::Future;
use futures::future::TryFuture;
use futures::TryFutureExt;
use log::info;

use solana_transaction_status::{Reward, RewardType};

pub struct TcpClient {
    // client: ClientHandle,
    client: Pool,
}

impl TcpClient {
    pub async fn new(db_creds: DSN) -> Result<Self, MainStorageError> {
        let mut database_url = format!("{}://", db_creds.driver);

        if let Some(user_name) = db_creds.username {
            database_url = format!("{database_url}{user_name}");
        }
        if let Some(password) = db_creds.password {
            database_url = format!("{database_url}:{password}");
        }

        let address = db_creds.address;
        database_url = format!("{database_url}@{address}");

        if let Some(db) = db_creds.database {
            database_url = format!("{database_url}/{db}");
        }

        let client = Self::connect(&database_url).await?;
        Ok(Self { client })
    }

    pub async fn connect(url: &str) -> Result<Pool, MainStorageError> {
        let pool = Pool::new(url);
        // let pool = pool.max_size(1).build();
        // let client = pool.get_handle().await?;

        Ok(pool)
    }
}

#[async_trait]
impl MainStorage for TcpClient {
    async fn execute(&mut self, ddl: &str) -> Result<(), MainStorageError> {
        // self.client
        //     .get_handle()
        //     .and_then(move |mut c| c.execute(ddl))
        //     .await?;

        self.client.get_handle().await?.execute(ddl).await?;

        Ok(())
    }

    async fn migration_exists(&mut self, version: &str) -> Result<bool, MainStorageError> {
        let query = &format!(
            "SELECT COUNT(*) AS count FROM __schema_migrations WHERE version = '{}'",
            version
        );

        // let block = self.client.query(query).fetch_all().await?;
        let block = self
            .client
            .get_handle()
            .await?
            .query(query)
            .fetch_all()
            .await?;

        return if let Some(row) = block.rows().next() {
            let count: u64 = row.get("count")?;
            Ok(count > 0)
        } else {
            Ok(false)
        };
    }

    #[cfg(feature = "on_ch_cluster")]
    async fn clean_unfinished(&mut self, epoch: Epoch) -> Result<(), MainStorageError> {
        let ddl = format!(
            "ALTER TABLE rewards ON CLUSTER '{{cluster}}' DELETE WHERE epoch = {}",
            epoch - 1
        );
        // self.client.execute(ddl).await?;

        self.client.get_handle().await?.execute(ddl).await?;

        Ok(())
    }

    #[cfg(not(feature = "on_ch_cluster"))]
    async fn clean_unfinished(&mut self, epoch: Epoch) -> Result<(), MainStorageError> {
        let ddl = format!("ALTER TABLE rewards DELETE WHERE epoch = {}", epoch - 1);
        // self.client.execute(ddl).await?;
        self.client.get_handle().await?.execute(ddl).await?;

        Ok(())
    }

    async fn lookup_vote_acc(
        &mut self,
        slot: u64,
        stake_acc: &str,
    ) -> Result<Option<String>, MainStorageError> {
        let ddl = format!(
            "
                SELECT * FROM (
                    SELECT 
                        slot, raw_instruction_idx, vote_acc, 1 as is_delegation
                    FROM delegations
                    WHERE stake_acc = '{}' AND slot <= {}
                    ORDER BY slot DESC, raw_instruction_idx DESC
                    LIMIT 1
                    UNION ALL 
                    SELECT 
                        slot, raw_instruction_idx, vote_acc, 0 as is_delegation
                    FROM undelegations
                    WHERE stake_acc = '{}' AND slot <= {}
                    ORDER BY slot DESC, raw_instruction_idx DESC
                    LIMIT 1
                ) ORDER BY slot DESC, raw_instruction_idx DESC LIMIT 1
            ",
            stake_acc, slot, stake_acc, slot
        );

        // let block = self.client.query(&ddl).fetch_all().await?;

        let block = self
            .client
            .get_handle()
            .await?
            .query(ddl)
            .fetch_all()
            .await?;

        if let Some(row) = block.rows().next() {
            let is_delegation: u8 = row.get(3)?;
            if is_delegation != 0 {
                return Ok(row.get(2)?);
            }
        }

        Ok(None)
    }

    async fn store_rewards_block(
        &mut self,
        rewards: Vec<(String, Epoch, Option<u64>, Reward, i64)>,
    ) -> Result<(), MainStorageError> {
        let block_size = rewards.len();

        let mut block = Block::with_capacity(block_size);

        for reward in rewards {
            let reward_type = reward.3.reward_type.map(|rew_type| match rew_type {
                RewardType::Fee => "fee",
                RewardType::Rent => "rent",
                RewardType::Staking => "staking",
                RewardType::Voting => "voting",
            });

            block.push(row! {
                vote_account: reward.0,
                epoch: reward.1 - 1,
                pubkey: reward.3.pubkey,
                lamports: reward.3.lamports,
                post_balance: reward.3.post_balance,
                reward_type,
                commission: reward.3.commission,
                first_block_slot: reward.2,
                block_time: Value::DateTime(reward.4 as u32, Tz::UTC),
            })?;
        }

        // self.client.insert("rewards", block).await?;

        self.client
            .get_handle()
            .await?
            .insert("rewards", block)
            .await?;

        Ok(())
    }

    async fn get_rewards_with_empty_vote_acc(
        &mut self,
    ) -> Result<Vec<RewardRecResult>, MainStorageError> {
        let ddl = String::from(
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
        );

        // let block = self.client.query(&ddl).fetch_all().await?;

        let block = self
            .client
            .get_handle()
            .await?
            .query(ddl)
            .fetch_all()
            .await?;

        let mut reward_records: Vec<RewardRecResult> = Vec::new();

        for row in block.rows() {
            let vote_account = row.get(0)?;
            let epoch = row.get(1)?;
            let pubkey = row.get(2)?;
            let lamports = row.get(3)?;
            let post_balance = row.get(4)?;
            let reward_type = row.get(5)?;
            let commission = row.get(6)?;
            let first_block_slot = row.get(7)?;
            let block_time = row.get(8)?;

            let reward_record = RewardRecResult {
                vote_account,
                epoch,
                pubkey,
                lamports,
                post_balance,
                reward_type,
                commission,
                first_block_slot,
                block_time,
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

        // self.client.execute(ddl).await?;
        self.client.get_handle().await?.execute(ddl).await?;

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

        // self.client.execute(ddl).await?;
        self.client.get_handle().await?.execute(ddl).await?;

        info!(
            "Updated reward recird: epoch: {}, pubkey: {}, vote_account: {}",
            epoch, pubkey, vote_acc
        );

        Ok(())
    }
}
