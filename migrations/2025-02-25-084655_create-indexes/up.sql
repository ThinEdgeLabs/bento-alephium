-- Your SQL goes here

-- event table
CREATE INDEX idx_event_transaction_id ON events(tx_id);
CREATE INDEX idx_event_contract_address ON events(contract_address);

-- block table
CREATE INDEX idx_block_hash ON blocks(hash);
CREATE INDEX idx_block_chain_from ON blocks(chain_from);
CREATE INDEX idx_block_chain_to ON blocks(chain_to);
CREATE INDEX idx_block_height ON blocks(height);

-- transaction table
CREATE INDEX idx_transaction_block_height ON transactions(block_hash);

-- processor status table
CREATE INDEX idx_processor_status_name ON processor_status(processor);