-- Your SQL goes here
CREATE TABLE downloading_statuses (
    id SERIAL PRIMARY KEY,
    key VARCHAR(64) UNIQUE,
    downloading_status VARCHAR
);
