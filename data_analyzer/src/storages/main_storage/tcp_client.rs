use anyhow::Result;
use async_trait::async_trait;
use clickhouse_rs::{
    row,
    types::{Block, Enum8},
    ClientHandle, Pool,
};
use dsn::DSN;

use crate::errors::MainStorageError;
use crate::storages::main_storage::{
    Balance, ErroneousTransaction, Instruction, InstructionArgument, MainStorage,
};

use super::Delegation;

pub struct TcpClient {
    client: ClientHandle,
}

impl TcpClient {
    pub async fn new(db_creds: DSN) -> Result<Self, MainStorageError> {
        let mut database_url = format!("{}://", db_creds.driver);

        if let Some(user_name) = db_creds.username {
            database_url = format!("{database_url}{user_name}");
        }
        if let Some(password) = db_creds.password {
            database_url = format!("{database_url}:{password}");
        }

        let address = db_creds.address;
        database_url = format!("{database_url}@{address}");

        if let Some(db) = db_creds.database {
            database_url = format!("{database_url}/{db}");
        }

        let pool = Pool::new(database_url);
        let client = pool.get_handle().await?;
        Ok(Self { client })
    }

    #[allow(unused)]
    pub async fn ping(&mut self) -> Result<()> {
        self.client.ping().await?;
        Ok(())
    }

    pub fn get_handle(&mut self) -> &mut ClientHandle {
        &mut self.client
    }
}

#[allow(unused)]
#[async_trait]
impl MainStorage for TcpClient {
    async fn execute(&mut self, ddl: &str) -> Result<()> {
        let client = self.get_handle();
        client.execute(ddl).await?;
        Ok(())
    }

    async fn migration_exists(&mut self, version: &str) -> Result<bool> {
        let client = self.get_handle();
        let query = &format!(
            "SELECT COUNT(*) AS count FROM __schema_migrations WHERE version = '{}'",
            version
        );

        let block = client.query(query).fetch_all().await?;

        return if let Some(row) = block.rows().next() {
            let count: u64 = row.get("count")?;
            Ok(count > 0)
        } else {
            Ok(false)
        };
    }

    async fn store_instructions_block(&mut self, instructions: Vec<Instruction>) -> Result<()> {
        let block_size = instructions.len();

        let mut block = Block::with_capacity(block_size);

        for instruction in instructions {
            block.push(row! {program: *instruction.program,
                tx_signature: *instruction.tx_signature,
                tx_status: Enum8::of(instruction.tx_status.into()),
                slot: instruction.slot,
                block_time: instruction.block_time,
                instruction_idx: instruction.instruction_idx,
                inner_instructions_set: instruction.inner_instructions_set,
                transaction_instruction_idx: instruction.transaction_instruction_idx,
                instruction_name: *instruction.instruction_name,
                account_0: instruction.accounts[0].clone(),
                account_1: instruction.accounts[1].clone(),
                account_2: instruction.accounts[2].clone(),
                account_3: instruction.accounts[3].clone(),
                account_4: instruction.accounts[4].clone(),
                account_5: instruction.accounts[5].clone(),
                account_6: instruction.accounts[6].clone(),
                account_7: instruction.accounts[7].clone(),
                account_8: instruction.accounts[8].clone(),
                account_9: instruction.accounts[9].clone(),
                account_10: instruction.accounts[10].clone(),
                account_11: instruction.accounts[11].clone(),
                account_12: instruction.accounts[12].clone(),
                account_13: instruction.accounts[13].clone(),
                account_14: instruction.accounts[14].clone(),
                account_15: instruction.accounts[15].clone(),
                account_16: instruction.accounts[16].clone(),
                account_17: instruction.accounts[17].clone(),
                account_18: instruction.accounts[18].clone(),
                account_19: instruction.accounts[19].clone(),
                account_20: instruction.accounts[20].clone(),
                account_21: instruction.accounts[21].clone(),
                account_22: instruction.accounts[22].clone(),
                account_23: instruction.accounts[23].clone(),
                account_24: instruction.accounts[24].clone(),
                account_25: instruction.accounts[25].clone(),
                account_26: instruction.accounts[26].clone(),
                account_27: instruction.accounts[27].clone(),
                account_28: instruction.accounts[28].clone(),
                account_29: instruction.accounts[29].clone(),
                account_30: instruction.accounts[30].clone(),
                account_31: instruction.accounts[31].clone(),
                account_32: instruction.accounts[32].clone(),
                account_33: instruction.accounts[33].clone(),
                account_34: instruction.accounts[34].clone(),
                data: *instruction.data,
            })?;
        }

        let client = self.get_handle();
        client.insert("instructions", block).await?;

        Ok(())
    }

