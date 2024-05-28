//! Instruction types
#![allow(missing_docs)]

use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct AddCardToPackArgs {
    /// How many editions of this card will exists in pack
    pub max_supply: u32,
    /// Probability value, required only if PackSet distribution type == Fixed
    pub weight: u16,
    /// Index
    pub index: u32,
}

/// Initialize a PackSet arguments
#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct InitPackSetArgs {
    /// Name
    pub name: [u8; 32],
    /// Description
    pub description: String,
    /// Pack set preview image
    pub uri: String,
    /// If true authority can make changes at deactivated phase
    pub mutable: bool,
    /// Distribution type
    pub distribution_type: PackDistributionType,
    /// Allowed amount to redeem
    pub allowed_amount_to_redeem: u32,
    /// Redeem start date, if not filled set current timestamp
    pub redeem_start_date: Option<u64>,
    /// Redeem end date
    pub redeem_end_date: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, Serialize, Deserialize, BorshSerialize)]
#[instr_args_parse]
pub enum PackDistributionType {
    /// Max supply
    MaxSupply,
    /// Fixed
    Fixed,
    /// Unlimited
    Unlimited,
}

/// Edit a PackSet arguments
#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct EditPackSetArgs {
    /// Name
    pub name: Option<[u8; 32]>,
    /// Description
    pub description: Option<String>,
    /// URI
    pub uri: Option<String>,
    /// If true authority can make changes at deactivated phase
    pub mutable: Option<bool>,
}

/// Claim card from pack
#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct ClaimPackArgs {
    /// Card index
    pub index: u32,
}

/// Request card to redeem arguments
#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct RequestCardToRedeemArgs {
    /// Voucher index
    pub index: u32,
}

/// Instruction definition
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, Debug)]
#[instr_args_parse(InstrRoot)]
pub enum NFTPacksInstruction {
    /// InitPack
    ///
    /// Initialize created account.
    ///
    /// Accounts:
    /// - write                          pack_set
    /// - signer                         authority
    /// - read                           store
    /// - read                           Rent account
    /// - read                           Clock account
    /// - read                           whitelisted_creator. Optional key
    ///
    /// Parameters:
    /// - name	[u8; 32]
    /// - description String
    /// - URI String
    /// - mutable	bool
    /// - distribution_type    DistributionType
    /// - allowed_amount_to_redeem    u32
    /// - redeem_start_date    Option<u64>
    /// - redeem_end_date    Option<u64>
    InitPack(InitPackSetArgs),

    /// AddCardToPack
    ///
    /// Creates new account with PackCard structure and program token account which will hold MasterEdition token.
    /// Also admin points how many items of this specific MasterEdition will be in the pack. Check MasterEdition for V2.
    ///
    /// Accounts:
    /// - read, write                   pack_set
    /// - write                         pack_config (PDA, ['config', pack])
    /// - write                         pack_card (PDA, ['card', pack, index])
    /// - signer                        authority
    /// - read                          master_edition
    /// - read                          master_metadata
    /// - read                          mint
    /// - write                         source
    /// - write                         token_account (program account to hold MasterEdition token)
    /// - read                          program_authority
    /// - read                          store
    /// - read                          rent
    /// - read                          system_program
    /// - read                          spl_token program
    ///
    /// Parameters:
    /// - max_supply	Option<u32>
    /// - probability_type	enum[fixed number, probability based]
    /// - probability	u64
    AddCardToPack(AddCardToPackArgs),

    /// AddVoucherToPack
    ///
    /// Creates new account with PackVoucher structure, saves there data about NFTs which user has to provide to open the pack.
    /// Check MasterEdition for V2.
    ///
    /// Accounts:
    /// - read, write                   pack_set
    /// - write                         pack_voucher (PDA, ['voucher', pack, index])
    /// - signer, write                 authority
    /// - signer, read                  voucher_owner
    /// - read                          master_edition
    /// - read                          master_metadata
    /// - read                          mint
    /// - write                         source
    /// - read                          store
    /// - read                          rent
    /// - read                          system_program
    /// - read                          spl_token program
    AddVoucherToPack,

    /// Activate
    ///
    /// Pack authority call this instruction to activate pack, means close for changing.
    ///
    /// Accounts:
    /// - write            pack_set
    /// - signer           authority
    Activate,

