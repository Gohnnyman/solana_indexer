CREATE TABLE IF NOT EXISTS balances 
(
    tx_signature String, 
    account String,
    pre_balance Nullable(UInt64),
    post_balance Nullable(UInt64),
    pre_token_balance_mint Nullable(String),
    pre_token_balance_owner Nullable(String),
    pre_token_balance_amount Nullable(Float64),
    pre_token_balance_program_id Nullable(String),
    post_token_balance_mint Nullable(String),
    post_token_balance_owner Nullable(String),
    post_token_balance_amount Nullable(Float64),
    post_token_balance_program_id Nullable(String)
) ENGINE = MergeTree()
ORDER BY (tx_signature, account)
SETTINGS index_granularity = 8192;
