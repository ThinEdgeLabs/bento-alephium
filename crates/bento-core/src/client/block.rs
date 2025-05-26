use std::time::Duration;

use backoff::{backoff::Backoff, ExponentialBackoff as BackoffExp};
use bento_trait::stage::BlockProvider;
use bento_types::{
    BlockAndEvents, BlockEntry, BlockHeaderEntry, BlocksAndEventsPerTimestampRange,
    BlocksPerTimestampRange,
};

use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use super::Client;
#[async_trait]
impl BlockProvider for Client {
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
    /// TODO: Fix hardcoded parameters in retry
    async fn get_blocks_and_events(
        &self,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<BlocksAndEventsPerTimestampRange> {
        // Using the rich-blocks endpoint to get complete tx input data
        let endpoint = format!("blockflow/rich-blocks?fromTs={}&toTs={}", from_ts, to_ts);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;

        // Configure backoff for deserialization retries
        let mut backoff = BackoffExp {
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(10),
            ..BackoffExp::default()
        };

        let mut attempt = 0;
        let max_attempts = 3;

        loop {
            attempt += 1;
            tracing::debug!(
                "Requesting blocks with events from: {} to: {} (attempt {}/{})",
                from_ts,
                to_ts,
                attempt,
                max_attempts
            );

            // The middleware will handle retries for the network request itself
            let response_result = self.inner.get(url.clone()).send().await;

            match response_result {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();

                        // For server errors, we might want to retry
                        if status.is_server_error() && attempt < max_attempts {
                            if let Some(duration) = backoff.next_backoff() {
                                tracing::warn!(
                                    "API returned error status: {}. Retrying in {:?}...",
                                    status,
                                    duration
                                );
                                tokio::time::sleep(duration).await;
                                continue;
                            }
                        }
                        return Err(anyhow::anyhow!("API returned error status: {}", status));
                    }

                    // Try to deserialize with retry on deserialization errors
                    match response.json::<BlocksAndEventsPerTimestampRange>().await {
                        Ok(data) => return Ok(data),
                        Err(e) => {
                            tracing::error!("Failed to deserialize response: {:?}", e);
                            tracing::error!("timestamp range: {} - {}", from_ts, to_ts);

                            // Retry deserialization errors if we have attempts left
                            if attempt < max_attempts {
                                if let Some(duration) = backoff.next_backoff() {
                                    tracing::warn!(
                                        "Deserialization failed. Retrying in {:?}...",
                                        duration
                                    );
                                    tokio::time::sleep(duration).await;
                                    continue;
                                }
                            }

                            return Err(anyhow::anyhow!("Error decoding response body: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    // The middleware should have already retried network errors,
                    // but if we get here, it means all retries failed
                    return Err(anyhow::anyhow!(
                        "Request failed after {} attempts: {:?}",
                        attempt,
                        e
                    ));
                }
            }
        }
    }

    // Get a block with hash.
    // GET:/blockflow/blocks/{block_hash}
    async fn get_block(&self, block_hash: &str) -> Result<BlockEntry> {
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
    async fn get_block_and_events_by_hash(&self, block_hash: &str) -> Result<BlockAndEvents> {
        let endpoint = format!("blockflow/blocks-with-events/{}", block_hash);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    // Get block header.
    // GET:/blockflow/headers/{block_hash}
    async fn get_block_header(&self, block_hash: &str) -> Result<BlockHeaderEntry> {
        let endpoint = format!("blockflow/headers/{}", block_hash);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }
}
