-- Your SQL goes here
ALTER TABLE blocks ADD COLUMN parent VARCHAR(50) NOT NULL;
ALTER TABLE blocks ADD COLUMN main_chain BOOLEAN NOT NULL;