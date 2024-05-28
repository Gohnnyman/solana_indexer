use super::schema::{downloading_statuses, signatures, transactions};

#[derive(Insertable, Debug)]
#[table_name = "signatures"]
pub struct NewSignature<'a> {
    pub signature: &'a str,
    pub slot: i32,
    pub err: String,
    pub memo: String,
    pub block_time: i32,
    pub confirmation_status: String,
    pub loading_status: i32,
    pub program: &'a str,
    pub potential_gap_start: bool,
}

#[derive(Queryable)]
pub struct Signature {
    pub signature: String,
    pub slot: i32,
    pub err: String,
    pub memo: String,
    pub block_time: i32,
    pub confirmation_status: String,
    pub loading_status: i32,
    pub program: String,
    pub potential_gap_start: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "downloading_statuses"]
pub struct NewDownloadingStatus<'a> {
    pub key: &'a str,
    pub downloading_status: &'a str,
}

#[derive(Queryable)]
pub struct DownloadingStatus {
    pub id: i32,
    pub key: String,
    pub downloading_status: String,
}

#[derive(Insertable)]
#[table_name = "transactions"]
pub struct NewTransaction<'a> {
    pub slot: i32,
    pub transaction: &'a str,
    pub block_time: i32,
    pub parsing_status: i32,
    pub signature: &'a str,
}

#[derive(Queryable)]
pub struct Transaction {
    pub transaction: String,
    pub block_time: i32,
    pub parsing_status: i32,
    pub signature: String,
}
