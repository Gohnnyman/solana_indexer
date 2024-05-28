use std::str::FromStr;

use crate::solana_client::{SolanaClient, TRANSACTIONS_BATCH_LEN};
use async_trait::async_trait;
use solana_client::{
    client_error::ClientError, nonblocking::rpc_client::RpcClient,
    rpc_client::GetConfirmedSignaturesForAddress2Config, rpc_config::RpcTransactionConfig,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};

pub struct SolanaRpcClient {
    pub(crate) rpc_client: RpcClient,
}

#[async_trait]
impl SolanaClient for SolanaRpcClient {
    async fn load_signatures_batch(
        &self,
        account_key: &Pubkey,
        before: Option<Signature>,
        until: Option<Signature>,
    ) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>, ClientError> {
        let config = GetConfirmedSignaturesForAddress2Config {
            before,
            until,
            limit: Some(TRANSACTIONS_BATCH_LEN),
            commitment: Some(CommitmentConfig::finalized()),
        };

        self.rpc_client
            .get_signatures_for_address_with_config(account_key, config)
            .await
    }

    async fn load_transaction_info(
        &self,
        signature: &str,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta, ClientError> {
        let signature = Signature::from_str(signature).unwrap();
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };

        self.rpc_client
            .get_transaction_with_config(&signature, config)
            .await
    }
}
