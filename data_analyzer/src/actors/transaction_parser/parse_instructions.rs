use crate::instructions::system_instruction::SystemInstruction;
use crate::instructions::token_metadata_instruction::MetadataInstruction;
use crate::instructions::vote_instruction::VoteInstruction;
use crate::instructions::{
    auction_house_instruction::AuctionHouseInstruction, auction_instruction::AuctionInstruction,
    candy_machine_instruction::CandyMachineInstruction,
    fixed_price_sale_instruction::FixedPriceSaleInstruction,
    gumdrop_instruction::GumdropInstruction, metaplex_instruction::MetaplexInstruction,
    nft_packs_instruction::NFTPacksInstruction, stake_instruction::StakeInstruction,
    token_entangler_instruction::TokenEntanglerInstruction,
    token_vault_instruction::VaultInstruction,
};

use crate::errors::ParseInstructionError;
use crate::storages::main_storage::{
    Balance, Instruction, InstructionArgument, TxStatus, ACCOUNTS_ARRAY_SIZE,
};

use anyhow::Result;
use borsh::BorshDeserialize;
use log::debug;
use solana_sdk::program_utils::limited_deserialize;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiLoadedAddresses, UiMessage,
    UiTransactionTokenBalance,
};
use std::collections::{BTreeSet, HashMap};
use std::convert::TryInto;

use super::{TransactionParser, TransactionParsingResult};

