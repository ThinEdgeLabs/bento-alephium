-- This file should undo anything in `up.sql`
ALTER TABLE transactions 
DROP COLUMN block_hash;
