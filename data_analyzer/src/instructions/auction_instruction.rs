use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::{clock::UnixTimestamp, pubkey::Pubkey};

//////////////////////////////////auction/program/src/processor.rs//////////////////////////
pub type AuctionName = [u8; 32];
type Price = u64;
type Salt = u64;
type Revealer = (Price, Salt);

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub struct CancelBidArgs {
    pub resource: Pubkey,
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub struct CreateAuctionArgs {
    /// How many winners are allowed for this auction. See AuctionData.
    pub winners: WinnerLimit,
    /// End time is the cut-off point that the auction is forced to end by. See AuctionData.
    pub end_auction_at: Option<UnixTimestamp>,
    /// Gap time is how much time after the previous bid where the auction ends. See AuctionData.
    pub end_auction_gap: Option<UnixTimestamp>,
    /// Token mint for the SPL token used for bidding.
    pub token_mint: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// The resource being auctioned. See AuctionData.
    pub resource: Pubkey,
    /// Set a price floor.
    pub price_floor: PriceFloor,
    /// Add a tick size increment
    pub tick_size: Option<u64>,
    /// Add a minimum percentage increase each bid must meet.
    pub gap_tick_size_percentage: Option<u8>,
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub struct ClaimBidArgs {
    pub resource: Pubkey,
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub struct EndAuctionArgs {
    /// The resource being auctioned. See AuctionData.
    pub resource: Pubkey,
    /// If the auction was blinded, a revealing price must be specified to release the auction
    /// winnings.
    pub reveal: Option<Revealer>,
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub struct StartAuctionArgs {
    /// The resource being auctioned. See AuctionData.
    pub resource: Pubkey,
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub struct PlaceBidArgs {
    /// Size of the bid being placed. The user must have enough SOL to satisfy this amount.
    pub amount: u64,
    /// Resource being bid on.
    pub resource: Pubkey,
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse]
pub struct CreateAuctionArgsV2 {
    /// How many winners are allowed for this auction. See AuctionData.
    pub winners: WinnerLimit,
    /// End time is the cut-off point that the auction is forced to end by. See AuctionData.
    pub end_auction_at: Option<UnixTimestamp>,
    /// Gap time is how much time after the previous bid where the auction ends. See AuctionData.
    pub end_auction_gap: Option<UnixTimestamp>,
    /// Token mint for the SPL token used for bidding.
    pub token_mint: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// The resource being auctioned. See AuctionData.
    pub resource: Pubkey,
    /// Set a price floor.
    pub price_floor: PriceFloor,
    /// Add a tick size increment
    pub tick_size: Option<u64>,
    /// Add a minimum percentage increase each bid must meet.
    pub gap_tick_size_percentage: Option<u8>,
    /// Add a instant sale price.
    pub instant_sale_price: Option<u64>,
    /// Auction name
    pub name: Option<AuctionName>,
}

pub const HASH_BYTES: usize = 32;
#[derive(
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Clone,
    Copy,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
)]
#[repr(transparent)]
#[instr_args_parse]
pub struct Hash(pub(crate) [u8; HASH_BYTES]);

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug)]
#[instr_args_parse]
pub enum PriceFloor {
    /// Due to borsh on the front end disallowing different arguments in enums, we have to make sure data is
    /// same size across all three
    /// No price floor, any bid is valid.
    None([u8; 32]),
    /// Explicit minimum price, any bid below this is rejected.
    MinimumPrice([u64; 4]),
    /// Hidden minimum price, revealed at the end of the auction.
    // BlindedPrice(solana_program::hash::Hash),
    BlindedPrice(Hash),
}

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug)]
#[instr_args_parse]
pub enum WinnerLimit {
    Unlimited(usize),
    Capped(usize),
}

//////////////////////////////////auction/program/src/instruction.rs//////////////////////////

