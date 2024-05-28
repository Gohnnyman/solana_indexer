mod big_table_client;
mod rpc_client;

pub use big_table_client::*;
pub use rpc_client::*;

use async_trait::async_trait;
use serde::Deserialize;
use solana_client::{
    client_error::ClientError, nonblocking::rpc_client::RpcClient,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_storage_bigtable::LedgerStorage;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

// Attention! TRANSACTIONS_BATCH_LEN should not be less than 2
pub const TRANSACTIONS_BATCH_LEN: usize = 500;

#[derive(Debug, Clone, Deserialize)]
pub enum ClientType {
    Rpc,
    BigTable,
}

#[async_trait]
pub trait SolanaClient: Sync + Send {
    async fn load_signatures_batch(
        &self,
        account_key: &Pubkey,
        before: Option<Signature>,
        until: Option<Signature>,
    ) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>, ClientError>;

    async fn load_transaction_info(
        &self,
        signature: &str,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta, ClientError>;
}

pub async fn new_with_url(client_type: &ClientType, url: &str) -> Box<dyn SolanaClient> {
    match client_type {
        ClientType::Rpc => Box::new(SolanaRpcClient {
            rpc_client: RpcClient::new(url.to_string()),
        }),
        ClientType::BigTable => Box::new(SolanaBigTableClient {
            rpc_client: LedgerStorage::new(true, None, Some(url.to_string()))
                .await
                .unwrap(),
        }),
    }
}
