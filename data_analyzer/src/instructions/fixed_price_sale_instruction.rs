use crate::errors::ParseInstructionError;
use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use chrono::format::Fixed;
use serde::Serialize;
use solana_program::pubkey::Pubkey;
use solana_sdk::address_lookup_table::instruction;

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse(InstrRoot)]
pub enum FixedPriceSaleInstruction {
    InitSellingResource {
        master_edition_bump: u8,
        vault_owner_bump: u8,
        max_supply: Option<u64>,
    },
    CreateStore {
        name: String,
        description: String,
    },
    Buy {
        trade_history_bump: u8,
        vault_owner_bump: u8,
    },
    CloseMarket,
    SuspendMarket,
    ChangeMarket {
        new_name: Option<String>,
        new_description: Option<String>,
        mutable: Option<bool>,
        new_price: Option<u64>,
        new_pieces_in_one_wallet: Option<u64>,
    },
    ResumeMarket,
    Withdraw {
        treasury_owner_bump: u8,
        payout_ticket_bump: u8,
    },
    CreateMarket {
        treasury_owner_bump: u8,
        name: String,
        description: String,
        mutable: bool,
        price: u64,
        pieces_in_one_wallet: Option<u64>,
        start_date: u64,
        end_date: Option<u64>,
        gating_config: Option<GatingConfig>,
    },
    ClaimResource {
        vault_owner_bump: u8,
    },
    SavePrimaryMetadataCreators {
        primary_metadata_creators_bump: u8,
        creators: Vec<Creator>,
    },
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct InitSellingResource {
    master_edition_bump: u8,
    vault_owner_bump: u8,
    max_supply: Option<u64>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct CreateStore {
    name: String,
    description: String,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct Buy {
    trade_history_bump: u8,
    vault_owner_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct ChangeMarket {
    new_name: Option<String>,
    new_description: Option<String>,
    mutable: Option<bool>,
    new_price: Option<u64>,
    new_pieces_in_one_wallet: Option<u64>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct Withdraw {
    treasury_owner_bump: u8,
    payout_ticket_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct CreateMarket {
    treasury_owner_bump: u8,
    name: String,
    description: String,
    mutable: bool,
    price: u64,
    pieces_in_one_wallet: Option<u64>,
    start_date: u64,
    end_date: Option<u64>,
    gating_config: Option<GatingConfig>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct GatingConfig {
    pub collection: Pubkey,
    /// whether program will burn token or just check availability
    pub expire_on_use: bool,
    pub gating_time: Option<u64>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct ClaimResource {
    vault_owner_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct SavePrimaryMetadataCreators {
    primary_metadata_creators_bump: u8,
    creators: Vec<Creator>,
}

// mpl_token_metadata::state::Creator
#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

impl FixedPriceSaleInstruction {
    pub fn match_sighash(
        sighash: [u8; 8],
        data: &[u8],
    ) -> Result<FixedPriceSaleInstruction, ParseInstructionError> {
        match sighash {
            [56, 15, 222, 211, 147, 205, 4, 145] => {
                let init_selling_resource = InitSellingResource::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::InitSellingResource {
                    master_edition_bump: init_selling_resource.master_edition_bump,
                    vault_owner_bump: init_selling_resource.vault_owner_bump,
                    max_supply: init_selling_resource.max_supply,
                })
            }
            [132, 152, 9, 27, 112, 19, 95, 83] => {
                let create_store = CreateStore::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::CreateStore {
                    name: create_store.name,
                    description: create_store.description,
                })
            }
            [102, 6, 61, 18, 1, 218, 235, 234] => {
                let buy = Buy::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::Buy {
                    trade_history_bump: buy.trade_history_bump,
                    vault_owner_bump: buy.vault_owner_bump,
                })
            }
            [88, 154, 248, 186, 48, 14, 123, 244] => Ok(FixedPriceSaleInstruction::CloseMarket),
            [246, 27, 129, 46, 10, 196, 165, 118] => Ok(FixedPriceSaleInstruction::SuspendMarket),
            [130, 59, 109, 101, 85, 226, 37, 88] => {
                let change_market = ChangeMarket::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::ChangeMarket {
                    new_name: change_market.new_name,
                    new_description: change_market.new_description,
                    mutable: change_market.mutable,
                    new_price: change_market.new_price,
                    new_pieces_in_one_wallet: change_market.new_pieces_in_one_wallet,
                })
            }
            [198, 120, 104, 87, 44, 103, 108, 143] => Ok(FixedPriceSaleInstruction::ResumeMarket),
            [183, 18, 70, 156, 148, 109, 161, 34] => {
                let withdraw = Withdraw::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::Withdraw {
                    treasury_owner_bump: withdraw.treasury_owner_bump,
                    payout_ticket_bump: withdraw.payout_ticket_bump,
                })
            }
            [103, 226, 97, 235, 200, 188, 251, 254] => {
                let create_market = CreateMarket::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::CreateMarket {
                    treasury_owner_bump: create_market.treasury_owner_bump,
                    name: create_market.name,
                    description: create_market.description,
                    mutable: create_market.mutable,
                    price: create_market.price,
                    pieces_in_one_wallet: create_market.pieces_in_one_wallet,
                    start_date: create_market.start_date,
                    end_date: create_market.end_date,
                    gating_config: create_market.gating_config,
                })
            }
            [0, 160, 164, 96, 237, 118, 74, 27] => {
                let claim_resource = ClaimResource::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::ClaimResource {
                    vault_owner_bump: claim_resource.vault_owner_bump,
                })
            }
            [66, 240, 213, 46, 185, 60, 192, 254] => {
                let save_primary_metadata_creators =
                    SavePrimaryMetadataCreators::try_from_slice(data)?;
                Ok(FixedPriceSaleInstruction::SavePrimaryMetadataCreators {
                    primary_metadata_creators_bump: save_primary_metadata_creators
                        .primary_metadata_creators_bump,
                    creators: save_primary_metadata_creators.creators,
                })
            }
            _ => Err(ParseInstructionError::SighashMatchError(
                "Fixed Price Sale".to_string(),
            )),
        }
    }

    pub fn parse_instruction(
        sighash: [u8; 8],
        data: &[u8],
    ) -> Result<(String, Vec<InstructionArgument>), ParseInstructionError> {
        let instruction = Self::match_sighash(sighash, data);

        let instruction = match instruction {
            Err(ParseInstructionError::DeserializeError(err)) => {
                return Err(ParseInstructionError::DeserializeInInstructionError {
                    instruction: "Fixed Price Sale".to_string(),
                    err,
                });
            }
            _ => instruction,
        }?;

        let json = serde_json::to_string(&instruction)?;

        let instruction_arguments = instruction.get_arguments("", 0, None, "");

        Ok((json, instruction_arguments))
    }
}
