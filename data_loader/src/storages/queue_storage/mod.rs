#[allow(clippy::extra_unused_lifetimes)]
pub mod models;
pub mod schema;

use self::models::{NewDownloadingStatus, NewSignature, NewTransaction};
use self::schema::{
    downloading_statuses::columns::key, downloading_statuses::dsl::*, signatures::dsl::*,
    transactions::dsl::*,
};
use anyhow::Result;

use diesel::{pg::PgConnection, prelude::*};
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

pub struct QueueStorage {
    connection: PgConnection,
}

embed_migrations!("./src/storages/queue_storage/migrations");

impl QueueStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let connection = establish_connection(database_url)?;
        embedded_migrations::run(&connection)?;
        Ok(QueueStorage { connection })
    }
}

fn establish_connection(database_url: &str) -> Result<PgConnection> {
    Ok(PgConnection::establish(database_url)?)
}

fn format_or_empty<T: std::fmt::Debug>(val: Option<T>) -> String {
    if val.is_some() {
        format!("{:?}", val.unwrap())
    } else {
        String::from("")
    }
}

impl QueueStorage {
    pub fn load_downloading_status(&self, account_key: &str) -> Option<String> {
        let conn = &self.connection;

        if let Ok(result) = downloading_statuses
            .select(downloading_status)
            .filter(key.eq(account_key))
            .first::<Option<String>>(conn)
        {
            result
        } else {
            None
        }
    }

    pub fn get_signature_from_queue(
        &self,
        load_only_successful_transactions: bool,
    ) -> Option<String> {
        let conn = &self.connection;

        let result = if load_only_successful_transactions {
            signatures
                .select(schema::signatures::dsl::signature)
                .filter(loading_status.eq(0))
                .filter(err.eq(""))
                .order(schema::signatures::dsl::slot.desc())
                .first::<String>(conn)
        } else {
            signatures
                .select(schema::signatures::dsl::signature)
                .filter(loading_status.eq(0))
                .order(schema::signatures::dsl::slot.desc())
                .first::<String>(conn)
        };

        match result {
            Ok(result) => {
                let sign = result.clone();
                let target = signatures.filter(schema::signatures::dsl::signature.eq(sign));

                diesel::update(target)
                    .set(loading_status.eq(1))
                    .execute(conn)
                    .unwrap();
                Some(result)
            }
            Err(_) => None,
        }
    }

    pub fn mark_signature_as_loaded(&self, sign: String) -> Result<()> {
        let target = signatures.filter(schema::signatures::dsl::signature.eq(sign));

        diesel::update(target)
            .set(loading_status.eq(2))
            .execute(&self.connection)?;

        Ok(())
    }

    pub fn mark_signature_loading_fault(&self, sign: String) -> Result<()> {
        let target = signatures.filter(schema::signatures::dsl::signature.eq(sign));

        diesel::update(target)
            .set(loading_status.eq(99))
            .execute(&self.connection)?;

        Ok(())
    }

    pub fn store_transaction(
        &self,
        sign: &str,
        tx: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<()> {
        let new_transaction = NewTransaction {
            slot: tx.slot as i32,
            transaction: &serde_json::to_string(&tx.transaction).unwrap(),
            block_time: tx.block_time.unwrap_or_default() as i32,
            parsing_status: 0_i32,
            signature: sign,
        };

        let conn = &self.connection;

        conn.build_transaction()
            .run::<(), diesel::result::Error, _>(|| {
                diesel::insert_into(transactions)
                    .values(&new_transaction)
                    .on_conflict_do_nothing()
                    .execute(conn)?;

                let target = signatures.filter(schema::signatures::dsl::signature.eq(sign));

                diesel::update(target)
                    .set(loading_status.eq(2))
                    .execute(conn)?;

                Ok(())
            })?;
        Ok(())
    }

    pub fn store_signatures_and_state(
        &self,
        transaction_statuses: &[RpcConfirmedTransactionStatusWithSignature],
        account_key: &str,
        status: &str,
    ) -> Result<usize> {
        let conn = &self.connection;

        let mut new_signatures = Vec::new();

        for transaction_status in transaction_statuses {
            let new_signature = NewSignature {
                signature: &transaction_status.signature,
                slot: transaction_status.slot as i32,
                err: format_or_empty(transaction_status.err.as_ref()),
                memo: format_or_empty(transaction_status.memo.as_ref()),
                block_time: transaction_status.block_time.unwrap_or_default() as i32,
                confirmation_status: format_or_empty(
                    transaction_status.confirmation_status.as_ref(),
                ),
                loading_status: 0_i32,
                program: account_key,
                potential_gap_start: false,
            };

            new_signatures.push(new_signature);
        }

        if !new_signatures.is_empty() {
            new_signatures
                .iter_mut()
                .last()
                .unwrap()
                .potential_gap_start = true
        }

        let new_downloading_status = NewDownloadingStatus {
            key: account_key,
            downloading_status: status,
        };

        let ret_result = conn
            .build_transaction()
            .run::<usize, diesel::result::Error, _>(|| {
                let mut rows_inserted = 0;

                if !new_signatures.is_empty() {
                    let first_in_batch = new_signatures.get(0).unwrap().signature;

                    diesel::update(
                        signatures
                            .filter(schema::signatures::dsl::signature.eq(first_in_batch))
                            .filter(program.eq(account_key)),
                    )
                    .set(potential_gap_start.eq(false))
                    .execute(conn)?;

                    rows_inserted = diesel::insert_into(signatures)
                        .values(&new_signatures)
                        .on_conflict_do_nothing()
                        .execute(conn)?;
                }

                let result = diesel::update(downloading_statuses.filter(key.eq(account_key)))
                    .set(downloading_status.eq(status))
                    .execute(conn);

                if result.is_err() || (result.is_ok() && result? < 1) {
                    diesel::insert_into(downloading_statuses)
                        .values(&new_downloading_status)
                        .on_conflict_do_nothing()
                        .execute(conn)?;
                }

                Ok(rows_inserted)
            })?;
        Ok(ret_result)
    }

    pub fn reset_loading_status(&self) -> Result<()> {
        let conn = &self.connection;

        let target = signatures.filter(schema::signatures::dsl::loading_status.eq(99));
        diesel::update(target)
            .set(loading_status.eq(0))
            .execute(conn)
            .unwrap();

        Ok(())
    }

    pub fn reset_status_loading_in_progress(&self) -> Result<()> {
        let conn = &self.connection;

        let target = signatures.filter(schema::signatures::dsl::loading_status.eq(1));
        diesel::update(target)
            .set(loading_status.eq(0))
            .execute(conn)
            .unwrap();

        Ok(())
    }
}
