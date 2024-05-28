use std::{str::FromStr, time::Duration};

use anyhow::Result;
use log::info;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio::time::sleep;

use crate::{
    actors::{
        saved_state_manager::SavedStateManagerHandle, signatures_rpc_loader::*,
        signatures_saver::SignaturesSaverHandle,
    },
    register::Register,
};

pub struct SignaturesLoadingCtx;

impl SignaturesLoadingCtx {
    pub async fn setup_and_run(register: &Register) -> Result<Self> {
        for key in register.config.get_account_keys() {
            let contract_address = key.clone();
            let contract_address_for_logging = key.clone();
            let rpc_loader = SignaturesRpcLoaderHandle::new(
                register.config.get_solana_client_type(),
                &register.config.get_endpoint_url(),
                &key,
            )
            .await;

            let signatures_saver = SignaturesSaverHandle::new(register).await?;

            let saved_state_manager = SavedStateManagerHandle::new(register).await?;

            let mut saved_state = saved_state_manager
                .load_state(Pubkey::from_str(&key).unwrap())
                .await;

            info!(
                "{}: Saved state loaded: {:?}",
                &contract_address_for_logging, &saved_state
            );

            let mut sleep_time = 0;

            tokio::spawn(async move {
                loop {
                    let signatures = rpc_loader.signatures_rpc_load(saved_state).await;

                    info!(
                        "{}: {} signatures loaded",
                        &contract_address_for_logging,
                        signatures.len()
                    );

                    if saved_state.newest_transaction.is_none() && !signatures.is_empty() {
                        saved_state.newest_transaction = Some(
                            Signature::from_str(&signatures.get(0).unwrap().signature).unwrap(),
                        );
                    }

                    if signatures.is_empty() {
                        if sleep_time < 5000 {
                            sleep_time += 1000;
                        }

                        sleep(Duration::from_millis(sleep_time)).await;
                        continue;
                    } else {
                        sleep_time = 0;

                        let before_idx = signatures.len().saturating_sub(2);

                        info!(
                            "{}: first in a batch: {}",
                            &contract_address_for_logging,
                            &signatures.get(0).unwrap().signature
                        );
                        info!(
                            "{}: new before: {}",
                            &contract_address_for_logging,
                            &signatures.get(before_idx).unwrap().signature
                        );

                        saved_state.before = Some(
                            Signature::from_str(&signatures.get(before_idx).unwrap().signature)
                                .unwrap(),
                        );
                    };

                    let until = saved_state.until.unwrap_or_default().to_string();

                    if signatures.iter().any(|s| s.signature == until) {
                        // We have loaded all retrospective transactions signatures.
                        // Move the the head to the current top and the end of a tail to the prev one.
                        if saved_state.newest_transaction.is_some() {
                            saved_state.until = saved_state.newest_transaction;
                        }

                        info!(
                            "{}: until updated: {:?}",
                            &contract_address_for_logging, saved_state.until
                        );

                        saved_state.before = None;
                        saved_state.newest_transaction = None;
                    }

                    let signatures_to_store = signatures.len();
                    info!(
                        "{}: {} signatures sent to storage ",
                        &contract_address_for_logging, &signatures_to_store
                    );

                    let signatures_stored = signatures_saver
                        .store_signatures_and_state(
                            signatures,
                            Pubkey::from_str(&key).unwrap(),
                            saved_state,
                        )
                        .await;

                    info!(
                        "{}: {} signatures stored ",
                        &contract_address_for_logging, &signatures_stored
                    );

                    if signatures_to_store > 0 && signatures_stored == 0 {
                        saved_state.before = None;
                        saved_state.newest_transaction = None;
                        sleep(Duration::from_millis(5000)).await;
                    }
                }
            });
            info!("{}: Signature loader spawned", contract_address);
        }

        Ok(Self {})
    }
}
