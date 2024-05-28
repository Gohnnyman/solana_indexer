use std::str::FromStr;

use crate::solana_client::{SolanaClient, TRANSACTIONS_BATCH_LEN};
use async_trait::async_trait;
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_storage_bigtable::LedgerStorage;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};

pub struct SolanaBigTableClient {
    pub(crate) rpc_client: LedgerStorage,
}

#[async_trait]
impl SolanaClient for SolanaBigTableClient {
    async fn load_signatures_batch(
        &self,
        account_key: &Pubkey,
        before: Option<Signature>,
        until: Option<Signature>,
    ) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>, ClientError> {
        let before_signature = before.as_ref();
        let until_signature = until.as_ref();

        let signatures = self
            .rpc_client
            .get_confirmed_signatures_for_address(
                account_key,
                before_signature,
                until_signature,
                TRANSACTIONS_BATCH_LEN,
            )
            .await
            .map_err(|_| ClientError {
                request: None,
                kind: ClientErrorKind::Custom(String::from("BigTableError")),
            })?;

        let result_signatures = signatures
            .into_iter()
            .map(|v| RpcConfirmedTransactionStatusWithSignature::from(v.0))
            .collect();

        Ok(result_signatures)
    }

    async fn load_transaction_info(
        &self,
        signature: &str,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta, ClientError> {
        let signature = Signature::from_str(signature).unwrap();

        let tx = self
            .rpc_client
            .get_confirmed_transaction(&signature)
            .await
            .map_err(|_| ClientError {
                request: None,
                kind: ClientErrorKind::Custom(String::from("BigTableError")),
            })?
            .unwrap();

        Ok(tx.encode(UiTransactionEncoding::Json, None).unwrap())
    }
}
