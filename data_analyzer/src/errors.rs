use thiserror::Error;
#[derive(Debug, Error)]
pub enum ParseInstructionError {
    #[error("Failed to convert to serde_json: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Failed to get sighash of instruction: {0}")]
    SighashFromSliceError(#[from] std::array::TryFromSliceError),

    #[error("Failed to deserialize instruction: {0}")]
    DeserializeError(#[from] std::io::Error),

    #[error("Failed to deserialize in {instruction}: {err}")]
    DeserializeInInstructionError {
        instruction: String,
        err: std::io::Error,
    },

    #[error("Failed to limited_deserialize in {instruction}: {err}")]
    LimDeserializeInInstructionError {
        instruction: String,
        err: solana_program::instruction::InstructionError,
    },

    #[error("Failed to deserialize instruction from base58")]
    DeserializeFromBase58Error,

    #[error("Failed to parse instruction: {0}")]
    ParseError(String),

    #[error("Invalid index in {site}: {index}, when length is {max_len}")]
    InvalidIndex {
        site: String,
        index: usize,
        max_len: usize,
    },

    #[error("{site} has invalid length: {len} instead of {expected_len}")]
    InvalidLength {
        site: String,
        len: usize,
        expected_len: usize,
    },

    #[error("Converting Error: {0}")]
    ConvertingError(#[from] ConvertingError),

    #[error("Cannot get instruction name")]
    InvalidInstructionName,

    #[error("Given hash doesn't match any sighash in {0}")]
    SighashMatchError(String),

    #[error("Address doesn't match any program")]
    ProgramAddressMatchError,

    #[error("{0} is unsupported")]
    Unsupported(String),
}

impl From<rust_base58::base58::FromBase58Error> for ParseInstructionError {
    fn from(_: rust_base58::base58::FromBase58Error) -> Self {
        Self::DeserializeFromBase58Error
    }
}

#[derive(Debug, Error)]
pub enum ConvertingError {
    #[error("Cannot get {0} field")]
    EmptyField(String),

    #[error("Types has different lengths")]
    DifferentLengths,

    #[error("{0} is unsupported")]
    Unsupported(String),

    #[error("Failed to deserialize: {0}")]
    DeserializeError(#[from] serde_json::error::Error),
}

#[derive(Debug, Error, PartialEq)]
#[error("Failed to connect to PostgreSQL {source}")]
pub struct PostgreSQLError {
    #[from]
    source: diesel::result::ConnectionError,
}

#[derive(Debug, Error)]
#[error("Failed to connect to Main Storage")]
pub struct MainStorageError {
    #[from]
    source: clickhouse_rs::errors::Error,
}

#[derive(Debug, Error)]
pub enum QueueManagerError {
    #[error("Failed to get data from queue manager")]
    RecvError(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Custom error: {0}")]
    CustomError(#[from] anyhow::Error),
}

#[derive(Debug, Error, PartialEq)]
pub enum RabbitMQError {
    #[error("Failed to connect to RabbitMQ: {0}")]
    ConnectionError(#[from] lapin::Error),
}
