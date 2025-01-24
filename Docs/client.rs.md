# Client Documentation

## Overview
This provides detailed documentation of the public functions and modules within the Alephium Node Client. This client facilitates interaction with the Alephium blockchain network, allowing retrieval of blocks, transactions, and related data.

## Public Modules and Functions

| Function | Description | Return Type |
|----------|-------------|-------------|
| **new(network: Network) -> Self** | Creates a new client for the specified network. | `Client` |
| **get_blocks(from_ts: u64, to_ts: u64) -> Result<BlocksPerTimestampRange>** | Fetches a list of blocks within a given time range. | `Result<BlocksPerTimestampRange>` |
| **get_blocks_and_events(from_ts: u64, to_ts: u64) -> Result<BlocksAndEventsPerTimestampRange>** | Fetches blocks and their events within a given time range. | `Result<BlocksAndEventsPerTimestampRange>` |
| **get_block(block_hash: &BlockHash) -> Result<BlockEntry>** | Fetches details of a specific block by its hash. | `Result<BlockEntry>` |
| **get_block_and_events_by_hash(block_hash: &BlockHash) -> Result<BlockAndEvents>** | Fetches details of a block and its events by block hash. | `Result<BlockAndEvents>` |
| **get_block_header(block_hash: &BlockHash) -> Result<BlockHeaderEntry>** | Fetches the header of a specific block by its hash. | `Result<BlockHeaderEntry>` |
| **get_transaction(tx_id: &str) -> Result<Transaction>** | Fetches transaction details by transaction ID. | `Result<Transaction>` |

## Modules

### `Network`
Defines the available network environments for the Alephium blockchain.

- `Testnet` – Represents the Alephium testnet.
- `Mainnet` – Represents the Alephium mainnet.
- `Custom(String)` – Allows specifying a custom Alephium node URL.

#### Methods
| Method | Description | Return Type |
|----------|-------------|-------------|
| **base_url(&self) -> String** | Returns the base URL corresponding to the selected network. | `String` |

### `Client`
A struct representing a client for interacting with an Alephium blockchain node.

#### Fields
| Field | Description | Type |
|----------|-------------|-------------|
| **inner** | The underlying HTTP client (reqwest). | `reqwest::Client` |
| **base_url** | Base URL of the Alephium node. | `String` |

#### Default Implementation
- Defaults to `Mainnet` if no network is provided.


