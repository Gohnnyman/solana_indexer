use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum EpochTrackerError {
    #[error("Failed to retrieve first and last slots of an epoch: {0}")]
    EpochStorage(#[from] EpochStorageError),

    #[error("Failed rpc request: {0}")]
    SolanaClient(#[from] solana_client::client_error::ClientError),

    #[error("Failed JSON encode: {0}")]
    SerdeJsonEncoding(#[from] serde_json::error::Error),
}

#[derive(Error, Debug)]
pub enum EpochStorageError {
    #[error("Failed to connect to PostgreSQL Server: {0} ")]
    PostgresConnection(#[from] tokio_postgres::Error),
}
