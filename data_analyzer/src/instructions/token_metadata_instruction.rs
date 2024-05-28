#![allow(deprecated)]

use std::collections::HashMap;

use crate::storages::main_storage::{instr_args_parse, InstructionArgument, PathTree};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
///////////////////////////token-metadata/program/src/state.rs///////////////////////////////////////
///

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum VerificationArgs {
    CreatorV1,
    CollectionV1,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum PrintArgs {
    V1 { edition: u64 },
    V2 { edition: u64 },
}

#[repr(C)]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy,
)]
#[instr_args_parse]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct Data {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct DataV2 {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct Uses {
    // 17 bytes + Option byte
    pub use_method: UseMethod, //1
    pub remaining: u64,        //8
    pub total: u64,            //8
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct Reservation {
    pub address: Pubkey,
    pub spots_remaining: u64,
    pub total_spots: u64,
}

/////////////////////////////token-metadata/program/src/deprecated_instruction.rs////////////////////////////////////////

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct SetReservationListArgs {
    /// If set, means that no more than this number of editions can ever be minted. This is immutable.
    pub reservations: Vec<Reservation>,
    /// should only be present on the very first call to set reservation list.
    pub total_reservation_spots: Option<u64>,
    /// Where in the reservation list you want to insert this slice of reservations
    pub offset: u64,
    /// What the total spot offset is in the reservation list from the beginning to your slice of reservations.
    /// So if is going to be 4 total editions eventually reserved between your slice and the beginning of the array,
    /// split between 2 reservation entries, the offset variable above would be "2" since you start at entry 2 in 0 indexed array
    /// (first 2 taking 0 and 1) and because they each have 2 spots taken, this variable would be 4.
    pub total_spot_offset: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct MintPrintingTokensViaTokenArgs {
    pub supply: u64,
}

/////////////////////////////token-metadata/program/src/instruction.rs////////////////////////////////////////

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
/// Args for update call
pub struct UpdateMetadataAccountArgs {
    pub data: Option<Data>,
    pub update_authority: Option<Pubkey>,
    pub primary_sale_happened: Option<bool>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
/// Args for update call
pub struct UpdateMetadataAccountArgsV2 {
    pub data: Option<DataV2>,
    pub update_authority: Option<Pubkey>,
    pub primary_sale_happened: Option<bool>,
    pub is_mutable: Option<bool>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
/// Args for create call
pub struct CreateMetadataAccountArgs {
    /// Note that unique metadatas are disabled for now.
    pub data: Data,
    /// Whether you want your metadata to be updateable in the future.
    pub is_mutable: bool,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
/// Args for create call
pub struct CreateMetadataAccountArgsV2 {
    /// Note that unique metadatas are disabled for now.
    pub data: DataV2,
    /// Whether you want your metadata to be updateable in the future.
    pub is_mutable: bool,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct CreateMasterEditionArgs {
    /// If set, means that no more than this number of editions can ever be minted. This is immutable.
    pub max_supply: Option<u64>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct MintNewEditionFromMasterEditionViaTokenArgs {
    pub edition: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct ApproveUseAuthorityArgs {
    pub number_of_uses: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct UtilizeArgs {
    pub number_of_uses: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[instr_args_parse]
/// Args for create call
pub struct CreateMetadataAccountArgsV3 {
    /// Note that unique metadatas are disabled for now.
    pub data: DataV2,
    /// Whether you want your metadata to be updateable in the future.
    pub is_mutable: bool,
    /// If this is a collection parent NFT.
    pub collection_details: Option<CollectionDetails>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[instr_args_parse]
pub enum CollectionDetails {
    V1 { size: u64 },
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[instr_args_parse]
pub struct SetCollectionSizeArgs {
    pub size: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct TransferOutOfEscrowArgs {
    pub amount: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum BurnArgs {
    V1 {
        /// The amount of the token to burn
        amount: u64,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum CreateArgs {
    V1 {
        asset_data: AssetData,
        decimals: Option<u8>,
        print_supply: Option<PrintSupply>,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum PrintSupply {
    /// The asset does not have any prints.
    Zero,
    /// The asset has a limited amount of prints.
    Limited(u64),
    /// The asset has an unlimited amount of prints.
    Unlimited,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct AssetData {
    /// The name of the asset.
    pub name: String,
    /// The symbol for the asset.
    pub symbol: String,
    /// URI pointing to JSON representing the asset.
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000).
    pub seller_fee_basis_points: u16,
    /// Array of creators.
    pub creators: Option<Vec<Creator>>,
    // Immutable, once flipped, all sales of this metadata are considered secondary.
    pub primary_sale_happened: bool,
    // Whether or not the data struct is mutable (default is not).
    pub is_mutable: bool,
    /// Type of the token.
    pub token_standard: TokenStandard,
    /// Collection information.
    pub collection: Option<Collection>,
    /// Uses information.
    pub uses: Option<Uses>,
    /// Collection item details.
    pub collection_details: Option<CollectionDetails>,
    /// Programmable rule set for the asset.
    #[cfg_attr(
        feature = "serde-feature",
        serde(
            deserialize_with = "deser_option_pubkey",
            serialize_with = "ser_option_pubkey"
        )
    )]
    pub rule_set: Option<Pubkey>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum MintArgs {
    V1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum DelegateArgs {
    CollectionV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    SaleV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    TransferV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    DataV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    UtilityV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    StakingV1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    StandardV1 {
        amount: u64,
    },
    LockedTransferV1 {
        amount: u64,
        #[deprecated(
            since = "1.13.2",
            note = "The locked address is deprecated and will soon be removed."
        )]
        /// locked destination pubkey
        locked_address: Pubkey,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    ProgrammableConfigV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AuthorityItemV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    DataItemV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    CollectionItemV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    ProgrammableConfigItemV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    PrintDelegateV1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum RevokeArgs {
    CollectionV1,
    SaleV1,
    TransferV1,
    DataV1,
    UtilityV1,
    StakingV1,
    StandardV1,
    LockedTransferV1,
    ProgrammableConfigV1,
    MigrationV1,
    AuthorityItemV1,
    DataItemV1,
    CollectionItemV1,
    ProgrammableConfigItemV1,
    PrintDelegateV1,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct SeedsVec {
    /// The vector of derivation seeds.
    pub seeds: Vec<Vec<u8>>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct ProofInfo {
    /// The merkle proof.
    pub proof: Vec<[u8; 32]>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum PayloadType {
    /// A plain `Pubkey`.
    Pubkey(Pubkey),
    /// PDA derivation seeds.
    Seeds(SeedsVec),
    /// A merkle proof.
    MerkleProof(ProofInfo),
    /// A plain `u64` used for `Amount`.
    Number(u64),
}

#[repr(C)]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default,
)]
#[instr_args_parse]
pub struct Payload {
    map: HashMap<String, PayloadType>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub struct AuthorizationData {
    pub payload: Payload,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum LockArgs {
    V1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum UnlockArgs {
    V1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum TransferArgs {
    V1 {
        amount: u64,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum UseArgs {
    V1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default,
)]
#[instr_args_parse]
pub enum CollectionToggle {
    #[default]
    None,
    Clear,
    Set(Collection),
}

#[repr(C)]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default,
)]
#[instr_args_parse]
pub enum UsesToggle {
    #[default]
    None,
    Clear,
    Set(Uses),
}

#[repr(C)]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default,
)]
#[instr_args_parse]
pub enum CollectionDetailsToggle {
    #[default]
    None,
    Clear,
    Set(CollectionDetails),
}

#[repr(C)]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default,
)]
#[instr_args_parse]
pub enum RuleSetToggle {
    #[default]
    None,
    Clear,
    Set(Pubkey),
}

#[repr(C)]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy,
)]
#[instr_args_parse]
pub enum TokenStandard {
    NonFungible,                    // This is a master edition
    FungibleAsset,                  // A token with metadata that can also have attributes
    Fungible,                       // A token with simple metadata
    NonFungibleEdition,             // This is a limited edition
    ProgrammableNonFungible,        // NonFungible with programmable configuration
    ProgrammableNonFungibleEdition, // NonFungible with programmable configuration
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum UpdateArgs {
    V1 {
        /// The new update authority.
        new_update_authority: Option<Pubkey>,
        /// The metadata details.
        data: Option<Data>,
        /// Indicates whether the primary sale has happened or not (once set to `true`, it cannot be
        /// changed back).
        primary_sale_happened: Option<bool>,
        // Indicates Whether the data struct is mutable or not (once set to `true`, it cannot be
        /// changed back).
        is_mutable: Option<bool>,
        /// Collection information.
        collection: CollectionToggle,
        /// Additional details of the collection.
        collection_details: CollectionDetailsToggle,
        /// Uses information.
        uses: UsesToggle,
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsUpdateAuthorityV2 {
        /// The new update authority.
        new_update_authority: Option<Pubkey>,
        /// The metadata details.
        data: Option<Data>,
        /// Indicates whether the primary sale has happened or not (once set to `true`, it cannot be
        /// changed back).
        primary_sale_happened: Option<bool>,
        // Indicates Whether the data struct is mutable or not (once set to `true`, it cannot be
        /// changed back).
        is_mutable: Option<bool>,
        /// Collection information.
        collection: CollectionToggle,
        /// Additional details of the collection.
        collection_details: CollectionDetailsToggle,
        /// Uses information.
        uses: UsesToggle,
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Token standard.
        token_standard: Option<TokenStandard>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsAuthorityItemDelegateV2 {
        /// The new update authority.
        #[deprecated(
            since = "1.13.3",
            note = "A delegate cannot change the update authority. This field will be removed in a future release."
        )]
        new_update_authority: Option<Pubkey>,
        /// Indicates whether the primary sale has happened or not (once set to `true`, it cannot be
        /// changed back).
        primary_sale_happened: Option<bool>,
        // Indicates Whether the data struct is mutable or not (once set to `true`, it cannot be
        /// changed back).
        is_mutable: Option<bool>,
        /// Token standard.
        token_standard: Option<TokenStandard>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsCollectionDelegateV2 {
        /// Collection information.
        collection: CollectionToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsDataDelegateV2 {
        /// The metadata details.
        data: Option<Data>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsProgrammableConfigDelegateV2 {
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsDataItemDelegateV2 {
        /// The metadata details.
        data: Option<Data>,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsCollectionItemDelegateV2 {
        /// Collection information.
        collection: CollectionToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
    AsProgrammableConfigItemDelegateV2 {
        // Programmable rule set configuration (only applicable to `Programmable` asset types).
        rule_set: RuleSetToggle,
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
#[instr_args_parse]
pub enum MigrationType {
    CollectionV1,
    ProgrammableV1,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
#[instr_args_parse]
pub enum MigrateArgs {
    V1 {
        migration_type: MigrationType,
        rule_set: Option<Pubkey>,
    },
}

/// Instructions supported by the Metadata program.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[rustfmt::skip]
#[instr_args_parse(InstrRoot)]
pub enum MetadataInstruction {
    CreateMetadataAccount(CreateMetadataAccountArgs),
    UpdateMetadataAccount(UpdateMetadataAccountArgs),
    DeprecatedCreateMasterEdition(CreateMasterEditionArgs),
    DeprecatedMintNewEditionFromMasterEditionViaPrintingToken,
    UpdatePrimarySaleHappenedViaToken,
    DeprecatedSetReservationList(SetReservationListArgs),
    DeprecatedCreateReservationList,
    SignMetadata,
    DeprecatedMintPrintingTokensViaToken(MintPrintingTokensViaTokenArgs),
    DeprecatedMintPrintingTokens(MintPrintingTokensViaTokenArgs),
    CreateMasterEdition(CreateMasterEditionArgs),
    MintNewEditionFromMasterEditionViaToken(MintNewEditionFromMasterEditionViaTokenArgs),
    ConvertMasterEditionV1ToV2,
    MintNewEditionFromMasterEditionViaVaultProxy(MintNewEditionFromMasterEditionViaTokenArgs),
    PuffMetadata,
    UpdateMetadataAccountV2(UpdateMetadataAccountArgsV2),
    CreateMetadataAccountV2(CreateMetadataAccountArgsV2),
    CreateMasterEditionV3(CreateMasterEditionArgs),
    VerifyCollection,
    Utilize(UtilizeArgs),
    ApproveUseAuthority(ApproveUseAuthorityArgs),
    RevokeUseAuthority,
    UnverifyCollection,
    ApproveCollectionAuthority,
    RevokeCollectionAuthority,
    SetAndVerifyCollection,
    FreezeDelegatedAccount,
    ThawDelegatedAccount,
    RemoveCreatorVerification,
    BurnNft,
    VerifySizedCollectionItem,
    UnverifySizedCollectionItem,
    SetAndVerifySizedCollectionItem,
    CreateMetadataAccountV3(CreateMetadataAccountArgsV3),
    SetCollectionSize(SetCollectionSizeArgs),
    SetTokenStandard,
    BubblegumSetCollectionSize(SetCollectionSizeArgs),
    BurnEditionNft,
    CreateEscrowAccount,
    CloseEscrowAccount,
    TransferOutOfEscrow(TransferOutOfEscrowArgs),
    Burn(BurnArgs),
    Create(CreateArgs),
    Mint(MintArgs),
    Delegate(DelegateArgs),
    Revoke(RevokeArgs),
    Lock(LockArgs),
    Unlock(UnlockArgs),
    Migrate(MigrateArgs),
    Transfer(TransferArgs),
    Update(UpdateArgs),
    Use(UseArgs),
    Verify(VerificationArgs),
    Unverify(VerificationArgs),
    Collect,
    Print(PrintArgs),
}

// Instructions supported by the Metadata program.
// #[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
// #[rustfmt::skip]
// #[instr_args_parse(InstrRoot)]
// pub enum MetadataInstruction {
//     CreateMetadataAccount,
//     UpdateMetadataAccount,
//     DeprecatedCreateMasterEdition,
//     DeprecatedMintNewEditionFromMasterEditionViaPrintingToken,
//     UpdatePrimarySaleHappenedViaToken,
//     DeprecatedSetReservationList,
//     DeprecatedCreateReservationList,
//     SignMetadata,
//     DeprecatedMintPrintingTokensViaToken,
//     DeprecatedMintPrintingTokens,
//     CreateMasterEdition,
//     MintNewEditionFromMasterEditionViaToken(MintNewEditionFromMasterEditionViaTokenArgs),
//     ConvertMasterEditionV1ToV2,
//     MintNewEditionFromMasterEditionViaVaultProxy(MintNewEditionFromMasterEditionViaTokenArgs),
//     PuffMetadata,
//     UpdateMetadataAccountV2(UpdateMetadataAccountArgsV2),
//     CreateMetadataAccountV2,
//     CreateMasterEditionV3(CreateMasterEditionArgs),
//     VerifyCollection,
//     Utilize(UtilizeArgs),
//     ApproveUseAuthority(ApproveUseAuthorityArgs),
//     RevokeUseAuthority,
//     UnverifyCollection,
//     ApproveCollectionAuthority,
//     RevokeCollectionAuthority,
//     SetAndVerifyCollection,
//     FreezeDelegatedAccount,
//     ThawDelegatedAccount,
//     RemoveCreatorVerification,
//     BurnNft,
//     VerifySizedCollectionItem,
//     UnverifySizedCollectionItem,
//     SetAndVerifySizedCollectionItem,
//     CreateMetadataAccountV3(CreateMetadataAccountArgsV3),
//     SetCollectionSize(SetCollectionSizeArgs),
//     SetTokenStandard,
//     BubblegumSetCollectionSize(SetCollectionSizeArgs),
//     BurnEditionNft,
//     CreateEscrowAccount,
//     CloseEscrowAccount,
//     TransferOutOfEscrow(TransferOutOfEscrowArgs),
//     Burn(BurnArgs),
//     Create(CreateArgs),
//     Mint(MintArgs),
//     Delegate(DelegateArgs),
//     Revoke(RevokeArgs),
//     Lock(LockArgs),
//     Unlock(UnlockArgs),
//     Migrate,
//     Transfer(TransferArgs),
//     Update(UpdateArgs),
//     Use(UseArgs),
//     Verify(VerificationArgs),
//     Unverify(VerificationArgs),
//     Collect,
//     Print(PrintArgs),
// }