impl TransactionParser {
    pub fn parse_transactions(
        confirmed_transaction: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<TransactionParsingResult, ParseInstructionError> {
        let transaction = confirmed_transaction.transaction.transaction;
        let slot = confirmed_transaction.slot;
        let block_time = confirmed_transaction.block_time.unwrap_or_default();
        let mut parsed_instruction_arguments = Vec::new();
        let mut balances = Vec::new();
        let mut pre_balances_map = HashMap::new();
        let mut inner_instructions = OptionSerializer::None;
        let mut instructions_set: BTreeSet<Instruction> = BTreeSet::new();

        // ToDo: remove this deprecated field. Look at https://github.com/solana-labs/solana/issues/9302
        let mut tx_status = TxStatus::Success;

        if let EncodedTransaction::Json(transaction_json) = transaction {
            let message = transaction_json.message;
            let tx_signature = &transaction_json.signatures[0];

            if let UiMessage::Raw(message_raw) = message {
                if message_raw.account_keys.len() > ACCOUNTS_ARRAY_SIZE {
                    return Err(ParseInstructionError::InvalidLength {
                        site: "accounts".to_string(),
                        len: message_raw.account_keys.len(),
                        expected_len: ACCOUNTS_ARRAY_SIZE,
                    });
                }
                let mut accounts = message_raw.account_keys;
                let instructions = message_raw.instructions;

                //////////////////////////Balances////////////////////////////////////////////
                if let Some(transaction_meta) = confirmed_transaction.transaction.meta {
                    let loaded_addresses =
                        <OptionSerializer<_> as Into<Option<UiLoadedAddresses>>>::into(
                            transaction_meta.loaded_addresses,
                        )
                        .unwrap_or_default();

                    accounts.extend(loaded_addresses.writable.into_iter());
                    accounts.extend(loaded_addresses.readonly.into_iter());

                    inner_instructions = transaction_meta.inner_instructions;
                    let mut pre_balances = vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut post_balances = vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut pre_token_balance_mint = vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut pre_token_balance_owner: Vec<Option<String>> =
                        vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut pre_token_balance_amount =
                        vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut pre_token_balance_program_id: Vec<Option<String>> =
                        vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut post_token_balance_mint = vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut post_token_balance_owner: Vec<Option<String>> =
                        vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut post_token_balance_amount =
                        vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    let mut post_token_balance_program_id: Vec<Option<String>> =
                        vec![Default::default(); ACCOUNTS_ARRAY_SIZE];
                    tx_status = if transaction_meta.status.is_ok() {
                        TxStatus::Success
                    } else {
                        TxStatus::Failed
                    };

                    if transaction_meta.pre_balances.len() > ACCOUNTS_ARRAY_SIZE {
                        return Err(ParseInstructionError::InvalidLength {
                            site: "pre_balances".to_string(),
                            len: transaction_meta.pre_balances.len(),
                            expected_len: ACCOUNTS_ARRAY_SIZE,
                        });
                    }
                    transaction_meta
                        .pre_balances
                        .iter()
                        .enumerate()
                        .for_each(|(i, pre_balance)| pre_balances[i] = Some(*pre_balance));

                    if transaction_meta.post_balances.len() > ACCOUNTS_ARRAY_SIZE {
                        return Err(ParseInstructionError::InvalidLength {
                            site: "post_balances".to_string(),
                            len: transaction_meta.post_balances.len(),
                            expected_len: ACCOUNTS_ARRAY_SIZE,
                        });
                    }

                    transaction_meta
                        .post_balances
                        .iter()
                        .enumerate()
                        .for_each(|(i, post_balance)| post_balances[i] = Some(*post_balance));

                    let pre_token_balances: Option<Vec<UiTransactionTokenBalance>> =
                        transaction_meta.pre_token_balances.into();

                    for pre_token_balance in pre_token_balances.unwrap_or_default() {
                        let indx = pre_token_balance.account_index as usize;

                        if indx >= ACCOUNTS_ARRAY_SIZE {
                            return Err(ParseInstructionError::InvalidIndex {
                                site: "pre_token_balance".to_string(),
                                index: indx,
                                max_len: ACCOUNTS_ARRAY_SIZE,
                            });
                        }

                        pre_token_balance_mint[indx] = Some(pre_token_balance.mint.clone());
                        pre_token_balance_owner[indx] = pre_token_balance.owner.clone().into();
                        pre_token_balance_amount[indx] =
                            pre_token_balance.ui_token_amount.ui_amount;
                        pre_token_balance_program_id[indx] =
                            pre_token_balance.program_id.clone().into();
                    }

                    let post_token_balances: Option<Vec<UiTransactionTokenBalance>> =
                        transaction_meta.post_token_balances.into();

                    for post_token_balance in post_token_balances.unwrap_or_default() {
                        let indx = post_token_balance.account_index as usize;

                        if indx >= ACCOUNTS_ARRAY_SIZE {
                            return Err(ParseInstructionError::InvalidIndex {
                                site: "post_token_balance".to_string(),
                                index: indx,
                                max_len: ACCOUNTS_ARRAY_SIZE,
                            });
                        }

                        post_token_balance_mint[indx] = Some(post_token_balance.mint.clone());
                        post_token_balance_owner[indx] = post_token_balance.owner.clone().into();
                        post_token_balance_amount[indx] =
                            post_token_balance.ui_token_amount.ui_amount;
                        post_token_balance_program_id[indx] =
                            post_token_balance.program_id.clone().into();
                    }

                    accounts.iter().enumerate().for_each(|(i, account)| {
                        pre_balances_map.insert(account.clone(), pre_balances[i].unwrap());
                        balances.push(Balance {
                            tx_signature: tx_signature.clone(),
                            account: account.clone(),
                            pre_balance: pre_balances[i],
                            post_balance: post_balances[i],
                            pre_token_balance_mint: pre_token_balance_mint[i].clone(),
                            pre_token_balance_owner: pre_token_balance_owner[i].clone(),
                            pre_token_balance_amount: pre_token_balance_amount[i],
                            pre_token_balance_program_id: pre_token_balance_program_id[i].clone(),
                            post_token_balance_mint: post_token_balance_mint[i].clone(),
                            post_token_balance_owner: post_token_balance_owner[i].clone(),
                            post_token_balance_amount: post_token_balance_amount[i],
                            post_token_balance_program_id: post_token_balance_program_id[i].clone(),
                        });
                    });
                }

                //////////////////////////Instructions////////////////////////////////////////////

                Self::append_instructions(
                    instructions,
                    inner_instructions.into(),
                    accounts,
                    tx_signature.clone(),
                    slot,
                    block_time as u64,
                    tx_status,
                    &mut instructions_set,
                    &mut parsed_instruction_arguments,
                )?;
            } else {
                return Err(ParseInstructionError::Unsupported(
                    "UiMessage::Raw in message".to_string(),
                ));
            }
        } else {
            return Err(ParseInstructionError::Unsupported(
                "EncodedTransaction::Json in message".to_string(),
            ));
        }

        let instructions: Vec<Instruction> = instructions_set.into_iter().collect();

        Ok((instructions, balances, parsed_instruction_arguments))
    }

    pub fn parse_instruction(
        program_address: &str,
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        debug!("{}", program_address);
        let (instruction_raw, instruction_arguments) = match program_address {
            "packFeFNZzMfD9aVWL7QbGz1WcU7R9zpf6pvNsw2BLu" => {
                TransactionParser::parse_nft_packs_instruction(data)
            }
            "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s" => {
                TransactionParser::parse_token_metadata_instruction(data)
            }
            "vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn" => {
                TransactionParser::parse_token_vault_instruction(data)
            }
            "p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98" => {
                TransactionParser::parse_metaplex_instruction(data)
            }
            "auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8" => {
                TransactionParser::parse_auction_instruction(data)
            }
            "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk" => {
                let res = TransactionParser::parse_auction_house_instruction(data);
                if res.is_err() {
                    // log::error!("AAAA: {:#?}", res);
                    // todo!();
                }

                res
            }
            "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ" => {
                TransactionParser::parse_candy_machine_instraction(data)
            }
            "SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo" => {
                TransactionParser::parse_fixed_price_sale_instruction(data)
            }
            "gdrpGjVffourzkdDRrQmySw4aTHr8a3xmQzzxSwFD1a" => {
                TransactionParser::parse_gumdrop_instruction(data)
            }
            "qntmGodpGkrM42mN68VCZHXnKqDCT8rdY23wFcXCLPd" => {
                TransactionParser::parse_tokent_entangler_instruction(data)
            }
            "Stake11111111111111111111111111111111111111" => {
                TransactionParser::parse_stake_instruction(data)
            }
            "Vote111111111111111111111111111111111111111" => {
                TransactionParser::parse_vote_instruction(data)
            }
            "11111111111111111111111111111111" => TransactionParser::parse_system_instruction(data),

            _ => Err(ParseInstructionError::ProgramAddressMatchError),
        }?;

        Ok((instruction_raw, instruction_arguments))
    }

    fn parse_tokent_entangler_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let sighash: [u8; 8] = (&data[..8]).try_into()?;
        let data = &data[8..];
        TokenEntanglerInstruction::parse_instruction(sighash, data)
    }

