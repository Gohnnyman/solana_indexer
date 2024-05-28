-- Your SQL goes here
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    slot INTEGER,
    transaction TEXT,
    block_time INTEGER,
    parsing_status INTEGER
);