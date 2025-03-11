
use crate::{traits::BlockProvider, types::{
    BlockAndEvents, BlockEntry, BlockHeaderEntry, BlocksAndEventsPerTimestampRange,
    BlocksPerTimestampRange,
}};
use anyhow::Result;
use url::Url;
use async_trait::async_trait;

use super::Client;
#[async_trait]
impl BlockProvider for  Client {
    // List blocks on the given time interval.
    // GET:/blockflow/blocks?fromTs={from_ts}&toTs={to_ts}
    async fn get_blocks(&self, from_ts: u128, to_ts: u128) -> Result<BlocksPerTimestampRange> {
        let endpoint = format!("blockflow/blocks?fromTs={}&toTs={}", from_ts, to_ts);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    /// List blocks with events on the given time interval.
    ///
    /// # Arguments
    ///
    /// * `from_ts` - The starting timestamp for the block and event query.
    /// * `to_ts` - The ending timestamp for the block and event query.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `BlocksAndEventsPerTimestampRange` structure, or an error if the request fails.
    async fn get_blocks_and_events(
        &self,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<BlocksAndEventsPerTimestampRange> {
        let endpoint = format!("blockflow/blocks-with-events?fromTs={}&toTs={}", from_ts, to_ts);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    // Get a block with hash.
    // GET:/blockflow/blocks/{block_hash}
    async fn get_block(&self, block_hash: &String) -> Result<BlockEntry> {
        let endpoint = format!("blockflow/blocks/{}", block_hash);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    /// Get a block with events by its hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The hash of the block to retrieve along with events.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `BlockAndEvents` structure, or an error if the request fails.
    async fn get_block_and_events_by_hash(
        &self,
        block_hash: &String,
    ) -> Result<BlockAndEvents> {
        let endpoint = format!("blockflow/blocks-with-events/{}", block_hash);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    // Get block header.
    // GET:/blockflow/headers/{block_hash}
    async fn get_block_header(&self, block_hash: &String) -> Result<BlockHeaderEntry> {
        let endpoint = format!("blockflow/headers/{}", block_hash);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }
}