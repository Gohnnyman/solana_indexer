use crate::errors::ParseInstructionError;
use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse(InstrRoot)]
pub enum TokenEntanglerInstruction {
    CreateEntangledPair {
        bump: u8,
        reverse_bump: u8,
        token_a_escrow_bump: u8,
        token_b_escrow_bump: u8,
        price: u64,
        pays_every_time: bool,
    },
    UpdateEntangledPair {
        price: u64,
        pays_every_time: bool,
    },
    Swap,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct CreateEntangledPair {
    bump: u8,
    reverse_bump: u8,
    token_a_escrow_bump: u8,
    token_b_escrow_bump: u8,
    price: u64,
    pays_every_time: bool,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse]
struct UpdateEntangledPair {
    price: u64,
    pays_every_time: bool,
}

impl TokenEntanglerInstruction {
    pub fn match_sighash(
        sighash: [u8; 8],
        data: &[u8],
    ) -> Result<TokenEntanglerInstruction, ParseInstructionError> {
        match sighash {
            [166, 106, 32, 45, 156, 210, 209, 240] => {
                let create_entangled_pair = CreateEntangledPair::try_from_slice(data)?;
                Ok(TokenEntanglerInstruction::CreateEntangledPair {
                    bump: create_entangled_pair.bump,
                    reverse_bump: create_entangled_pair.reverse_bump,
                    token_a_escrow_bump: create_entangled_pair.token_a_escrow_bump,
                    token_b_escrow_bump: create_entangled_pair.token_b_escrow_bump,
                    price: create_entangled_pair.price,
                    pays_every_time: create_entangled_pair.pays_every_time,
                })
            }
            [41, 97, 247, 218, 98, 162, 75, 244] => {
                let update_entangled_pair = UpdateEntangledPair::try_from_slice(data)?;
                Ok(TokenEntanglerInstruction::UpdateEntangledPair {
                    price: update_entangled_pair.price,
                    pays_every_time: update_entangled_pair.pays_every_time,
                })
            }
            [248, 198, 158, 145, 225, 117, 135, 200] => Ok(TokenEntanglerInstruction::Swap),
            _ => Err(ParseInstructionError::SighashMatchError(
                "Token Entangler".to_string(),
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
                    instruction: "token Entangler".to_string(),
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
