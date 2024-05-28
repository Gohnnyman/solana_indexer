CREATE TABLE IF NOT EXISTS erroneous_transactions ON CLUSTER '{cluster}'
(
    slot UInt64,
    transaction String,
    tx_signature String,
    cause String
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/01/{database}/{table}', '{replica}')
ORDER BY (tx_signature, slot)
SETTINGS index_granularity = 8192;
