CREATE TABLE IF NOT EXISTS instructions ON CLUSTER '{cluster}'
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
    data String,
    INDEX slot_idx slot TYPE minmax GRANULARITY 8192,
    INDEX instruction_name_idx instruction_name TYPE minmax GRANULARITY 8192,
    INDEX account_1_idx account_1 TYPE minmax GRANULARITY 8192,
    INDEX tx_signature_idx tx_signature TYPE minmax GRANULARITY 8192
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/01/{database}/{table}', '{replica}')
ORDER BY (program, instruction_name)
SETTINGS index_granularity = 8192;
