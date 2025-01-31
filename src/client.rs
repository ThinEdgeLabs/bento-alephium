use crate::types::{
    BlockAndEvents, BlockEntry, BlockHash, BlockHeaderEntry, BlocksAndEventsPerTimestampRange,
    BlocksPerTimestampRange, Transaction,
};
use anyhow::Result;
use std::env;
use url::Url;

/// Enum representing different networks for the client to interact with.
#[derive(Clone, Debug)]
pub enum Network {
    Development,  // Represents the development network.
    Testnet,     // Represents the testnet network.
    Mainnet,     // Represents the mainnet network.
    Custom(String), // Represents a custom network, specified by a URL.
}

impl Network {
    /// Returns the base URL for the network.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the network instance.
    ///
    /// # Returns
    ///
    /// A string containing the base URL of the network.
    pub fn base_url(&self) -> String {
        match self {
            Network::Development => {
                env::var("DEV_NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:12973".to_owned())
            }
            Network::Testnet => env::var("TESTNET_NODE_URL")
                .unwrap_or_else(|_| "https://node.testnet.alephium.org".to_owned()),
            Network::Mainnet => env::var("MAINNET_NODE_URL")
                .unwrap_or_else(|_| "https://node.mainnet.alephium.org".to_owned()),
            Network::Custom(url) => url.clone(),
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        env::var("ENVIRONMENT")
            .map(|env| match env.as_str() {
                "development" => Network::Development,
                "testnet" => Network::Testnet,
                "mainnet" => Network::Mainnet,
                _ => Network::Mainnet,
            })
            .unwrap_or(Network::Mainnet)
    }
}

/// Struct representing a client that interacts with the Alephium node network.
#[derive(Clone, Debug)]
pub struct Client {
    inner: reqwest::Client, // The inner HTTP client used for requests.
    base_url: String,       // The base URL for making requests to the node network.
}

impl Client {
    /// Creates a new `Client` instance for interacting with a specified network.
    ///
    /// # Arguments
    ///
    /// * `network` - The network to connect to.
    ///
    /// # Returns
    ///
    /// A new `Client` instance.
    pub fn new(network: Network) -> Self {
        Self { inner: reqwest::Client::new(), base_url: network.base_url() }
    }

    /// List blocks on the given time interval.
    ///
    /// # Arguments
    ///
    /// * `from_ts` - The starting timestamp for the block query.
    /// * `to_ts` - The ending timestamp for the block query.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `BlocksPerTimestampRange` structure, or an error if the request fails.
    pub async fn get_blocks(&self, from_ts: u64, to_ts: u64) -> Result<BlocksPerTimestampRange> {
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
    pub async fn get_blocks_and_events(
        &self,
        from_ts: u64,
        to_ts: u64,
    ) -> Result<BlocksAndEventsPerTimestampRange> {
        let endpoint = format!("blockflow/blocks-with-events?fromTs={}&toTs={}", from_ts, to_ts);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    /// Get a block by its hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `BlockEntry` structure, or an error if the request fails.
    pub async fn get_block(&self, block_hash: &BlockHash) -> Result<BlockEntry> {
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
    pub async fn get_block_and_events_by_hash(
        &self,
        block_hash: &BlockHash,
    ) -> Result<BlockAndEvents> {
        let endpoint = format!("blockflow/blocks-with-events/{}", block_hash);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    /// Get the header of a block by its hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The hash of the block to retrieve the header for.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `BlockHeaderEntry` structure, or an error if the request fails.
    pub async fn get_block_header(&self, block_hash: &BlockHash) -> Result<BlockHeaderEntry> {
        let endpoint = format!("blockflow/headers/{}", block_hash);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }

    /// Get transaction details by transaction ID.
    ///
    /// # Arguments
    ///
    /// * `tx_id` - The ID of the transaction to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Transaction` structure, or an error if the request fails.
    pub async fn get_transaction(&self, tx_id: &str) -> Result<Transaction> {
        let endpoint = format!("transactions/details/{}", tx_id);
        let url = Url::parse(&format!("{}/{}", self.base_url, endpoint))?;
        let response = self.inner.get(url).send().await?.json().await?;
        Ok(response)
    }
}
