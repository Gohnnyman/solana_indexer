use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

/////////////////////////token-vault/program/src/state.rs/////////////////////////////////////////////
#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub enum Key {
    Uninitialized,
    SafetyDepositBoxV1,
    ExternalAccountKeyV1,
    VaultV1,
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[instr_args_parse]
pub struct ExternalPriceAccount {
    pub key: Key,
    pub price_per_share: u64,
    /// Mint of the currency we are pricing the shares against, should be same as redeem_treasury.
    /// Most likely will be USDC mint most of the time.
    pub price_mint: Pubkey,
    /// Whether or not combination has been allowed for this vault.
    pub allowed_to_combine: bool,
}

/////////////////////////token-vault/program/src/instruction.rs///////////////////////////////////////
#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[instr_args_parse]
pub struct InitVaultArgs {
    pub allow_further_share_creation: bool,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[instr_args_parse]
pub struct AmountArgs {
    pub amount: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[instr_args_parse]
pub struct NumberOfShareArgs {
    pub number_of_shares: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct MintEditionProxyArgs {
    pub edition: u64,
}

/// Instructions supported by the Fraction program.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[rustfmt::skip]
#[instr_args_parse(InstrRoot)]
pub enum VaultInstruction {
    /// Initialize a token vault, starts inactivate. Add tokens in subsequent instructions, then activate.
    InitVault(InitVaultArgs),

    /// Add a token to a inactive token vault
    AddTokenToInactiveVault(AmountArgs),

    /// Activates the vault, distributing initial shares into the fraction treasury.
    /// Tokens can no longer be removed in this state until Combination.
    ActivateVault(NumberOfShareArgs),

    /// This act checks the external pricing oracle for permission to combine and the price of the circulating market cap to do so.
    /// If you can afford it, this amount is charged and placed into the redeem treasury for shareholders to redeem at a later time.
    /// The treasury then unlocks into Combine state and you can remove the tokens.
    CombineVault,

    /// If in the combine state, shareholders can hit this endpoint to burn shares in exchange for monies from the treasury.
    /// Once fractional supply is zero and all tokens have been removed this action will take vault to Deactivated
    RedeemShares,

    /// If in combine state, authority on vault can hit this to withdrawal some of a token type from a safety deposit box.
    /// Once fractional supply is zero and all tokens have been removed this action will take vault to Deactivated
    WithdrawTokenFromSafetyDepositBox(AmountArgs),

    /// Self explanatory - mint more fractional shares if the vault is configured to allow such.
    MintFractionalShares(NumberOfShareArgs),

    /// Withdraws shares from the treasury to a desired account.
    WithdrawSharesFromTreasury(NumberOfShareArgs),

    /// Returns shares to the vault if you wish to remove them from circulation.
    AddSharesToTreasury(NumberOfShareArgs),

    /// Helpful method that isn't necessary to use for main users of the app, but allows one to create/update
    /// existing external price account fields if they are signers of this account.
    /// Useful for testing purposes, and the CLI makes use of it as well so that you can verify logic.
    UpdateExternalPriceAccount(ExternalPriceAccount),

    /// Sets the authority of the vault to a new authority.
    ///
    SetAuthority,
}
