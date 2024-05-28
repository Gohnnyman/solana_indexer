CREATE TABLE IF NOT EXISTS delegations ON CLUSTER '{cluster}'
(
    slot UInt64,
    block_time UInt64,
    stake_acc String,
    vote_acc Nullable(String),
    tx_signature String,
    amount UInt64,
    raw_instruction_idx UInt16
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/01/{database}/{table}', '{replica}')
ORDER BY (stake_acc, slot)
SETTINGS index_granularity = 8192;