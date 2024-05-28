CREATE TABLE IF NOT EXISTS instruction_arguments ON CLUSTER '{cluster}'
(
    tx_signature String,
    instruction_idx UInt8,
    inner_instructions_set Nullable(UInt8),
    program String,
    arg_idx UInt16,
    arg_path String,
    int_value Nullable(Int64),
    unsigned_value Nullable(UInt64),
    float_value Nullable(Float64),
    string_value Nullable(String),
    enum_value Nullable(String),
    INDEX arg_path_idx arg_path TYPE minmax GRANULARITY 8192,
    INDEX inner_instructions_set_idx inner_instructions_set TYPE minmax GRANULARITY 8192,
    INDEX instruction_idx_idx instruction_idx TYPE minmax GRANULARITY 8192
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/01/{database}/{table}', '{replica}')
ORDER BY (tx_signature, program)
SETTINGS index_granularity = 8192;
