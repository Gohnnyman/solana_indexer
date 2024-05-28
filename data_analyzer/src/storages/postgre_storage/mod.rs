#[allow(clippy::extra_unused_lifetimes)]
pub mod models;
pub mod schema;

use self::models::{Delegation, Transaction};
use super::QueueStorage;

use crate::errors::PostgreSQLError;
use anyhow::Result;
use async_trait::async_trait;
use diesel::{
    pg::{upsert::excluded, PgConnection},
    prelude::*,
    result::Error,
};
use log::{error, info};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use url::Url;

pub struct PostgreStorage {
    connection: PgConnection,
}

impl PostgreStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let connection = establish_connection(database_url)?;
        let parsed_url = Url::parse(database_url)?;
        info!(
            "PostgreSQL connection established: {}://******:******@{}{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap(),
            parsed_url.path()
        );
        Ok(PostgreStorage { connection })
    }
}

fn establish_connection(database_url: &str) -> Result<PgConnection, PostgreSQLError> {
    Ok(PgConnection::establish(database_url)?)
}

fn _format_or_empty<T: std::fmt::Debug>(val: Option<T>) -> String {
    if val.is_some() {
        format!("{:?}", val.unwrap())
    } else {
        String::from("")
    }
}

#[async_trait]
impl QueueStorage for PostgreStorage {
    async fn get_transactions(&mut self) -> Vec<EncodedConfirmedTransactionWithStatusMeta> {
        use schema::transactions::dsl::*;
        let conn = &self.connection;

        let query_result = transactions
            .filter(parsing_status.eq(0))
            .order(slot)
            .limit(1000)
            .load::<Transaction>(conn);

        match query_result {
            Ok(query_result) => {
                let mut sgntrs = Vec::with_capacity(query_result.len());
                let encoded_confirmed_transactions: Vec<_> = query_result
                    .into_iter()
                    .map(|tx| {
                        sgntrs.push(tx.signature.clone());
                        EncodedConfirmedTransactionWithStatusMeta {
                            slot: tx.slot.unwrap_or_default() as u64,
                            transaction: serde_json::from_str(&tx.transaction.unwrap()).unwrap(),
                            block_time: Some(tx.block_time.unwrap_or_default().into()),
                        }
                    })
                    .collect();

                encoded_confirmed_transactions
            }
            Err(err) => match err {
                Error::NotFound => {
                    info!("get_transaction: NotFound");
                    vec![]
                }
                _ => {
                    error!("Postgre cannot run query: {:#?}", err);
                    vec![]
                }
            },
        }
    }

    async fn get_delegations(&mut self, stake_accs: Vec<String>) -> Result<Vec<Delegation>> {
        use schema::delegations::dsl::*;
        let conn = &self.connection;

        Ok(delegations
            .filter(stake_acc.eq_any(stake_accs))
            .load::<Delegation>(conn)?)
    }

    async fn save_delegations(&mut self, delegations_vec: Vec<Delegation>) -> Result<()> {
        use schema::delegations;
        let conn = &self.connection;

        diesel::insert_into(delegations::table)
            .values(delegations_vec)
            .on_conflict(delegations::stake_acc)
            .do_update()
            .set(delegations::vote_acc.eq(excluded(delegations::vote_acc)))
            .execute(conn)?;

        Ok(())
    }

    async fn mark_transaction_as_parsed(&mut self, transaction: String) -> Result<()> {
        use schema::transactions;
        let conn = &self.connection;

        diesel::update(transactions::table)
            .filter(transactions::signature.eq(transaction))
            .set(transactions::parsing_status.eq(1))
            .execute(conn)?;

        Ok(())
    }
}
