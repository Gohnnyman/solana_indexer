-- Your SQL goes here
CREATE TABLE signatures (
    id SERIAL PRIMARY KEY,
    signature VARCHAR(88) UNIQUE,
    slot INTEGER,
    err TEXT,
    memo TEXT,   
    block_time INTEGER,
    confirmation_status VARCHAR(16),
    loading_status INTEGER
);

CREATE INDEX signatures_loading_status_index ON signatures (loading_status);
