use super::super::Metadata as NativeMetadata;
use crate::errors::RabbitMQError;
use anyhow::Result;
use metadata_generated::metadata::*;
use solana_program::message::MessageHeader;
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransaction, EncodedTransactionWithStatusMeta, Reward, RewardType, UiAddressTableLookup,
    UiCompiledInstruction, UiInnerInstructions, UiInstruction, UiMessage, UiRawMessage,
    UiTransaction, UiTransactionStatusMeta, UiTransactionTokenBalance,
};
use transaction_info_generated::transaction_info::{
    root_as_transaction_info, RewardType as TransactionInfoRewardType, SanitizedMessage,
};

use rust_base58::ToBase58;
use solana_account_decoder::parse_token::UiTokenAmount;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::all))]
mod metadata_generated;
#[cfg_attr(feature = "cargo-clippy", allow(clippy::all))]
mod transaction_info_generated;

pub fn deserialize_metadata(data: &[u8]) -> Result<NativeMetadata> {
    let metadata = root_as_metadata(data)?;

    Ok(NativeMetadata {
        slot: metadata.slot(),
        blockhash: metadata.blockhash().unwrap().to_string(),
        rewards: metadata.rewards().unwrap().to_string(),
        block_time: metadata.block_time(),
        block_height: if metadata.block_height() == 0 {
            None
        } else {
            Some(metadata.block_height())
        },
    })
}

