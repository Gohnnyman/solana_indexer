use crate::errors::ParseInstructionError;
use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
#[instr_args_parse(InstrRoot)]
pub enum GumdropInstruction {
    NewDistributor {
        _bump: u8,
        root: [u8; 32],
        temporal: Pubkey,
    },
    CloseDistributorTokenAccount {
        _bump: u8,
    },
    CloseDistributor {
        _bump: u8,
        _wallet_bump: u8,
    },
    ProveClaim {
        claim_prefix: Vec<u8>,
        claim_bump: u8,
        index: u64,
        amount: u64,
        claimant_secret: Pubkey,
        resource: Pubkey,
        resource_nonce: Vec<u8>,
        proof: Vec<[u8; 32]>,
    },
    Claim {
        bump: u8,
        index: u64,
        amount: u64,
        claimant_secret: Pubkey,
        proof: Vec<[u8; 32]>,
    },
    ClaimCandy {
        wallet_bump: u8,
        claim_bump: u8,
        index: u64,
        amount: u64,
        claimant_secret: Pubkey,
        proof: Vec<[u8; 32]>,
    },
    ClaimEdition {
        claim_bump: u8,
        index: u64,
        amount: u64,
        edition: u64,
        claimant_secret: Pubkey,
        proof: Vec<[u8; 32]>,
    },
    ClaimCandyProven {
        wallet_bump: u8,
        _claim_bump: u8,
        _index: u64,
    },
    RecoverUpdateAuthority {
        _bump: u8,
        wallet_bump: u8,
    },
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct NewDistributor {
    _bump: u8,
    root: [u8; 32],
    temporal: Pubkey,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct CloseDistributorTokenAccount {
    _bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct CloseDistributor {
    _bump: u8,
    _wallet_bump: u8,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct ProveClaim {
    claim_prefix: Vec<u8>,
    claim_bump: u8,
    index: u64,
    amount: u64,
    claimant_secret: Pubkey,
    resource: Pubkey,
    resource_nonce: Vec<u8>,
    proof: Vec<[u8; 32]>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct Claim {
    bump: u8,
    index: u64,
    amount: u64,
    claimant_secret: Pubkey,
    proof: Vec<[u8; 32]>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct ClaimCandy {
    wallet_bump: u8,
    claim_bump: u8,
    index: u64,
    amount: u64,
    claimant_secret: Pubkey,
    proof: Vec<[u8; 32]>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct ClaimEdition {
    claim_bump: u8,
    index: u64,
    amount: u64,
    edition: u64,
    claimant_secret: Pubkey,
    proof: Vec<[u8; 32]>,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct ClaimCandyProven {
    wallet_bump: u8,
    _claim_bump: u8,
    _index: u64,
}

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
struct RecoverUpdateAuthority {
    _bump: u8,
    wallet_bump: u8,
}

impl GumdropInstruction {
    pub fn match_sighash(
        sighash: [u8; 8],
        data: &[u8],
    ) -> Result<GumdropInstruction, ParseInstructionError> {
        match sighash {
            [32, 139, 112, 171, 0, 2, 225, 155] => {
                let new_distributor = NewDistributor::try_from_slice(data)?;
                Ok(GumdropInstruction::NewDistributor {
                    _bump: new_distributor._bump,
                    root: new_distributor.root,
                    temporal: new_distributor.temporal,
                })
            }
            [156, 174, 153, 120, 102, 150, 134, 142] => {
                let close_distributor_token_account =
                    CloseDistributorTokenAccount::try_from_slice(data)?;
                Ok(GumdropInstruction::CloseDistributorTokenAccount {
                    _bump: close_distributor_token_account._bump,
                })
            }
            [202, 56, 180, 143, 46, 104, 106, 112] => {
                let close_distributor = CloseDistributor::try_from_slice(data)?;
                Ok(GumdropInstruction::CloseDistributor {
                    _bump: close_distributor._bump,
                    _wallet_bump: close_distributor._wallet_bump,
                })
            }
            [52, 82, 123, 224, 40, 139, 230, 184] => {
                let prove_claim = ProveClaim::try_from_slice(data)?;
                Ok(GumdropInstruction::ProveClaim {
                    claim_prefix: prove_claim.claim_prefix,
                    claim_bump: prove_claim.claim_bump,
                    index: prove_claim.index,
                    amount: prove_claim.amount,
                    claimant_secret: prove_claim.claimant_secret,
                    resource: prove_claim.resource,
                    resource_nonce: prove_claim.resource_nonce,
                    proof: prove_claim.proof,
                })
            }
            [62, 198, 214, 193, 213, 159, 108, 210] => {
                let claim = Claim::try_from_slice(data)?;
                Ok(GumdropInstruction::Claim {
                    bump: claim.bump,
                    index: claim.index,
                    amount: claim.amount,
                    claimant_secret: claim.claimant_secret,
                    proof: claim.proof,
                })
            }
            [87, 176, 177, 90, 136, 95, 83, 242] => {
                let claim_candy = ClaimCandy::try_from_slice(data)?;
                Ok(GumdropInstruction::ClaimCandy {
                    wallet_bump: claim_candy.wallet_bump,
                    claim_bump: claim_candy.claim_bump,
                    index: claim_candy.index,
                    amount: claim_candy.amount,
                    claimant_secret: claim_candy.claimant_secret,
                    proof: claim_candy.proof,
                })
            }
            [150, 83, 124, 180, 53, 35, 144, 248] => {
                let claim_edition = ClaimEdition::try_from_slice(data)?;
                Ok(GumdropInstruction::ClaimEdition {
                    claim_bump: claim_edition.claim_bump,
                    index: claim_edition.index,
                    amount: claim_edition.amount,
                    edition: claim_edition.edition,
                    claimant_secret: claim_edition.claimant_secret,
                    proof: claim_edition.proof,
                })
            }
            [1, 2, 30, 252, 145, 228, 67, 145] => {
                let claim_candy_prove = ClaimCandyProven::try_from_slice(data)?;
                Ok(GumdropInstruction::ClaimCandyProven {
                    wallet_bump: claim_candy_prove.wallet_bump,
                    _claim_bump: claim_candy_prove._claim_bump,
                    _index: claim_candy_prove._index,
                })
            }
            [142, 251, 209, 116, 87, 100, 36, 191] => {
                let recover_update_authority = RecoverUpdateAuthority::try_from_slice(data)?;
                Ok(GumdropInstruction::RecoverUpdateAuthority {
                    _bump: recover_update_authority._bump,
                    wallet_bump: recover_update_authority.wallet_bump,
                })
            }
            _ => Err(ParseInstructionError::SighashMatchError(
                "Gumdrop".to_string(),
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
                    instruction: "Gumdrop".to_string(),
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