    async fn store_instruction_arguments_block(
        &mut self,
        instruction_arguments: Vec<InstructionArgument>,
    ) -> Result<()> {
        let block_size = instruction_arguments.len();

        let mut block = Block::with_capacity(block_size);

        for instruction_argument in instruction_arguments {
            block.push(row! {
                tx_signature: *instruction_argument.tx_signature,
                instruction_idx: instruction_argument.instruction_idx,
                inner_instructions_set: instruction_argument.inner_instructions_set,
                program: instruction_argument.program,
                arg_idx: instruction_argument.arg_idx,
                arg_path: instruction_argument.arg_path,
                int_value: instruction_argument.int_value,
                unsigned_value: instruction_argument.unsigned_value,
                float_value: instruction_argument.float_value,
                string_value: instruction_argument.string_value,
            })?;
        }

        let client = self.get_handle();
        client.insert("instruction_arguments", block).await?;
        Ok(())
    }

    async fn store_balances_block(&mut self, balances: Vec<Balance>) -> Result<()> {
        let block_size = balances.len();

        let mut block = Block::with_capacity(block_size);

        for balance in balances {
            block.push(row! {
                tx_signature: balance.tx_signature,
                account: balance.account,
                pre_balance: balance.pre_balance,
                post_balance: balance.post_balance,
                pre_token_balance_mint: balance.pre_token_balance_mint,
                pre_token_balance_owner: balance.pre_token_balance_owner,
                pre_token_balance_amount: balance.pre_token_balance_amount,
                pre_token_balance_program_id: balance.pre_token_balance_program_id,
                post_token_balance_mint: balance.post_token_balance_mint,
                post_token_balance_owner: balance.post_token_balance_owner,
                post_token_balance_amount: balance.post_token_balance_amount,
                post_token_balance_program_id: balance.post_token_balance_program_id,
            })?;
        }

        let client = self.get_handle();
        client.insert("balances", block).await?;
        Ok(())
    }

    async fn store_delegations_block(&mut self, delegations: Vec<Delegation>) -> Result<()> {
        let block_size = delegations.len();

        let mut block = Block::with_capacity(block_size);

        for delegation in delegations {
            block.push(row! {
                slot: delegation.slot,
                block_time: delegation.block_time,
                stake_acc: delegation.stake_acc,
                vote_acc: delegation.vote_acc,
                tx_signature: delegation.tx_signature,
                amount: delegation.amount,
                raw_instruction_idx: delegation.raw_instruction_idx,
            })?;
        }

        let client = self.get_handle();
        client.insert("delegations", block).await?;
        Ok(())
    }

    async fn store_undelegations_block(&mut self, undelegations: Vec<Delegation>) -> Result<()> {
        let block_size = undelegations.len();

        let mut block = Block::with_capacity(block_size);

        for undelegation in undelegations {
            block.push(row! {
                slot: undelegation.slot,
                block_time: undelegation.block_time,
                stake_acc: undelegation.stake_acc,
                vote_acc: undelegation.vote_acc,
                tx_signature: undelegation.tx_signature,
                amount: undelegation.amount,
                raw_instruction_idx: undelegation.raw_instruction_idx,
            })?;
        }

        let client = self.get_handle();
        client.insert("undelegations", block).await?;
        Ok(())
    }

    async fn store_erroneous_transaction_block(
        &mut self,
        erroneous_transactions: Vec<ErroneousTransaction>,
    ) -> Result<()> {
        let block_size = erroneous_transactions.len();

        let mut block = Block::with_capacity(block_size);

        for erroneous_transactions in erroneous_transactions {
            block.push(row! {
               slot: erroneous_transactions.slot,
               transaction: erroneous_transactions.transaction,
               tx_signature: erroneous_transactions.tx_signature,
               cause: erroneous_transactions.cause
            })?;
        }

        let client = self.get_handle();

        client.insert("erroneous_transactions", block).await?;

        Ok(())
    }
}
