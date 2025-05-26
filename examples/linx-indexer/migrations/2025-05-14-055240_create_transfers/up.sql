CREATE TABLE transfers (
    id SERIAL PRIMARY KEY,
    token_id TEXT NOT NULL,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    from_group SMALLINT NOT NULL,
    to_group SMALLINT NOT NULL,
    amount NUMERIC NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    tx_id TEXT NOT NULL
);

CREATE INDEX idx_transfers_from_address ON transfers(from_address);

CREATE UNIQUE INDEX unique_transfers ON transfers (tx_id, token_id, from_address, to_address, amount);