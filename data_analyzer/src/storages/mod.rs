pub mod main_storage;
pub mod postgre_storage;
// pub mod rabbit_storage;

use self::postgre_storage::models::Delegation;
use anyhow::Result;
use async_trait::async_trait;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

#[async_trait]
pub trait QueueStorage: Send {
    async fn get_transactions(&mut self) -> Vec<EncodedConfirmedTransactionWithStatusMeta>;
    async fn get_delegations(&mut self, stake_accs: Vec<String>) -> Result<Vec<Delegation>>;
    async fn save_delegations(&mut self, delegations: Vec<Delegation>) -> Result<()>;
    async fn mark_transaction_as_parsed(&mut self, transactions: String) -> Result<()>;
}

#[macro_export]
macro_rules! repeat_until_ok {
    ( $func:expr, $sleep_time:expr ) => {{
        loop {
            match $func {
                Ok(result) => break result,
                Err(err) => {
                    log::error!("Error in func {}: {}", stringify!($func), err);
                    tokio::time::sleep(std::time::Duration::from_secs($sleep_time)).await;
                }
            }
        }
    }};
}