pub fn deserialize_transaction(data: &[u8]) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    let transaction_info = root_as_transaction_info(data)?;

    let slot: u64 = transaction_info.slot();
    let transaction = {
        let meta_info = transaction_info.transaction_meta().unwrap();
        let meta = Some(UiTransactionStatusMeta {
            err: None,
            status: Ok(()),
            fee: meta_info.fee(),
            pre_balances: meta_info.pre_balances().unwrap().safe_slice().into(),
            post_balances: meta_info.post_balances().unwrap().safe_slice().into(),
            inner_instructions: Some(
                meta_info
                    .inner_instructions()
                    .unwrap()
                    .iter()
                    .map(|inn| UiInnerInstructions {
                        index: inn.index(),
                        instructions: inn
                            .instructions()
                            .unwrap()
                            .iter()
                            .map(|inst| {
                                UiInstruction::Compiled(UiCompiledInstruction {
                                    program_id_index: inst.program_id_index(),
                                    accounts: inst.accounts().unwrap().into(),
                                    data: inst.data().unwrap().to_base58(),
                                })
                            })
                            .collect(),
                    })
                    .collect(),
            )
            .into(),
            log_messages: Some(
                meta_info
                    .log_messages()
                    .unwrap()
                    .iter()
                    .map(|log| log.to_string())
                    .collect(),
            )
            .into(),
            pre_token_balances: Some(
                meta_info
                    .pre_token_balances()
                    .unwrap()
                    .iter()
                    .map(|token| {
                        let ui_token_amount = token.ui_token_amount().unwrap();
                        UiTransactionTokenBalance {
                            account_index: token.account_index(),
                            mint: token.mint().unwrap().to_string(),
                            ui_token_amount: UiTokenAmount {
                                ui_amount: Some(ui_token_amount.ui_amount()),
                                decimals: ui_token_amount.decimals(),
                                amount: ui_token_amount.amount().unwrap().to_string(),
                                ui_amount_string: ui_token_amount
                                    .ui_amount_string()
                                    .unwrap()
                                    .to_string(),
                            },
                            owner: Some(token.owner().unwrap().to_string()).into(),
                            program_id: token.program_id().map(|val| val.to_string()).into(),
                        }
                    })
                    .collect(),
            )
            .into(),
            post_token_balances: Some(
                meta_info
                    .post_token_balances()
                    .unwrap()
                    .iter()
                    .map(|token| {
                        let ui_token_amount = token.ui_token_amount().unwrap();
                        UiTransactionTokenBalance {
                            account_index: token.account_index(),
                            mint: token.mint().unwrap().to_string(),
                            ui_token_amount: UiTokenAmount {
                                ui_amount: Some(ui_token_amount.ui_amount()),
                                decimals: ui_token_amount.decimals(),
                                amount: ui_token_amount.amount().unwrap().to_string(),
                                ui_amount_string: ui_token_amount
                                    .ui_amount_string()
                                    .unwrap()
                                    .to_string(),
                            },
                            owner: Some(token.owner().unwrap().to_string()).into(),
                            program_id: token.program_id().map(|val| val.to_string()).into(),
                        }
                    })
                    .collect(),
            )
            .into(),
            rewards: Some(
                meta_info
                    .rewards()
                    .unwrap()
                    .iter()
                    .map(|reward| Reward {
                        pubkey: reward.pubkey().unwrap().to_string(),
                        lamports: reward.lamports(),
                        post_balance: reward.post_balance(),
                        reward_type: match reward.reward_type() {
                            TransactionInfoRewardType::Rent => Some(RewardType::Rent),
                            TransactionInfoRewardType::Fee => Some(RewardType::Fee),
                            TransactionInfoRewardType::Staking => Some(RewardType::Staking),
                            TransactionInfoRewardType::Voting => Some(RewardType::Voting),
                            _ => None,
                        },
                        commission: Some(reward.commission()),
                    })
                    .collect(),
            )
            .into(),
            loaded_addresses: OptionSerializer::None,
            return_data: OptionSerializer::None,
            compute_units_consumed: OptionSerializer::None,
        });

        let sanitized_transaction = transaction_info.transaction().unwrap();
        let message_type = sanitized_transaction.message_type();
        let message = match message_type {
            SanitizedMessage::Legacy => {
                let legacy_message = sanitized_transaction.message_as_legacy().unwrap();

                UiMessage::Raw(UiRawMessage {
                    header: MessageHeader {
                        num_readonly_signed_accounts: legacy_message
                            .header()
                            .unwrap()
                            .num_readonly_signed_accounts(),
                        num_readonly_unsigned_accounts: legacy_message
                            .header()
                            .unwrap()
                            .num_readonly_unsigned_accounts(),
                        num_required_signatures: legacy_message
                            .header()
                            .unwrap()
                            .num_required_signatures(),
                    },
                    account_keys: legacy_message
                        .account_keys()
                        .unwrap()
                        .iter()
                        .map(|key| key.key().unwrap().to_base58())
                        .collect(),
                    recent_blockhash: legacy_message.recent_blockhash().unwrap().to_base58(),
                    instructions: legacy_message
                        .instructions()
                        .unwrap()
                        .iter()
                        .map(|inst| UiCompiledInstruction {
                            program_id_index: inst.program_id_index(),
                            accounts: inst.accounts().unwrap().into(),
                            data: inst.data().unwrap().to_base58(),
                        })
                        .collect(),
                    address_table_lookups: None,
                })
            }
            SanitizedMessage::V0 => {
                let v0_message = sanitized_transaction.message_as_v0().unwrap();
                let message = v0_message.message().unwrap();

                UiMessage::Raw(UiRawMessage {
                    header: MessageHeader {
                        num_readonly_signed_accounts: message
                            .header()
                            .unwrap()
                            .num_readonly_signed_accounts(),
                        num_readonly_unsigned_accounts: message
                            .header()
                            .unwrap()
                            .num_readonly_unsigned_accounts(),
                        num_required_signatures: message
                            .header()
                            .unwrap()
                            .num_required_signatures(),
                    },
                    account_keys: message
                        .account_keys()
                        .unwrap()
                        .iter()
                        .map(|key| key.key().unwrap().to_base58())
                        .collect(),
                    recent_blockhash: message.recent_blockhash().unwrap().to_base58(),
                    instructions: message
                        .instructions()
                        .unwrap()
                        .iter()
                        .map(|inst| UiCompiledInstruction {
                            program_id_index: inst.program_id_index(),
                            accounts: inst.accounts().unwrap().into(),
                            data: inst.data().unwrap().to_base58(),
                        })
                        .collect(),
                    address_table_lookups: Some(
                        message
                            .address_table_lookups()
                            .unwrap()
                            .iter()
                            .map(|lookup| UiAddressTableLookup {
                                account_key: lookup
                                    .account_key()
                                    .unwrap()
                                    .key()
                                    .unwrap()
                                    .to_base58(),
                                writable_indexes: lookup.writable_indexes().unwrap().into(),
                                readonly_indexes: lookup.readonly_indexes().unwrap().into(),
                            })
                            .collect(),
                    ),
                })
            }
            _ => {
                return Err(anyhow::anyhow!(RabbitMQError::DeserializationError(
                    "Invalid SanitizedMessage".to_string()
                )));
            }
        };

        let transaction = EncodedTransaction::Json(UiTransaction {
            signatures: sanitized_transaction
                .signatures()
                .unwrap()
                .iter()
                .map(|sig| sig.key().unwrap().to_base58())
                .collect(),
            message,
        });

        EncodedTransactionWithStatusMeta {
            transaction,
            meta,
            version: None,
        }
    };

    Ok(EncodedConfirmedTransactionWithStatusMeta {
        transaction,
        slot,
        block_time: None,
    })
}