#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[instr_args_parse(InstrRoot)]
pub enum AuctionInstruction {
    /// Cancel a bid on a running auction.
    ///   0. `[signer]` The bidders primary account, for PDA calculation/transit auth.
    ///   1. `[writable]` The bidders token account they'll receive refund with
    ///   2. `[writable]` The pot, containing a reference to the stored SPL token account.
    ///   3. `[writable]` The pot SPL account, where the tokens will be deposited.
    ///   4. `[writable]` The metadata account, storing information about the bidders actions.
    ///   5. `[writable]` Auction account, containing data about the auction and item being bid on.
    ///   6. `[writable]` Token mint, for transfer instructions and verification.
    ///   7. `[]` Clock sysvar
    ///   8. `[]` Rent sysvar
    ///   9. `[]` System program
    ///   10. `[]` SPL Token Program
    CancelBid(CancelBidArgs),

    /// Create a new auction account bound to a resource, initially in a pending state.
    ///   0. `[signer]` The account creating the auction, which is authorised to make changes.
    ///   1. `[writable]` Uninitialized auction account.
    ///   2. `[writable]` Auction extended data account (pda relative to auction of ['auction', program id, vault key, 'extended']).
    ///   3. `[]` Rent sysvar
    ///   4. `[]` System account
    CreateAuction(CreateAuctionArgs),

    /// Move SPL tokens from winning bid to the destination account.
    ///   0. `[writable]` The destination account
    ///   1. `[writable]` The bidder pot token account
    ///   2. `[]` The bidder pot pda account [seed of ['auction', program_id, auction key, bidder key]]
    ///   3. `[signer]` The authority on the auction
    ///   4. `[]` The auction
    ///   5. `[]` The bidder wallet
    ///   6. `[]` Token mint of the auction
    ///   7. `[]` Clock sysvar
    ///   8. `[]` Token program
    ///   9. `[]` Auction extended (pda relative to auction of ['auction', program id, vault key, 'extended'])
    ClaimBid(ClaimBidArgs),

    /// Ends an auction, regardless of end timing conditions
    ///
    ///   0. `[writable, signer]` Auction authority
    ///   1. `[writable]` Auction
    ///   6. `[]` Clock sysvar
    EndAuction(EndAuctionArgs),

    /// Start an inactive auction.
    ///   0. `[signer]` The creator/authorised account.
    ///   1. `[writable]` Initialized auction account.
    ///   2. `[]` Clock sysvar
    StartAuction(StartAuctionArgs),

    /// Update the authority for an auction account.
    ///   0. `[writable]` auction (pda of ['auction', program id, resource id])
    ///   1. `[signer]` authority
    ///   2. `[]` newAuthority
    SetAuthority,

    /// Place a bid on a running auction.
    ///   0. `[signer]` The bidders primary account, for PDA calculation/transit auth.
    ///   1. `[writable]` The bidders token account they'll pay with
    ///   2. `[writable]` The pot, containing a reference to the stored SPL token account.
    ///   3. `[writable]` The pot SPL account, where the tokens will be deposited.
    ///   4. `[writable]` The metadata account, storing information about the bidders actions.
    ///   5. `[writable]` Auction account, containing data about the auction and item being bid on.
    ///   6. `[writable]` Token mint, for transfer instructions and verification.
    ///   7. `[signer]` Transfer authority, for moving tokens into the bid pot.
    ///   8. `[signer]` Payer
    ///   9. `[]` Clock sysvar
    ///   10. `[]` Rent sysvar
    ///   11. `[]` System program
    ///   12. `[]` SPL Token Program
    PlaceBid(PlaceBidArgs),

    /// Create a new auction account bound to a resource, initially in a pending state.
    /// The only one difference with above instruction it's additional parameters in CreateAuctionArgsV2
    ///   0. `[signer]` The account creating the auction, which is authorised to make changes.
    ///   1. `[writable]` Uninitialized auction account.
    ///   2. `[writable]` Auction extended data account (pda relative to auction of ['auction', program id, vault key, 'extended']).
    ///   3. `[]` Rent sysvar
    ///   4. `[]` System account
    CreateAuctionV2(CreateAuctionArgsV2),
}