    /// Deactivate
    ///
    /// Forbid users prove vouchers ownership and claiming.
    ///
    /// Accounts:
    /// - write            pack_set
    /// - signer           authority
    Deactivate,

    /// Close the pack
    ///
    /// Set pack state to "ended", irreversible operation
    ///
    /// Accounts:
    /// - write            pack_set
    /// - signer           authority
    /// - read             clock
    ClosePack,

    /// ClaimPack
    ///
    /// Call this instruction with ProvingProcess and PackCard accounts and program will transfer
    /// MasterEdition to user account or return empty response depends successfully or not user open pack with specific MasterEdition.
    ///
    /// Accounts:
    /// - read              pack_set
    /// - read, write       proving_process (PDA, ['proving', pack, user_wallet])
    /// - signer            user_wallet
    /// - read, write       pack_card (PDA, ['card', pack, index])
    /// - write             user_token_acc (user token account ot hold new minted edition)
    /// - read              new_metadata_acc
    /// - read              new_edition_acc
    /// - read              master_edition_acc
    /// - read              new_mint_account
    /// - signer            new_mint_authority_acc
    /// - read              metadata_acc
    /// - read              metadata_mint_acc
    /// - read              edition_acc
    /// - read              rent program
    /// - read              mpl_token_metadata program
    /// - read              spl_token program
    /// - read              system program
    ///
    /// Parameters:
    /// - index             u32
    ClaimPack(ClaimPackArgs),

    /// TransferPackAuthority
    ///
    /// Change pack authority.
    ///
    /// Accounts:
    /// - write            pack_set
    /// - signer           current_authority
    /// - read             new_authority
    TransferPackAuthority,

    /// DeletePack
    ///
    /// Transfer all the SOL from pack set account to refunder account and thus remove it.
    ///
    /// Accounts:
    /// - write            pack_set
    /// - signer           authority
    /// - write            refunder
    DeletePack,

    /// DeletePackCard
    ///
    /// Transfer all the SOL from pack card account to refunder account and thus remove it.
    /// Also transfer master token to new owner.
    ///
    /// Accounts:
    /// - write            pack_set
    /// - write            pack_card
    /// - signer           authority
    /// - write            refunder
    /// - write            new_master_edition_owner
    /// - write            token_account
    /// - read             program_authority
    /// - read             rent
    /// - read             spl_token program
    DeletePackCard,

    /// DeletePackVoucher
    ///
    /// Transfer all the SOL from pack voucher account to refunder account and thus remove it.
    ///
    /// Accounts:
    /// - write            pack_set
    /// - write            pack_voucher
    /// - signer           authority
    /// - write            refunder
    DeletePackVoucher,

    /// EditPack
    ///
    /// Edit pack data.
    ///
    /// Accounts:
    /// - write            pack_set
    /// - signer           authority
    ///
    /// Parameters:
    /// - name Option<[u8; 32]>
    /// - description Option<String>
    /// - URI Option<String>
    /// - mutable	Option<bool> (only can be changed from true to false)
    EditPack(EditPackSetArgs),

    /// RequestCardForRedeem
    ///
    /// Count card index which user can redeem next
    ///
    /// Accounts:
    /// - read                     pack_set
    /// - read, write              pack_config (PDA, ['config', pack])
    /// - read                     store
    /// - read                     edition
    /// - read                     edition_mint
    /// - read                     pack_voucher
    /// - read, write              proving_process (PDA, ['proving', pack, user_wallet])
    /// - signer                   user_wallet
    /// - read                     recent_slothashes
    /// - read                     clock
    /// - read                     rent
    /// - read                     system_program
    /// - read                     user_token_account optional
    ///
    /// Parameters:
    /// - index    u32
    RequestCardForRedeem(RequestCardToRedeemArgs),

    /// CleanUp
    ///
    /// Sorts weights of all the cards and removes exhausted
    ///
    /// Accounts:
    /// - read                     pack_set
    /// - read, write              pack_config (PDA, ['config', pack])
    CleanUp,

    /// Delete PackConfig account
    ///
    /// Transfer all the SOL from pack card account to refunder account and thus remove it.
    ///
    /// Accounts:
    /// - read                pack_set
    /// - write               pack_config (PDA, ['config', pack])
    /// - write               refunder
    /// - signer              authority
    DeletePackConfig,
}
