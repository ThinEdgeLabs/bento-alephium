pub mod block;
pub mod transaction;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest::Client as ReqwestClient;

use std::{env, time::Duration};
#[derive(Clone, Debug)]
pub enum Network {
    Development,
    Testnet,
    Mainnet,
    Custom(String),
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
    inner: reqwest_middleware::ClientWithMiddleware, // The inner HTTP client used for requests.
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

        let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(
            Duration::from_millis(100), // Minimum retry delay
            Duration::from_secs(10)     // Maximum retry delay
        )
        .build_with_max_retries(5);     // Maximum number of retries
    
    // Create client with retry middleware
    let client = ClientBuilder::new(ReqwestClient::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

        Self { inner: client, base_url: network.base_url() }
    }
}