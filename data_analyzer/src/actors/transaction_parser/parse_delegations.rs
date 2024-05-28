use crate::actors::queue_manager::QueueManagerHandle;
use crate::errors::{ConvertingError, ParseInstructionError};
use crate::storages::main_storage::{
    Delegation, Instruction, InstructionArgument, TxStatus, ACCOUNTS_ARRAY_SIZE,
};

use anyhow::Result;
use rust_base58::FromBase58;
use solana_transaction_status::{UiCompiledInstruction, UiInnerInstructions, UiInstruction};
use std::collections::{BTreeSet, HashMap};
use std::convert::TryInto;

use super::{Delegations, TransactionParser, Undelegations, STAKE_ACC_RENT_EXEMPTION};

const FIRST_ACCOUNTS: usize = 2;

impl TransactionParser {
    pub async fn parse_delegations(
        mut queue_manager: QueueManagerHandle,
        instructions: Vec<Instruction>,
        pre_balances: HashMap<String, u64>,
    ) -> Result<(Delegations, Undelegations)> {
        let mut previous_balance: HashMap<String, u64> = HashMap::new();
        let mut delegations = Delegations::new();
        let mut undelegations = Undelegations::new();
        let instructions_accounts = instructions
            .iter()
            .flat_map(|instruction| instruction.accounts.clone())
            .enumerate()
            // We are taking only first 2 accounts because only they are used in staking instructions
            .filter(|(i, account)| account.is_some() && *i < FIRST_ACCOUNTS)
            .map(|(_, account)| account.unwrap())
            .collect();

        let mut vote_accounts: HashMap<String, Option<String>> = queue_manager
            .get_delegations(instructions_accounts)
            .await??
            .into_iter()
            .map(|d| (d.stake_acc, d.vote_acc))
            .collect();

        let instruction_names = vec![
            "Withdraw",
            "Merge",
            "Split",
            "Deactivate",
            "DelegateStake",
            "CreateAccount",
            "CreateAccountWithSeed",
            "Transfer",
        ];
        for instruction in instructions {
            if !instruction_names.contains(&instruction.instruction_name.as_str())
                || instruction.program != "Stake11111111111111111111111111111111111111"
            {
                continue;
            }

            let raw_instruction_idx = instruction.get_raw_instruction_idx();
            let instruction_name = instruction.instruction_name;
            let tx_signature = instruction.tx_signature.clone();
            let account_0 = instruction.accounts[0].clone().unwrap();
            let account_1 = instruction.accounts[1].clone().unwrap();
            let data = instruction.data;
            let slot = instruction.slot;
            let block_time = instruction.block_time;

            previous_balance
                .entry(account_0.clone())
                .or_insert(pre_balances.get(&account_0).cloned().unwrap_or(0));

            previous_balance
                .entry(account_1.clone())
                .or_insert(pre_balances.get(&account_1).cloned().unwrap_or(0));

            match instruction_name.as_str() {
                "DelegateStake" => {
                    delegations.push(Delegation {
                        slot,
                        block_time,
                        stake_acc: account_0.clone(),
                        vote_acc: Some(account_1.clone()),
                        tx_signature,
                        amount: previous_balance[&account_0]
                            .saturating_sub(STAKE_ACC_RENT_EXEMPTION),
                        raw_instruction_idx,
                    });
                    vote_accounts.insert(account_0.clone(), Some(account_1.clone()));
                }
                "Deactivate" => {
                    undelegations.push(Delegation {
                        slot,
                        block_time,
                        stake_acc: account_0.clone(),
                        vote_acc: vote_accounts
                            .get(&account_0.clone())
                            .cloned()
                            .unwrap_or_default(),
                        tx_signature,
                        amount: previous_balance[&account_0]
                            .saturating_sub(STAKE_ACC_RENT_EXEMPTION),
                        raw_instruction_idx,
                    });
                    vote_accounts.insert(account_0.clone(), None);
                }
                "CreateAccountWithSeed" => {
                    *previous_balance.get_mut(&account_1).unwrap() +=
                        serde_json::from_str::<serde_json::Value>(&data).unwrap()
                            ["CreateAccountWithSeed"]["lamports"]
                            .as_u64()
                            .unwrap();
                }
                "Withdraw" => {
                    *previous_balance.get_mut(&account_0).unwrap() -=
                        serde_json::from_str::<serde_json::Value>(&data).unwrap()["Withdraw"]
                            .as_u64()
                            .unwrap();

                    *previous_balance.get_mut(&account_1).unwrap() +=
                        serde_json::from_str::<serde_json::Value>(&data).unwrap()["Withdraw"]
                            .as_u64()
                            .unwrap();
                }
                "Transfer" => {
                    *previous_balance.get_mut(&account_0).unwrap() -=
                        serde_json::from_str::<serde_json::Value>(&data).unwrap()["Transfer"]
                            ["lamports"]
                            .as_u64()
                            .unwrap();

                    *previous_balance.get_mut(&account_1).unwrap() +=
                        serde_json::from_str::<serde_json::Value>(&data).unwrap()["Transfer"]
                            ["lamports"]
                            .as_u64()
                            .unwrap();
                }
                "CreateAccount" => {
                    *previous_balance.get_mut(&account_1).unwrap() +=
                        serde_json::from_str::<serde_json::Value>(&data).unwrap()["CreateAccount"]
                            ["lamports"]
                            .as_u64()
                            .unwrap();
                }
                "Split" => {
                    let amount = serde_json::from_str::<serde_json::Value>(&data).unwrap()["Split"]
                        .as_u64()
                        .unwrap();

                    let vote_acc = vote_accounts.get(&account_0).cloned().unwrap_or_default();

                    undelegations.push(Delegation {
                        slot,
                        block_time,
                        stake_acc: account_0.clone(),
                        vote_acc: vote_acc.clone(),
                        tx_signature: tx_signature.clone(),
                        amount,
                        raw_instruction_idx,
                    });

                    delegations.push(Delegation {
                        slot,
                        block_time,
                        stake_acc: account_1.clone(),
                        vote_acc: vote_acc.clone(),
                        tx_signature,
                        amount: amount.saturating_sub(STAKE_ACC_RENT_EXEMPTION),
                        raw_instruction_idx,
                    });

                    vote_accounts.insert(account_1.clone(), vote_acc);

                    *previous_balance.get_mut(&account_0).unwrap() = previous_balance
                        .get(&account_0)
                        .unwrap()
                        .saturating_sub(amount);
                    *previous_balance.get_mut(&account_1).unwrap() += amount;

                    if *previous_balance.get_mut(&account_0).unwrap() < STAKE_ACC_RENT_EXEMPTION {
                        vote_accounts.insert(account_0.clone(), None);
                    }
                }
                "Merge" => {
                    let vote_acc = vote_accounts.get(&account_0).cloned().unwrap_or_default();

                    delegations.push(Delegation {
                        slot,
                        block_time,
                        stake_acc: account_0.clone(),
                        vote_acc: vote_acc.clone(),
                        tx_signature: tx_signature.clone(),
                        amount: previous_balance[&account_1]
                            .saturating_sub(STAKE_ACC_RENT_EXEMPTION),
                        raw_instruction_idx,
                    });

                    undelegations.push(Delegation {
                        slot,
                        block_time,
                        stake_acc: account_1.clone(),
                        vote_acc,
                        tx_signature,
                        amount: previous_balance[&account_1]
                            .saturating_sub(STAKE_ACC_RENT_EXEMPTION),
                        raw_instruction_idx,
                    });

                    vote_accounts.insert(account_0.clone(), None);
                    *previous_balance.get_mut(&account_0).unwrap() += previous_balance[&account_1];
                    *previous_balance.get_mut(&account_1).unwrap() = 0;

                    vote_accounts.remove(&account_1);
                }
                _ => unreachable!(),
            }
        }

        queue_manager
            .save_delegations(vote_accounts.into_iter().collect())
            .await?;

        Ok((delegations, undelegations))
    }

