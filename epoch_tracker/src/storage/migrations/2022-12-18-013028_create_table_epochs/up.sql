CREATE TABLE IF NOT EXISTS epochs
(
    epoch INTEGER PRIMARY KEY,
    first_slot INTEGER DEFAULT NULL,
    last_slot INTEGER DEFAULT NULL,
    first_block INTEGER DEFAULT NULL,
    last_block INTEGER DEFAULT NULL,
    first_block_raw TEXT DEFAULT NULL,
    last_block_raw TEXT DEFAULT NULL,
    first_block_json JSONB DEFAULT NULL,
    last_block_json JSONB DEFAULT NULL,
    stakes JSONB DEFAULT NULL,
    rewards_parsing_status INTEGER DEFAULT 0
);
