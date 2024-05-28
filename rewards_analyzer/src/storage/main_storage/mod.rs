use super::epoch_storage::Epoch;
use crate::{errors::MainStorageError, register::Register};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_transaction_status::Reward;

pub mod http_client;
pub mod migrations;
pub mod tcp_client;

#[derive(Row, Deserialize)]
pub struct LookupVoteAccRec {
    pub slot: u64,
    pub raw_instruction_idx: u16,
    pub vote_acc: Option<String>,
    pub is_delegation: bool,
}

#[derive(Row, Debug, Serialize, Deserialize)]
pub struct DelegationRec {
    pub slot: u64,
    pub block_time: u32,
    pub stake_acc: String,
    pub vote_acc: Option<String>,
    pub tx_signature: String,
    pub amount: u64,
    pub raw_instruction_idx: u16,
}

#[derive(Row, Serialize)]
pub struct RewardRec<'a> {
    pub vote_account: String,
    pub epoch: Epoch,
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: u64,
    pub reward_type: Option<&'a str>,
    pub commission: Option<u8>,
    pub first_block_slot: Option<u64>,
    pub block_time: u32,
}

#[derive(Default, Row, Deserialize)]
pub struct RewardRecResult {
    pub vote_account: String,
    pub epoch: Epoch,
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: u64,
    pub reward_type: Option<String>,
    pub commission: Option<u8>,
    pub first_block_slot: Option<u64>,
    pub block_time: u32,
}

#[async_trait]
pub trait MainStorage: Send {
    async fn execute(&mut self, ddl: &str) -> Result<(), MainStorageError>;
    async fn migration_exists(&mut self, version: &str) -> Result<bool, MainStorageError>;
    async fn clean_unfinished(&mut self, epoch: Epoch) -> Result<(), MainStorageError>;
    async fn lookup_vote_acc(
        &mut self,
        slot: u64,
        stake_acc: &str,
    ) -> Result<Option<String>, MainStorageError>;
    async fn store_rewards_block(
        &mut self,
        rewards: Vec<(String, Epoch, Option<u64>, Reward, i64)>,
    ) -> Result<(), MainStorageError>;
    async fn get_rewards_with_empty_vote_acc(
        &mut self,
    ) -> Result<Vec<RewardRecResult>, MainStorageError>;
    async fn update_reward(
        &mut self,
        vote_acc: &str,
        epoch: Epoch,
        pubkey: &str,
    ) -> Result<(), MainStorageError>;
}

pub async fn connect_main_storage() -> Result<Box<dyn MainStorage>, MainStorageError> {
    let register_current_state = Register::current().clone();
    let url = register_current_state.configuration.main_storage_url();
    let dsn = dsn::parse(url).unwrap();

    if dsn.driver == *"https" || dsn.driver == *"http" {
        return Ok(Box::new(http_client::HttpClient::new(dsn).await?));
    }

    if dsn.driver == *"tcp" {
        return Ok(Box::new(tcp_client::TcpClient::new(dsn).await?));
    }

    Err(MainStorageError::UnknownProtocol)
}