    pub fn append_instructions(
        instructions: Vec<UiCompiledInstruction>,
        inner_instructions: Option<Vec<UiInnerInstructions>>,
        accounts: Vec<String>,
        tx_signature: String,
        slot: u64,
        block_time: u64,
        tx_status: TxStatus,
        instructions_set: &mut BTreeSet<Instruction>,
        parsed_instruction_arguments: &mut Vec<InstructionArgument>,
    ) -> Result<(), ParseInstructionError> {
        Self::append_outer_instruction(
            instructions,
            accounts.clone(),
            tx_signature.clone(),
            slot,
            block_time,
            tx_status,
            instructions_set,
            parsed_instruction_arguments,
        )?;

        Self::append_inner_instruction(
            inner_instructions,
            accounts.clone(),
            tx_signature.clone(),
            slot,
            block_time,
            tx_status,
            instructions_set,
            parsed_instruction_arguments,
        )?;

        Ok(())
    }

    fn append_inner_instruction(
        inner_instructions: Option<Vec<UiInnerInstructions>>,
        accounts: Vec<String>,
        tx_signature: String,
        slot: u64,
        block_time: u64,
        tx_status: TxStatus,
        instructions_set: &mut BTreeSet<Instruction>,
        parsed_instruction_arguments: &mut Vec<InstructionArgument>,
    ) -> Result<(), ParseInstructionError> {
        if let Some(inner_instructions) = inner_instructions {
            for (inner_instructions_set, instruction) in inner_instructions.iter().enumerate() {
                let index = instruction.index;
                for (instruction_idx, instruction) in instruction.instructions.iter().enumerate() {
                    if let UiInstruction::Compiled(instruction) = instruction {
                        let inner_program_address =
                            accounts.get(instruction.program_id_index as usize);
                        if inner_program_address.is_none() {
                            return Err(ParseInstructionError::ParseError(
                                "Failed to get inner_program_address".to_string(),
                            ));
                        }
                        let inner_program_address = inner_program_address.unwrap();

                        let mut inner_instruction_accounts = Vec::new();

                        for account_idx in instruction.accounts.iter() {
                            let inner_instruction_account = accounts.get(*account_idx as usize);
                            if let Some(inner_instruction_account) = inner_instruction_account {
                                inner_instruction_accounts
                                    .push(Some(inner_instruction_account.to_owned()));
                            } else {
                                return Err(ParseInstructionError::InvalidIndex {
                                    site: "inner_instruction".to_string(),
                                    index: *account_idx as usize,
                                    max_len: accounts.len(),
                                });
                            };
                        }

                        inner_instruction_accounts.resize(ACCOUNTS_ARRAY_SIZE, Default::default());

                        let parsed_data = TransactionParser::parse_instruction(
                            inner_program_address,
                            &instruction.data.from_base58()?,
                        );

                        let mut parsed_data =
                            if let Err(ParseInstructionError::ProgramAddressMatchError) =
                                parsed_data
                            {
                                (instruction.data.clone(), Vec::new())
                            } else {
                                parsed_data?
                            };

                        let data_cloned = parsed_data.0.clone();
                        let splitted_data = data_cloned.split('\"').collect::<Vec<&str>>();

                        let instruction_name = if splitted_data.len() > 2 {
                            splitted_data[1].to_string()
                        } else if splitted_data.len() == 1 {
                            // splitted_data.len() == 1 means that parsed_data.0 is Base58 text (ProgramAddressMatchError occured)
                            std::default::Default::default()
                        } else {
                            return Err(ParseInstructionError::InvalidInstructionName);
                        };

                        let accounts: Result<[Option<String>; ACCOUNTS_ARRAY_SIZE], _> =
                            inner_instruction_accounts.try_into();

                        if accounts.is_err() {
                            Err(ConvertingError::DifferentLengths)?;
                        }
                        let accounts = accounts.unwrap();

                        let instr = Instruction {
                            program: inner_program_address.clone(),
                            tx_signature: tx_signature.clone(),
                            slot,
                            block_time: block_time as u64,
                            tx_status,
                            instruction_idx: instruction_idx as u8,
                            inner_instructions_set: Some(inner_instructions_set as u8),
                            transaction_instruction_idx: Some(index),
                            accounts,
                            instruction_name,
                            data: parsed_data.0,
                        };

                        instructions_set.insert(instr);

                        for instruction_argument in parsed_data.1.iter_mut() {
                            instruction_argument.tx_signature = tx_signature.clone();
                            instruction_argument.instruction_idx = instruction_idx as u8;
                            instruction_argument.inner_instructions_set =
                                Some(inner_instructions_set as u8);
                            instruction_argument.program = inner_program_address.clone();
                        }

                        parsed_instruction_arguments.append(&mut parsed_data.1);
                    } else {
                        return Err(ParseInstructionError::Unsupported(
                            "UiInstruction::Compiled in Inner instruction".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn append_outer_instruction(
        instructions: Vec<UiCompiledInstruction>,
        accounts: Vec<String>,
        tx_signature: String,
        slot: u64,
        block_time: u64,
        tx_status: TxStatus,
        instructions_set: &mut BTreeSet<Instruction>,
        parsed_instruction_arguments: &mut Vec<InstructionArgument>,
    ) -> Result<(), ParseInstructionError> {
        for (instruction_idx, instruction) in instructions.iter().enumerate() {
            let program_address = accounts.get(instruction.program_id_index as usize);

            if program_address.is_none() {
                return Err(ParseInstructionError::ParseError(
                    "Failed to get program_address".to_string(),
                ));
            }
            let program_address = program_address.unwrap();

            let mut instruction_accounts = Vec::new();

            for account_idx in instruction.accounts.iter() {
                let instruction_account = accounts.get(*account_idx as usize);
                if let Some(instruction_account) = instruction_account {
                    instruction_accounts.push(Some(instruction_account.to_owned()));
                } else {
                    return Err(ParseInstructionError::InvalidIndex {
                        site: "instruction".to_string(),
                        index: *account_idx as usize,
                        max_len: accounts.len(),
                    });
                };
            }

            instruction_accounts.resize_with(ACCOUNTS_ARRAY_SIZE, Default::default);

            // if program_address == "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk" {
            //     log::error!("DATA: {:?}, tx: {}", instruction.data, tx_signature)
            // }
            let parsed_data = TransactionParser::parse_instruction(
                program_address,
                &instruction.data.from_base58()?,
            );

            let mut parsed_data =
                if let Err(ParseInstructionError::ProgramAddressMatchError) = parsed_data {
                    (instruction.data.clone(), Vec::new())
                } else {
                    parsed_data?
                };

            let data_cloned = parsed_data.0.clone();
            let splitted_data = data_cloned.split('\"').collect::<Vec<&str>>();

            let instruction_name = if splitted_data.len() > 2 {
                splitted_data[1].to_string()
            } else if splitted_data.len() == 1 {
                // splitted_data.len() == 1 means that parsed_data.0 is Base58 text (ProgramAddressMatchError occured)
                std::default::Default::default()
            } else {
                return Err(ParseInstructionError::InvalidInstructionName);
            };

            let accounts: Result<[Option<String>; ACCOUNTS_ARRAY_SIZE], _> =
                instruction_accounts.try_into();

            if accounts.is_err() {
                Err(ConvertingError::DifferentLengths)?;
            }
            let accounts = accounts.unwrap();

            let instr = Instruction {
                program: program_address.clone(),
                tx_signature: tx_signature.clone(),
                slot,
                block_time,
                tx_status,
                instruction_idx: instruction_idx as u8,
                inner_instructions_set: None,
                transaction_instruction_idx: None,
                accounts,
                instruction_name,
                data: parsed_data.0,
            };

            instructions_set.insert(instr);

            for instruction_argument in parsed_data.1.iter_mut() {
                instruction_argument.tx_signature = tx_signature.clone();
                instruction_argument.instruction_idx = instruction_idx as u8;
                instruction_argument.inner_instructions_set = None;
                instruction_argument.program = program_address.clone();
            }

            parsed_instruction_arguments.append(&mut parsed_data.1);
        }

        Ok(())
    }
}
