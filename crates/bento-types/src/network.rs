use std::env;

#[derive(Clone, Debug)]
pub enum Network {
    Devnet,
    Testnet,
    Mainnet,
    Custom(String, NetworkType),
}

#[derive(Clone, Debug)]
pub enum NetworkType {
    Devnet,
    Testnet,
    Mainnet,
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
            Network::Devnet => {
                env::var("DEV_NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:12973".to_owned())
            }
            Network::Testnet => env::var("TESTNET_NODE_URL")
                .unwrap_or_else(|_| "https://node.testnet.alephium.org".to_owned()),
            Network::Mainnet => env::var("MAINNET_NODE_URL")
                .unwrap_or_else(|_| "https://node.mainnet.alephium.org".to_owned()),
            Network::Custom(url, _) => url.clone(),
        }
    }

    pub fn identifier(&self) -> String {
        match self {
            Network::Devnet => "devnet".to_string(),
            Network::Testnet => "testnet".to_string(),
            Network::Mainnet => "mainnet".to_string(),
            Network::Custom(_, network_type) => network_type.to_string(),
        }
    }

    /// Creates a custom network with the specified URL and network type
    pub fn custom(url: &str, network_type: NetworkType) -> Self {
        Network::Custom(url.to_string(), network_type)
    }
}

impl NetworkType {
    pub fn to_string(&self) -> String {
        match self {
            NetworkType::Devnet => "devnet".to_string(),
            NetworkType::Testnet => "testnet".to_string(),
            NetworkType::Mainnet => "mainnet".to_string(),
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        env::var("ENVIRONMENT")
            .map(|env| match env.as_str() {
                "development" => Network::Devnet,
                "testnet" => Network::Testnet,
                "mainnet" => Network::Mainnet,
                _ => Network::Mainnet,
            })
            .unwrap_or(Network::Mainnet)
    }
}

impl From<Network> for String {
    fn from(value: Network) -> Self {
        match value {
            Network::Devnet => "devnet".to_string(),
            Network::Testnet => "testnet".to_string(),
            Network::Mainnet => "mainnet".to_string(),
            Network::Custom(_, network_type) => network_type.to_string(),
        }
    }
}

impl From<String> for Network {
    fn from(value: String) -> Self {
        match value.as_str() {
            "devnet" => Network::Devnet,
            "testnet" => Network::Testnet,
            "mainnet" => Network::Mainnet,
            _ => panic!("Invalid network type"),
        }
    }
}

impl From<String> for NetworkType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "devnet" => NetworkType::Devnet,
            "testnet" => NetworkType::Testnet,
            "mainnet" => NetworkType::Mainnet,
            _ => panic!("Invalid network type"),
        }
    }
}
