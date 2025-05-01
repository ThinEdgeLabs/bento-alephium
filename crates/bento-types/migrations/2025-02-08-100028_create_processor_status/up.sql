-- Your SQL goes here
CREATE TABLE processor_status (
    processor VARCHAR(50) UNIQUE PRIMARY KEY NOT NULL,
    last_timestamp BIGINT NOT NULL
);