    fn parse_gumdrop_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let sighash: [u8; 8] = (&data[..8]).try_into()?;
        let data = &data[8..];
        GumdropInstruction::parse_instruction(sighash, data)
    }

    fn parse_fixed_price_sale_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let sighash: [u8; 8] = (&data[..8]).try_into()?;
        let data = &data[8..];
        FixedPriceSaleInstruction::parse_instruction(sighash, data)
    }

    fn parse_candy_machine_instraction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let sighash: [u8; 8] = (&data[..8]).try_into()?;
        let data = &data[8..];
        CandyMachineInstruction::parse_instruction(sighash, data)
    }

    fn parse_auction_house_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let sighash: [u8; 8] = (&data[..8]).try_into()?;
        let data = &data[8..];
        AuctionHouseInstruction::parse_instruction(sighash, data)
    }

    fn parse_nft_packs_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = NFTPacksInstruction::try_from_slice(data);

        let instruction = match instruction {
            Err(err) => {
                return Err(ParseInstructionError::DeserializeInInstructionError {
                    instruction: "Nft Packs".to_string(),
                    err,
                })
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }

    fn parse_token_metadata_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = MetadataInstruction::try_from_slice(data);

        let instruction = match instruction {
            Err(err) => {
                let err = Err(ParseInstructionError::DeserializeInInstructionError {
                    instruction: "Token Metadata".to_string(),
                    err,
                });

                return err;
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }

    fn parse_token_vault_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = VaultInstruction::try_from_slice(data);

        let instruction = match instruction {
            Err(err) => {
                return Err(ParseInstructionError::DeserializeInInstructionError {
                    instruction: "Token Vault".to_string(),
                    err,
                })
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }

    fn parse_metaplex_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = MetaplexInstruction::try_from_slice(data);

        let instruction = match instruction {
            Err(err) => {
                return Err(ParseInstructionError::DeserializeInInstructionError {
                    instruction: "Metaplex".to_string(),
                    err,
                })
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }

    fn parse_auction_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = AuctionInstruction::try_from_slice(data);

        let instruction = match instruction {
            Err(err) => {
                return Err(ParseInstructionError::DeserializeInInstructionError {
                    instruction: "Auction".to_string(),
                    err,
                })
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }

    fn parse_vote_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = limited_deserialize::<VoteInstruction>(data);

        let instruction = match instruction {
            Err(err) => {
                return Err(ParseInstructionError::LimDeserializeInInstructionError {
                    instruction: "Vote instruction".to_string(),
                    err,
                })
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }

    fn parse_stake_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = limited_deserialize::<StakeInstruction>(data);

        let instruction = match instruction {
            Err(err) => {
                return Err(ParseInstructionError::LimDeserializeInInstructionError {
                    instruction: "Stake instruction".to_string(),
                    err,
                })
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }

    fn parse_system_instruction(
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = limited_deserialize::<SystemInstruction>(data);

        let instruction = match instruction {
            Err(err) => {
                return Err(ParseInstructionError::LimDeserializeInInstructionError {
                    instruction: "SystemInstruction".to_string(),
                    err,
                })
            }
            Ok(val) => val,
        };

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }
}
