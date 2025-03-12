use anyhow::Result;
use async_trait::async_trait;

use crate::types::{BlockAndEvents, BlockEntry, BlockHeaderEntry, BlocksAndEventsPerTimestampRange, BlocksPerTimestampRange, Transaction};
#[async_trait]
pub trait BlockProvider {
    // List blocks on the given time interval.
    async fn get_blocks(&self, from_ts: u128, to_ts: u128) -> Result<BlocksPerTimestampRange>;

    // List blocks with events on the given time interval.
    async fn get_blocks_and_events(
        &self,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<BlocksAndEventsPerTimestampRange>;

    // Get a block with hash.
    async fn get_block(&self, block_hash: &str) -> Result<BlockEntry>;

    // Get a block with events by its hash.
    async fn get_block_and_events_by_hash(
        &self,
        block_hash: &str,
    ) -> Result<BlockAndEvents>;

    async fn get_block_header(&self, block_hash: &str) -> Result<BlockHeaderEntry>;
}

#[async_trait]
pub trait TransactionProvider {
    async fn get_block_txs(&self, block_hash: String, limit: i64, offset: i64) -> Result<Vec<Transaction>>;
    async fn get_tx_by_hash(&self, tx_hash_value: &str) -> Result<Option<Transaction>>;
}

