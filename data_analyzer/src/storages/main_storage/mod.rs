use crate::errors::{ConvertingError, ParseInstructionError};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use clickhouse::Row;
use serde_repr::{Deserialize_repr, Serialize_repr};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

pub use macros::{implement_path_tree, instr_args_parse};
use serde::Serialize;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction};

pub mod https_client;
pub mod migrations;
pub mod tcp_client;

pub const ACCOUNTS_ARRAY_SIZE: usize = 256;

#[allow(unused)]
use std::str::FromStr;
use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum TxStatus {
    Failed = 0,
    Success = 1,
    Undefined = 2,
}

impl From<TxStatus> for i8 {
    fn from(tx_status: TxStatus) -> Self {
        match tx_status {
            TxStatus::Failed => 0,
            TxStatus::Success => 1,
            TxStatus::Undefined => 2,
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Instruction {
    pub program: String,
    pub tx_signature: String,
    pub tx_status: TxStatus,
    pub slot: u64,
    pub block_time: u64,
    pub instruction_idx: u8,
    pub inner_instructions_set: Option<u8>,
    pub transaction_instruction_idx: Option<u8>,
    pub instruction_name: String,
    pub accounts: [Option<String>; ACCOUNTS_ARRAY_SIZE],
    pub data: String,
}

impl Instruction {
    pub fn get_raw_instruction_idx(&self) -> u16 {
        let transaction_instruction_idx = self.transaction_instruction_idx.map(|x| x as u16);
        let instruction_idx = self.instruction_idx as u16;

        if transaction_instruction_idx.is_none() {
            instruction_idx * 256 as u16
        } else {
            (transaction_instruction_idx.unwrap() * 256 + instruction_idx) + 1
        }
    }
}

impl Ord for Instruction {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.slot.cmp(&other.slot);

        if ord != Ordering::Equal {
            return ord;
        }

        let raw_instruction_idx1 = self.get_raw_instruction_idx();
        let raw_instruction_idx2 = other.get_raw_instruction_idx();

        raw_instruction_idx1.cmp(&raw_instruction_idx2)
    }
}

impl PartialOrd for Instruction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

#[allow(unused)]
impl Instruction {
    pub fn new(program: &Pubkey, tx_signature: &Signature) -> Self {
        Self {
            program: program.to_string(),
            tx_signature: tx_signature.to_string(),
            tx_status: TxStatus::Undefined,
            slot: 0,
            block_time: 0,
            instruction_idx: 0,
            inner_instructions_set: None,
            transaction_instruction_idx: None,
            instruction_name: String::from(""),
            accounts: [0; ACCOUNTS_ARRAY_SIZE]
                .iter()
                .map(|_| -> Option<String> { None })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(), // Will never fail because of the same size
            data: String::from(""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErroneousTransaction {
    pub slot: u64,
    pub transaction: String,
    pub tx_signature: String,
    pub cause: String,
}

impl ErroneousTransaction {
    pub fn try_from_transactions_with_error(
        enc_conf_transaction: EncodedConfirmedTransactionWithStatusMeta,
        error: ParseInstructionError,
    ) -> Result<Self, ConvertingError> {
        let slot = enc_conf_transaction.slot;
        let signature = if let EncodedTransaction::Json(ref transaction) =
            enc_conf_transaction.transaction.transaction
        {
            let sig = transaction.signatures.first();
            if sig.is_none() {
                return Err(ConvertingError::EmptyField("signature".to_string()));
            }

            sig.unwrap().clone()
        } else {
            return Err(ConvertingError::Unsupported(
                "Not EncodedTransaction::Json transaction".to_string(),
            ));
        };

        let transaction = serde_json::to_string(&enc_conf_transaction)?;
        let cause = error.to_string();

        Ok(Self {
            slot,
            transaction,
            tx_signature: signature,
            cause,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub tx_signature: String,
    pub account: String,
    pub pre_balance: Option<u64>,
    pub post_balance: Option<u64>,
    pub pre_token_balance_mint: Option<String>,
    pub pre_token_balance_owner: Option<String>,
    pub pre_token_balance_amount: Option<f64>,
    pub pre_token_balance_program_id: Option<String>,
    pub post_token_balance_mint: Option<String>,
    pub post_token_balance_owner: Option<String>,
    pub post_token_balance_amount: Option<f64>,
    pub post_token_balance_program_id: Option<String>,
}

#[derive(Serialize, Default, Debug, Clone, PartialEq, Row)]
pub struct Delegation {
    pub slot: u64,
    pub block_time: u64,
    pub stake_acc: String,
    pub vote_acc: Option<String>,
    pub tx_signature: String,
    pub amount: u64,
    pub raw_instruction_idx: u16,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct InstructionArgument {
    pub tx_signature: String,
    pub instruction_idx: u8,
    pub inner_instructions_set: Option<u8>,
    pub program: String,
    pub arg_idx: u16,
    pub arg_path: String,
    pub int_value: Option<i64>,
    pub unsigned_value: Option<u64>,
    pub float_value: Option<f64>,
    pub string_value: Option<String>,
}

impl InstructionArgument {
    pub fn new(
        tx_signature: &str,
        instruction_idx: u8,
        inner_instructions_set: Option<u8>,
        program: &str,
    ) -> Self {
        Self {
            tx_signature: tx_signature.to_string(),
            instruction_idx,
            inner_instructions_set,
            program: program.to_string(),
            ..Default::default()
        }
    }
}

/// PathTree represents a tree of paths to arguments for some instruction.
/// We can iterate through the tree and get vector if InstructionArgument objects.
#[implement_path_tree(Array(2, 3, 4, 8, 32), Tuple(2))]
pub enum PathTree {
    String(String),
    Int(i64),
    Unsigned(u64),
    Float(f64),
    Path(Vec<(String, Box<PathTree>)>),
    None,
}

impl<T: Into<PathTree> + Clone> From<HashMap<String, T>> for PathTree {
    fn from(hash_map: HashMap<String, T>) -> Self {
        let mut path_vec = Vec::new();
        hash_map.into_iter().for_each(|(key, val)| {
            path_vec.push((key, Box::new(val.clone().into())));
        });

        Self::Path(path_vec)
    }
}

impl PathTree {
    /// Returns a vector of InstructionArgument objects.
    pub fn get_instruction_args_vec(
        self,
        instruction_arguments: &mut Vec<InstructionArgument>,
        default_instruction_argument: InstructionArgument,
        arg_idx: &mut u16,
    ) {
        match self {
            Self::String(string_value) => {
                instruction_arguments.push(InstructionArgument {
                    string_value: Some(string_value),
                    arg_idx: *arg_idx,
                    ..default_instruction_argument
                });
                *arg_idx += 1;
            }
            Self::Int(int_value) => {
                instruction_arguments.push(InstructionArgument {
                    int_value: Some(int_value),
                    arg_idx: *arg_idx,
                    ..default_instruction_argument
                });
                *arg_idx += 1;
            }
            Self::Unsigned(unsigned_value) => {
                instruction_arguments.push(InstructionArgument {
                    unsigned_value: Some(unsigned_value),
                    arg_idx: *arg_idx,
                    ..default_instruction_argument
                });
                *arg_idx += 1;
            }
            Self::Float(float_value) => {
                instruction_arguments.push(InstructionArgument {
                    float_value: Some(float_value),
                    arg_idx: *arg_idx,
                    ..default_instruction_argument
                });
                *arg_idx += 1;
            }
            Self::None => {
                instruction_arguments.push(InstructionArgument {
                    arg_idx: *arg_idx,
                    ..default_instruction_argument
                });
                *arg_idx += 1;
            }
            Self::Path(path) => {
                path.into_iter().for_each(|(field_name, path_tree)| {
                    let mut mock = default_instruction_argument.clone();

                    // This if statement is to avoid adding '/' to the end of the path, but for to the beginning.
                    if !field_name.is_empty() || *arg_idx == 0 {
                        mock.arg_path = format!("{}/{}", mock.arg_path, field_name);
                    }

                    path_tree.get_instruction_args_vec(instruction_arguments, mock, arg_idx);
                });
            }
        };
    }
}

// From<..> implementation of basic types for PathTree
impl<T> From<&std::option::Option<T>> for PathTree
where
    T: Into<PathTree> + Clone,
{
    fn from(opt: &std::option::Option<T>) -> Self {
        if let Some(val) = opt {
            val.clone().into()
        } else {
            Self::None
        }
    }
}

impl<T> From<std::option::Option<T>> for PathTree
where
    T: Into<PathTree>,
{
    fn from(opt: std::option::Option<T>) -> Self {
        if let Some(val) = opt {
            val.into()
        } else {
            Self::None
        }
    }
}

impl<T> From<&[T]> for PathTree
where
    T: Into<PathTree> + Clone,
{
    fn from(slice: &[T]) -> Self {
        let mut path_vec = Vec::new();
        slice.iter().enumerate().for_each(|(i, val)| {
            path_vec.push((i.to_string(), Box::new(val.clone().into())));
        });

        Self::Path(path_vec)
    }
}

impl From<solana_program::hash::Hash> for PathTree {
    fn from(hash: solana_program::hash::Hash) -> Self {
        hash.as_ref().into()
    }
}

impl<T> From<Vec<T>> for PathTree
where
    T: Into<PathTree>,
{
    fn from(mut vec: Vec<T>) -> Self {
        let mut path_vec = Vec::new();
        vec.drain(..).into_iter().enumerate().for_each(|(i, val)| {
            path_vec.push((i.to_string(), Box::new(val.into())));
        });

        Self::Path(path_vec)
    }
}

impl<T> From<VecDeque<T>> for PathTree
where
    T: Into<PathTree>,
{
    fn from(mut vec: VecDeque<T>) -> Self {
        let mut path_vec = Vec::new();
        vec.drain(..).into_iter().enumerate().for_each(|(i, val)| {
            path_vec.push((i.to_string(), Box::new(val.into())));
        });

        Self::Path(path_vec)
    }
}

impl From<&str> for PathTree {
    fn from(string: &str) -> Self {
        PathTree::String(string.to_string())
    }
}

impl From<String> for PathTree {
    fn from(string: String) -> Self {
        PathTree::String(string)
    }
}

impl From<Pubkey> for PathTree {
    fn from(pubkey: Pubkey) -> Self {
        PathTree::String(pubkey.to_string())
    }
}

impl From<i64> for PathTree {
    fn from(int: i64) -> Self {
        PathTree::Int(int)
    }
}

impl From<i32> for PathTree {
    fn from(int: i32) -> Self {
        PathTree::Int(int.into())
    }
}

impl From<i16> for PathTree {
    fn from(int: i16) -> Self {
        PathTree::Int(int.into())
    }
}

impl From<u64> for PathTree {
    fn from(unsigned: u64) -> Self {
        PathTree::Unsigned(unsigned)
    }
}

impl From<u32> for PathTree {
    fn from(unsigned: u32) -> Self {
        PathTree::Unsigned(unsigned.into())
    }
}

impl From<u16> for PathTree {
    fn from(unsigned: u16) -> Self {
        PathTree::Unsigned(unsigned.into())
    }
}

impl From<u8> for PathTree {
    fn from(unsigned: u8) -> Self {
        PathTree::Unsigned(unsigned.into())
    }
}

impl From<usize> for PathTree {
    fn from(usz: usize) -> Self {
        PathTree::Unsigned(usz.try_into().unwrap())
    }
}

impl From<f64> for PathTree {
    fn from(float: f64) -> Self {
        PathTree::Float(float)
    }
}

impl From<f32> for PathTree {
    fn from(float: f32) -> Self {
        PathTree::Float(float.into())
    }
}

impl From<bool> for PathTree {
    fn from(bl: bool) -> Self {
        PathTree::Int(i64::from(bl))
    }
}

#[async_trait]
pub trait MainStorage: Send {
    async fn execute(&mut self, ddl: &str) -> Result<()>;
    async fn migration_exists(&mut self, version: &str) -> Result<bool>;
    async fn store_instructions_block(&mut self, instructions: Vec<Instruction>) -> Result<()>;
    async fn store_instruction_arguments_block(
        &mut self,
        instruction_arguments: Vec<InstructionArgument>,
    ) -> Result<()>;
    async fn store_balances_block(&mut self, balances: Vec<Balance>) -> Result<()>;
    async fn store_erroneous_transaction_block(
        &mut self,
        erroneous_transactions: Vec<ErroneousTransaction>,
    ) -> Result<()>;
    async fn store_delegations_block(&mut self, delegations: Vec<Delegation>) -> Result<()>;
    async fn store_undelegations_block(&mut self, undelegations: Vec<Delegation>) -> Result<()>;
}

pub async fn connect_main_storage(database_url: &str) -> Result<Box<dyn MainStorage>> {
    let dsn = dsn::parse(database_url)?;

    if dsn.driver == *"https" || dsn.driver == *"http" {
        return Ok(Box::new(https_client::HttpsClient::new(dsn).await?));
    }
    if dsn.driver == *"tcp" {
        return Ok(Box::new(tcp_client::TcpClient::new(dsn).await?));
    }

    Err(anyhow!("Unknown protocol"))
}

#[cfg(test)]
mod clickhouse_server_tests {
    use super::*;

    #[tokio::test]
    async fn store_instructions_block() -> Result<()> {
        let ddl = r"CREATE TABLE IF NOT EXISTS instructions
        (
            program String,
            tx_signature String,
            tx_status Enum('Failed' = 0, 'Success' = 1),
            slot UInt64,
            block_time UInt64,
            instruction_idx UInt8,
            inner_instructions_set Nullable(UInt8),
            transaction_instruction_idx Nullable(UInt8),
            instruction_name String,
            account_0 Nullable(String),
            account_1 Nullable(String),
            account_2 Nullable(String),
            account_3 Nullable(String),
            account_4 Nullable(String),
            account_5 Nullable(String),
            account_6 Nullable(String),
            account_7 Nullable(String),
            account_8 Nullable(String),
            account_9 Nullable(String),
            account_10 Nullable(String),
            account_11 Nullable(String),
            account_12 Nullable(String),
            account_13 Nullable(String),
            account_14 Nullable(String),
            account_15 Nullable(String),
            account_16 Nullable(String),
            account_17 Nullable(String),
            account_18 Nullable(String),
            account_19 Nullable(String),
            account_20 Nullable(String),
            account_21 Nullable(String),
            account_22 Nullable(String),
            account_23 Nullable(String),
            account_24 Nullable(String),
            account_25 Nullable(String),
            account_26 Nullable(String),
            account_27 Nullable(String),
            account_28 Nullable(String),
            account_29 Nullable(String),
            account_30 Nullable(String),
            account_31 Nullable(String),
            account_32 Nullable(String),
            account_33 Nullable(String),
            account_34 Nullable(String),
            data String
        ) ENGINE = Memory;";

        let dsn = dsn::parse("tcp://@tcp(badaddr:9000)")?;

        let mut main_storage = tcp_client::TcpClient::new(dsn).await?;
        let c = main_storage.get_handle();
        c.execute(ddl).await?;

        let mut instructions = Vec::new();

        for _i in 0..10000 {
            let pkey = Pubkey::from_str("SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo").unwrap();

            let signature = Signature::from_str("3o3WMi2xfsyt9GhJt1z8XbcauANLFtpLbgH9wvpwQDFiQ3H2MLyMtXVHrZi3wX5UXZEENnAFUFnTLu7G8ybjiR4x").unwrap();
            let instruction = Instruction::new(&pkey, &signature);
            instructions.push(instruction);
        }

        main_storage.store_instructions_block(instructions).await?;

        main_storage
            .get_handle()
            .execute("DROP TABLE IF EXISTS instructions")
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_create_table() -> Result<()> {
        let ddl = r"
                CREATE TABLE clickhouse_test_create_table (
                click_id   FixedString(64),
                click_time DateTime
                ) Engine=Memory";

        let dsn = dsn::parse("tcp://@tcp(badaddr:9000)")?;

        let mut main_storage = tcp_client::TcpClient::new(dsn).await?;
        let c = main_storage.get_handle();

        c.execute("DROP TABLE IF EXISTS clickhouse_test_create_table")
            .await?;
        c.execute(ddl).await?;

        c.execute("DROP TABLE IF EXISTS clickhouse_test_create_table")
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_ping() -> Result<()> {
        let dsn = dsn::parse("tcp://@tcp(badaddr:9000)")?;

        let mut main_storage = tcp_client::TcpClient::new(dsn).await?;
        main_storage.ping().await?;
        Ok(())
    }
}

#[cfg(test)]
mod clickhouse_tests {
    use super::*;
    use clickhouse_rs::Pool;

    #[tokio::test]
    async fn tx_status_variants() {
        let cases = [
            (0, TxStatus::Failed),
            (1, TxStatus::Success),
            (2, TxStatus::Undefined),
        ];

        for (i, status) in cases {
            let status: i8 = status.into();
            assert_eq!(i as i8, status);
        }
    }

    #[tokio::test]
    async fn test_new_instruction() {
        let pkey = Pubkey::from_str("SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo").unwrap();
        let signature = Signature::from_str(
            "3o3WMi2xfsyt9GhJt1z8XbcauANLFtpLbgH9wvpwQDFiQ3H2MLyMtXVHrZi3wX5UXZEENnAFUFnTLu7G8ybjiR4x",
        )
        .unwrap();

        let instruction = Instruction::new(&pkey, &signature);

        assert_eq!(pkey.to_string(), instruction.program);
        assert_eq!(signature.to_string(), instruction.tx_signature);
        assert_eq!(TxStatus::Undefined, instruction.tx_status);

        for account in instruction.accounts {
            assert_eq!(None, account);
        }

        assert_eq!("", instruction.data);
    }

    #[tokio::test]
    async fn test_connection_by_wrong_address() -> Result<()> {
        let pool = Pool::new("tcp://@tcp(badaddr:9000)");
        let ret: Result<()> = async move {
            let mut c = pool.get_handle().await?;
            c.ping().await?;
            Ok(())
        }
        .await;

        ret.unwrap_err();
        Ok(())
    }
}

#[cfg(test)]
mod inst_args_parser_tests {
    use super::*;
    use macros::instr_args_parse;

    #[derive(Debug, PartialEq)]
    #[instr_args_parse]
    pub enum EnumTest {
        Variant1,
        Variant2(f32),
        Variant3 { field1: i32, field2: Option<String> },
    }

    #[derive(Debug, PartialEq, Eq)]
    #[instr_args_parse]
    pub struct NestedPubkeyTest {
        pubkey: Pubkey,
    }

    #[derive(Debug, PartialEq, Eq)]
    #[instr_args_parse]
    pub struct NestedTest {
        field1: Option<Option<u64>>,
        field2: NestedPubkeyTest,
    }

    #[derive(Debug, PartialEq, Eq)]
    #[instr_args_parse]
    pub struct ArrayTest {
        array: [i32; 3],
        tuple: Option<(i32, String)>,
    }

    #[derive(Debug, PartialEq, Eq)]
    #[instr_args_parse]
    pub struct TestUnnamed(i32, [i32; 2]);

    #[derive(Debug, PartialEq, Eq)]
    #[instr_args_parse]
    pub struct TestUnit;

    #[derive(Debug, PartialEq)]
    #[instr_args_parse]
    pub struct Test {
        field1: u64,
        field2: std::option::Option<String>,
        field3: Option<NestedTest>,
        field4: TestUnnamed,
        field5: TestUnit,
        field6: EnumTest,
        field7: ArrayTest,
    }

    #[derive(Debug, PartialEq)]
    #[instr_args_parse(InstrRoot)]
    enum RootInstr {
        BoolVariant(bool),
        EnumVariant(EnumTest, EnumTest),
    }

    #[tokio::test]
    async fn test_root_instr() {
        let _test1 = RootInstr::EnumVariant(
            EnumTest::Variant2(1.1),
            EnumTest::Variant3 {
                field1: 2,
                field2: None,
            },
        );

        let test1 = RootInstr::EnumVariant(
            EnumTest::Variant2(1.1),
            EnumTest::Variant3 {
                field1: 2,
                field2: None,
            },
        );

        assert_eq!(
            test1.get_arguments("123", 0, None, "program"),
            vec![
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 0,
                    arg_path: "/0/variant_2".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 1,
                    arg_path: "/0/variant_2/0".to_string(),
                    float_value: Some(1.1f32 as f64), // WARNING: precision issues!
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 2,
                    arg_path: "/1/variant_3".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 3,
                    arg_path: "/1/variant_3/field1".to_string(),
                    int_value: Some(2),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 4,
                    arg_path: "/1/variant_3/field2".to_string(),
                    ..Default::default()
                },
            ]
        );
    }

    #[tokio::test]
    async fn test_simple_fields() {
        let test1 = EnumTest::Variant1;
        assert_eq!(
            test1.get_arguments("123", 0, None, "program"),
            vec![InstructionArgument {
                tx_signature: "123".to_string(),
                instruction_idx: 0,
                inner_instructions_set: None,
                program: "program".to_string(),
                arg_idx: 0,
                arg_path: "/variant_1".to_string(),
                ..Default::default()
            }]
        );

        let test2 = TestUnit;
        assert_eq!(
            test2.get_arguments("123", 0, None, "program"),
            vec![InstructionArgument {
                tx_signature: "123".to_string(),
                instruction_idx: 0,
                inner_instructions_set: None,
                program: "program".to_string(),
                arg_idx: 0,
                arg_path: "/test_unit".to_string(),
                ..Default::default()
            }]
        );

        let test3 = TestUnnamed(1, [2, 4]);
        assert_eq!(
            test3.get_arguments("123", 0, None, "program"),
            vec![
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 0,
                    arg_path: "/0".to_string(),
                    int_value: Some(1),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 1,
                    arg_path: "/1/0".to_string(),
                    int_value: Some(2),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 2,
                    arg_path: "/1/1".to_string(),
                    int_value: Some(4),
                    ..Default::default()
                },
            ]
        );

        let test4 = EnumTest::Variant2(228.1337);
        assert_eq!(
            test4.get_arguments("123", 0, None, "program"),
            vec![
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 0,
                    arg_path: "/variant_2".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 1,
                    arg_path: "/variant_2/0".to_string(),
                    float_value: Some(228.1337f32 as f64), // WARNING: precision issues!
                    ..Default::default()
                },
            ]
        );

        let test5 = RootInstr::BoolVariant(true);

        assert_eq!(
            test5.get_arguments("123", 0, None, "program"),
            vec![InstructionArgument {
                tx_signature: "123".to_string(),
                instruction_idx: 0,
                inner_instructions_set: None,
                program: "program".to_string(),
                arg_idx: 0,
                arg_path: "/0".to_string(),
                int_value: Some(1),
                ..Default::default()
            },]
        );
    }

    #[tokio::test]
    async fn test_advanced_fields() {
        let test1 = ArrayTest {
            array: [1, 2, 3],
            tuple: Some((4, "5".to_string())),
        };
        assert_eq!(
            test1.get_arguments("123", 0, None, "program"),
            vec![
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 0,
                    arg_path: "/array/0".to_string(),
                    int_value: Some(1),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 1,
                    arg_path: "/array/1".to_string(),
                    int_value: Some(2),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 2,
                    arg_path: "/array/2".to_string(),
                    int_value: Some(3),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 3,
                    arg_path: "/tuple/0".to_string(),
                    int_value: Some(4),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 4,
                    arg_path: "/tuple/1".to_string(),
                    string_value: Some("5".to_string()),
                    ..Default::default()
                },
            ]
        );

        let test2 = EnumTest::Variant3 {
            field1: 228,
            field2: Some("TestString".to_string()),
        };

        assert_eq!(
            test2.get_arguments("123", 0, None, "program"),
            vec![
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 0,
                    arg_path: "/variant_3".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 1,
                    arg_path: "/variant_3/field1".to_string(),
                    int_value: Some(228),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 2,
                    arg_path: "/variant_3/field2".to_string(),
                    string_value: Some("TestString".to_string()),
                    ..Default::default()
                },
            ]
        );
    }

    #[tokio::test]
    async fn test_nested_fields() {
        let test1 = Test {
            field1: 100,
            field2: None,
            field3: Some(NestedTest {
                field1: Some(Some(1337)),
                field2: NestedPubkeyTest {
                    pubkey: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                },
            }),
            field4: TestUnnamed(32, [64, 128]),
            field5: TestUnit,
            field6: EnumTest::Variant3 {
                field1: 1,
                field2: Some("TestField".to_string()),
            },
            field7: ArrayTest {
                array: [1, 2, 3],
                tuple: Some((4, "5".to_string())),
            },
        };

        assert_eq!(
            test1.get_arguments("123", 0, None, "program"),
            vec![
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 0,
                    arg_path: "/field1".to_string(),
                    unsigned_value: Some(100),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 1,
                    arg_path: "/field2".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 2,
                    arg_path: "/field3/field1".to_string(),
                    unsigned_value: Some(1337),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 3,
                    arg_path: "/field3/field2/pubkey".to_string(),
                    string_value: Some("11111111111111111111111111111111".to_string()),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 4,
                    arg_path: "/field4/0".to_string(),
                    int_value: Some(32),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 5,
                    arg_path: "/field4/1/0".to_string(),
                    int_value: Some(64),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 6,
                    arg_path: "/field4/1/1".to_string(),
                    int_value: Some(128),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 7,
                    arg_path: "/field5/test_unit".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 8,
                    arg_path: "/field6/variant_3".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 9,
                    arg_path: "/field6/variant_3/field1".to_string(),
                    int_value: Some(1),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 10,
                    arg_path: "/field6/variant_3/field2".to_string(),
                    string_value: Some("TestField".to_string()),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 11,
                    arg_path: "/field7/array/0".to_string(),
                    int_value: Some(1),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 12,
                    arg_path: "/field7/array/1".to_string(),
                    int_value: Some(2),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 13,
                    arg_path: "/field7/array/2".to_string(),
                    int_value: Some(3),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 14,
                    arg_path: "/field7/tuple/0".to_string(),
                    int_value: Some(4),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 15,
                    arg_path: "/field7/tuple/1".to_string(),
                    string_value: Some("5".to_string()),
                    ..Default::default()
                },
            ]
        );

        let test2 = Test {
            field1: 100,
            field2: None,
            field3: None,
            field4: TestUnnamed(32, [64, 128]),
            field5: TestUnit,
            field6: EnumTest::Variant3 {
                field1: 1,
                field2: Some("TestField".to_string()),
            },
            field7: ArrayTest {
                array: [1, 2, 3],
                tuple: Some((4, "5".to_string())),
            },
        };

        assert_eq!(
            test2.get_arguments("123", 0, None, "program"),
            vec![
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 0,
                    arg_path: "/field1".to_string(),
                    unsigned_value: Some(100),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 1,
                    arg_path: "/field2".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 2,
                    arg_path: "/field3".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 3,
                    arg_path: "/field4/0".to_string(),
                    int_value: Some(32),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 4,
                    arg_path: "/field4/1/0".to_string(),
                    int_value: Some(64),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 5,
                    arg_path: "/field4/1/1".to_string(),
                    int_value: Some(128),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 6,
                    arg_path: "/field5/test_unit".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 7,
                    arg_path: "/field6/variant_3".to_string(),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 8,
                    arg_path: "/field6/variant_3/field1".to_string(),
                    int_value: Some(1),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 9,
                    arg_path: "/field6/variant_3/field2".to_string(),
                    string_value: Some("TestField".to_string()),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 10,
                    arg_path: "/field7/array/0".to_string(),
                    int_value: Some(1),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 11,
                    arg_path: "/field7/array/1".to_string(),
                    int_value: Some(2),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 12,
                    arg_path: "/field7/array/2".to_string(),
                    int_value: Some(3),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 13,
                    arg_path: "/field7/tuple/0".to_string(),
                    int_value: Some(4),
                    ..Default::default()
                },
                InstructionArgument {
                    tx_signature: "123".to_string(),
                    instruction_idx: 0,
                    inner_instructions_set: None,
                    program: "program".to_string(),
                    arg_idx: 14,
                    arg_path: "/field7/tuple/1".to_string(),
                    string_value: Some("5".to_string()),
                    ..Default::default()
                },
            ]
        );
    }
}
