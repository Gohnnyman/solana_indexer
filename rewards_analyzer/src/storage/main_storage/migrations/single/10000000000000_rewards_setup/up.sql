CREATE TABLE IF NOT EXISTS rewards
(
    `vote_account` String,
    `epoch` UInt64,
    `pubkey` String,
    `lamports` Int64,
    `post_balance` UInt64,
    `reward_type` Nullable(String),
    `commission` Nullable(UInt8),
    `first_block_slot` Nullable(UInt64),
    `block_time` DateTime('UTC')
)
ENGINE = MergeTree()
ORDER BY (epoch, pubkey)
SETTINGS index_granularity = 8192;