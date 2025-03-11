use crate::{traits::TransactionProvider, types::Transaction};
use anyhow::Result;
use url::Url;
use async_trait::async_trait;

use super::Client;
#[async_trait]
impl TransactionProvider for Client {
    /// Get transaction details by transaction ID.
    ///
    /// # Arguments
    ///
    /// * `tx_id` - The ID of the transaction to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Transaction` structure, or an error if the request fails.
     async fn get_tx_by_hash(&self, tx_id: &str) -> Result<Option<Transaction>> {
        let endpoint = format!("transactions/details/{}", tx_id);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    /// List transactions with pagination.
    /// GET:/blocks/{block_hash}/transactions?limit={limit}&offset={offset}
    async fn get_block_txs(&self, block_hash: String, limit: i64, offset: i64) -> Result<Vec<Transaction>> {
        let endpoint = format!("blocks/{block_hash}/transactions?limit={limit}&offset={offset}");
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }



}