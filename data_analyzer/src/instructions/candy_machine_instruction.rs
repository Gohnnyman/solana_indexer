use crate::errors::ParseInstructionError;
use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub enum WhitelistMintMode {
    BurnEveryTime,
    NeverBurn,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub enum EndSettingType {
    Date,
    Amount,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct EndSettings {
    end_setting_type: EndSettingType,
    number: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct WhitelistMintSettings {
    mode: WhitelistMintMode,
    mint: Pubkey,
    presale: bool,
    discount_price: Option<u64>,
}

/// Hidden Settings for large mints used with offline data.
#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct HiddenSettings {
    name: String,
    uri: String,
    hash: [u8; 32],
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct Creator {
    address: Pubkey,
    verified: bool,
    share: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct GatekeeperConfig {
    /// The network for the gateway token required
    gatekeeper_network: Pubkey,
    /// Whether or not the token should expire after minting.
    /// The gatekeeper network must support this if true.
    expire_on_use: bool,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct CandyMachineData {
    uuid: String,
    price: u64,
    /// The symbol for the asset
    symbol: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    seller_fee_basis_points: u16,
    max_supply: u64,
    is_mutable: bool,
    retain_authority: bool,
    go_live_date: Option<i64>,
    end_settings: Option<EndSettings>,
    creators: Vec<Creator>,
    hidden_settings: Option<HiddenSettings>,
    whitelist_mint_settings: Option<WhitelistMintSettings>,
    items_available: u64,
    /// If [`Some`] requires gateway tokens on mint
    gatekeeper: Option<GatekeeperConfig>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
pub struct ConfigLine {
    name: String,
    /// URI pointing to JSON representing the asset
    uri: String,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse(InstrRoot)]
pub enum CandyMachineInstruction {
    MintNft {
        creator_bump: u8,
    },
    UpdateCandyMachine {
        data: CandyMachineData,
    },
    AddConfigLines {
        index: u32,
        config_lines: Vec<ConfigLine>,
    },
    InitializeCandyMachine {
        data: CandyMachineData,
    },
    UpdateAuthority {
        new_authority: Option<Pubkey>,
    },
    SetCollectionDuringMint,
    SetCollection,
    RemoveCollection,
    WithdrawFunds,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct MintNft {
    creator_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct UpdateCandyMachine {
    data: CandyMachineData,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct AddConfigLines {
    index: u32,
    config_lines: Vec<ConfigLine>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct InitializeCandyMachine {
    data: CandyMachineData,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct UpdateAuthority {
    new_authority: Option<Pubkey>,
}

impl CandyMachineInstruction {
    pub fn match_sighash(
        sighash: [u8; 8],
        data: &[u8],
    ) -> Result<CandyMachineInstruction, ParseInstructionError> {
        match sighash {
            [211, 57, 6, 167, 15, 219, 35, 251] => {
                let mint_nft = MintNft::try_from_slice(data)?;
                Ok(CandyMachineInstruction::MintNft {
                    creator_bump: mint_nft.creator_bump,
                })
            }
            [103, 17, 200, 25, 118, 95, 125, 61] => {
                Ok(CandyMachineInstruction::SetCollectionDuringMint)
            }
            [243, 251, 124, 156, 211, 211, 118, 239] => {
                let update_candy_machine = UpdateCandyMachine::try_from_slice(data)?;
                Ok(CandyMachineInstruction::UpdateCandyMachine {
                    data: update_candy_machine.data,
                })
            }
            [223, 50, 224, 227, 151, 8, 115, 106] => {
                let add_config_lines = AddConfigLines::try_from_slice(data)?;
                Ok(CandyMachineInstruction::AddConfigLines {
                    index: add_config_lines.index,
                    config_lines: add_config_lines.config_lines,
                })
            }
            [142, 137, 167, 107, 47, 39, 240, 124] => {
                let initialize_candy_machine = InitializeCandyMachine::try_from_slice(data)?;
                Ok(CandyMachineInstruction::InitializeCandyMachine {
                    data: initialize_candy_machine.data,
                })
            }
            [192, 254, 206, 76, 168, 182, 59, 223] => Ok(CandyMachineInstruction::SetCollection),
            [223, 52, 106, 217, 61, 220, 36, 160] => Ok(CandyMachineInstruction::RemoveCollection),
            [32, 46, 64, 28, 149, 75, 243, 88] => {
                let update_authority = UpdateAuthority::try_from_slice(data)?;
                Ok(CandyMachineInstruction::UpdateAuthority {
                    new_authority: update_authority.new_authority,
                })
            }
            [241, 36, 29, 111, 208, 31, 104, 217] => Ok(CandyMachineInstruction::WithdrawFunds),
            _ => Err(ParseInstructionError::SighashMatchError(
                "Candy Machine".to_string(),
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
                    instruction: "Candy Machine".to_string(),
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
