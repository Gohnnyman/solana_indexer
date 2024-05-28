use thiserror::Error;

#[derive(Error, Debug)]
pub enum RewardsAnalyzerError {
    #[error("Failed to retrieve an epoch: {0}")]
    EpochStorage(#[from] EpochStorageError),

    #[error("MainStorage error {0}")]
    MainStorage(#[from] MainStorageError),

    #[error("RewardsCollector error {0}")]
    RewardsCollector(#[from] RewardsCollectorError),
}

#[derive(Error, Debug)]
pub enum DelegationsAnalyzerError {
    #[error("MainStorage error {0}")]
    MainStorage(#[from] MainStorageError),

    #[error("DelegationsCollector error {0}")]
    DelegationsCollector(#[from] DelegationsCollectorError),
}

#[derive(Error, Debug)]
pub enum EpochStorageError {
    #[error("Failed to connect to PostgreSQL Server: {0} ")]
    PostgresConnection(#[from] tokio_postgres::Error),
}

#[derive(Debug, Error)]
pub enum MainStorageError {
    #[error("Unknown protocol")]
    UnknownProtocol,
    #[error("Failed to connect to Main Storage: {0} ")]
    ClickhouseError(#[from] clickhouse_rs::errors::Error),
    #[error("Clickhouse HTTP error: {0} ")]
    ClickhouseHttp(#[from] clickhouse_http::error::Error),
}

#[derive(Debug, Error)]
pub enum RewardsCollectorError {
    #[error("MainStorage error {0}")]
    MainStorage(#[from] MainStorageError),
}

#[derive(Debug, Error)]
pub enum DelegationsCollectorError {
    #[error("MainStorage error {0}")]
    MainStorage(#[from] MainStorageError),
}

#[derive(Error, Debug)]
pub enum VoteAccountResolverError {
    #[error("MainStorage error {0}")]
    MainStorage(#[from] MainStorageError),
}
