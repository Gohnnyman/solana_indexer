use anyhow::Result;
use async_trait::async_trait;
use clickhouse_http::{Client, Row};
use dsn::DSN;
use serde::{Deserialize, Serialize};

use crate::errors::MainStorageError;
use crate::storages::main_storage::{
    Balance, ErroneousTransaction, Instruction, InstructionArgument, MainStorage, TxStatus,
};

use super::Delegation;

pub struct HttpsClient {
    client: Client,
}

impl HttpsClient {
    pub async fn new(db_creds: DSN) -> Result<Self, MainStorageError> {
        let protocol = db_creds.driver;
        let address = db_creds.address;

        let mut client = if protocol == "https" {
            Client::with_https_client().with_url(format!("{protocol}://{address}"))
        } else {
            Client::default().with_url(format!("{protocol}://{address}"))
        };

        if let Some(user_name) = db_creds.username {
            client = client.with_user(user_name);
        }
        if let Some(password) = db_creds.password {
            client = client.with_password(password);
        }
        if let Some(db) = db_creds.database {
            client = client.with_database(db);
        }

        Ok(Self { client })
    }
}

#[async_trait]
impl MainStorage for HttpsClient {
    async fn execute(&mut self, ddl: &str) -> Result<()> {
        let query = self.client.query(ddl);
        query.execute().await?;
        Ok(())
    }

    async fn migration_exists(&mut self, version: &str) -> Result<bool> {
        let mut cursor = self
            .client
            .query("SELECT COUNT(*) AS count FROM __schema_migrations WHERE version = ?")
            .bind(version)
            .fetch::<u64>()?;

        if let Some(count) = cursor.next().await? {
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    async fn store_instructions_block(&mut self, instructions: Vec<Instruction>) -> Result<()> {
        let mut insert = self.client.insert("instructions")?;

        for instruction in instructions {
            insert
                .write(&InstructionRow {
                    program: instruction.program.clone(),
                    tx_signature: instruction.tx_signature.clone(),
                    tx_status: instruction.tx_status,
                    slot: instruction.slot,
                    block_time: instruction.block_time,
                    instruction_idx: instruction.instruction_idx,
                    inner_instructions_set: instruction.inner_instructions_set,
                    transaction_instruction_idx: instruction.transaction_instruction_idx,
                    instruction_name: instruction.instruction_name.clone(),
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
                    data: instruction.data.clone(),
                })
                .await?;
        }

        insert.end().await?;

        Ok(())
    }

    async fn store_instruction_arguments_block(
        &mut self,
        instruction_arguments: Vec<InstructionArgument>,
    ) -> Result<()> {
        let mut insert = self.client.insert("instruction_arguments")?;

        for instruction_argument in instruction_arguments {
            insert
                .write(&InstructionArgumentsRow {
                    tx_signature: instruction_argument.tx_signature.clone(),
                    instruction_idx: instruction_argument.instruction_idx,
                    inner_instructions_set: instruction_argument.inner_instructions_set,
                    program: instruction_argument.program,
                    arg_idx: instruction_argument.arg_idx,
                    arg_path: instruction_argument.arg_path,
                    int_value: instruction_argument.int_value,
                    unsigned_value: instruction_argument.unsigned_value,
                    float_value: instruction_argument.float_value,
                    string_value: instruction_argument.string_value,
                    enum_value: None, // TODO: Why?
                })
                .await?;
        }

        insert.end().await?;

        Ok(())
    }

    async fn store_balances_block(&mut self, balances: Vec<Balance>) -> Result<()> {
        let mut insert = self.client.insert("balances")?;

        for balance in balances {
            insert
                .write(&BalancesRow {
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
                })
                .await?;
        }

        insert.end().await?;

        Ok(())
    }

    async fn store_delegations_block(&mut self, delegations: Vec<Delegation>) -> Result<()> {
        let mut insert = self.client.insert("delegations")?;

        for delegation in delegations {
            insert.write(&delegation).await?;
        }

        insert.end().await?;

        Ok(())
    }

    async fn store_undelegations_block(&mut self, undelegations: Vec<Delegation>) -> Result<()> {
        let mut insert = self.client.insert("undelegations")?;

        for undelegation in undelegations {
            insert.write(&undelegation).await?;
        }

        insert.end().await?;

        Ok(())
    }

    async fn store_erroneous_transaction_block(
        &mut self,
        erroneous_transactions: Vec<ErroneousTransaction>,
    ) -> Result<()> {
        let mut insert = self.client.insert("erroneous_transactions")?;

        for erroneous_transaction in erroneous_transactions {
            insert
                .write(&ErroneousTransactionRow {
                    slot: erroneous_transaction.slot,
                    transaction: erroneous_transaction.transaction,
                    tx_signature: erroneous_transaction.tx_signature,
                    cause: erroneous_transaction.cause,
                })
                .await?;
        }

        insert.end().await?;

        Ok(())
    }
}

#[derive(Row, Serialize, Deserialize)]
pub struct InstructionRow {
    pub program: String,
    pub tx_signature: String,
    pub tx_status: TxStatus,
    pub slot: u64,
    pub block_time: u64,
    pub instruction_idx: u8,
    pub inner_instructions_set: Option<u8>,
    pub transaction_instruction_idx: Option<u8>,
    pub instruction_name: String,
    pub account_0: Option<String>,
    pub account_1: Option<String>,
    pub account_2: Option<String>,
    pub account_3: Option<String>,
    pub account_4: Option<String>,
    pub account_5: Option<String>,
    pub account_6: Option<String>,
    pub account_7: Option<String>,
    pub account_8: Option<String>,
    pub account_9: Option<String>,
    pub account_10: Option<String>,
    pub account_11: Option<String>,
    pub account_12: Option<String>,
    pub account_13: Option<String>,
    pub account_14: Option<String>,
    pub account_15: Option<String>,
    pub account_16: Option<String>,
    pub account_17: Option<String>,
    pub account_18: Option<String>,
    pub account_19: Option<String>,
    pub account_20: Option<String>,
    pub account_21: Option<String>,
    pub account_22: Option<String>,
    pub account_23: Option<String>,
    pub account_24: Option<String>,
    pub account_25: Option<String>,
    pub account_26: Option<String>,
    pub account_27: Option<String>,
    pub account_28: Option<String>,
    pub account_29: Option<String>,
    pub account_30: Option<String>,
    pub account_31: Option<String>,
    pub account_32: Option<String>,
    pub account_33: Option<String>,
    pub account_34: Option<String>,
    pub data: String,
}

#[derive(Row, Serialize, Deserialize)]
pub struct MetadataRow {
    pub slot: u64,
    pub blockhash: String,
    pub rewards: String,
    pub block_time: i64,
    pub block_height: Option<u64>,
}

#[derive(Row, Serialize, Deserialize)]
pub struct BalancesRow {
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

#[derive(Row, Serialize, Deserialize)]
pub struct InstructionArgumentsRow {
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
    pub enum_value: Option<String>,
}

#[derive(Row, Serialize, Deserialize)]
pub struct ErroneousTransactionRow {
    pub slot: u64,
    pub transaction: String,
    pub tx_signature: String,
    pub cause: String,
}
