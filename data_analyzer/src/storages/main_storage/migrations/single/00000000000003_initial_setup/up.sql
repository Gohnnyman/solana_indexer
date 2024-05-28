CREATE TABLE IF NOT EXISTS erroneous_transactions 
(
    slot UInt64,
    transaction String,
    tx_signature String,
    cause String
) ENGINE = MergeTree()
ORDER BY (tx_signature, slot)
SETTINGS index_granularity = 8192;
