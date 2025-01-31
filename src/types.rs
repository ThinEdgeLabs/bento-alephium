use std::fmt;
use serde::{Deserialize, Serialize};

/// A wrapper type for representing a generic hash.
/// Implements `Display` to output the hash as a string.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Hash(pub String);

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A wrapper type for representing a block hash.
/// Implements `Display` to output the block hash as a string.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct BlockHash(pub String);

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the header of a block in a blockchain.
/// Contains information such as the hash, timestamp, chain range, and block height.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeaderEntry {
    pub hash: BlockHash,                // The hash of the block.
    pub timestamp: i64,                 // The timestamp of when the block was created.
    pub chain_from: i64,                // The start range of the chain.
    pub chain_to: i64,                  // The end range of the chain.
    pub height: i64,                    // The block height in the chain.
    pub deps: Vec<BlockHash>,           // The dependencies of the block.
}

/// Represents a complete block entry with transaction details.
/// Contains information such as transactions, nonce, and ghost uncles.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockEntry {
    pub hash: BlockHash,                // The hash of the block.
    pub timestamp: i64,                 // The timestamp when the block was created.
    pub chain_from: i64,                // The start range of the chain.
    pub chain_to: i64,                  // The end range of the chain.
    pub height: i64,                    // The block height.
    pub deps: Vec<BlockHash>,           // The block dependencies.
    pub transactions: Vec<Transaction>, // The list of transactions in this block.
    pub nonce: String,                  // A unique nonce for the block.
    pub version: i8,                    // The version of the block.
    pub dep_state_hash: Hash,           // The state hash of the block dependencies.
    pub txs_hash: Hash,                 // The hash of the transactions in the block.
    pub target: String,                 // The target block for the chain.
    pub ghost_uncles: Vec<GhostUncleBlockEntry>, // A list of ghost uncles related to the block.
}

/// Represents a ghost uncle block entry.
/// Contains information about the block hash and miner.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GhostUncleBlockEntry {
    pub block_hash: BlockHash, // The hash of the ghost uncle block.
    pub miner: String,         // The miner of the ghost uncle block.
}

/// Represents the collection of blocks grouped by timestamp ranges.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlocksPerTimestampRange {
    pub blocks: Vec<Vec<BlockEntry>>, // A list of block entries per timestamp range.
}

/// Represents a block along with its associated events.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockAndEvents {
    pub block: BlockEntry,                     // The block entry.
    pub events: Vec<ContractEventByBlockHash>, // The list of events associated with the block.
}

/// Represents a collection of blocks and their associated events, grouped by timestamp range.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlocksAndEventsPerTimestampRange {
    pub blocks_and_events: Vec<Vec<BlockAndEvents>>, // A list of blocks and events grouped by timestamp range.
}

/// Represents different types of values used in contract events.
#[derive(Deserialize, Debug)]
pub enum Val {
    Address { value: String, typ: String }, // Represents an address value.
    Array { value: Vec<Val>, typ: String }, // Represents an array of values.
    Bool { value: bool, typ: String },     // Represents a boolean value.
    ByteVec { value: String, typ: String }, // Represents a byte vector value.
    I256 { value: String, typ: String },    // Represents an I256 value.
    U256 { value: String, typ: String },    // Represents a U256 value.
}

/// Represents a contract event by block hash.
/// Contains the transaction ID, contract address, event index, and the event fields.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContractEventByBlockHash {
    pub tx_id: String,       // The transaction ID associated with the event.
    pub contract_address: String, // The contract address where the event was emitted.
    pub event_index: i32,    // The index of the event.
    pub fields: Vec<Val>,    // The fields of the event.
}

/// Represents an unsigned transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedTx {
    pub tx_id: String,         // The transaction ID.
    pub version: i32,          // The version of the transaction.
    pub network_id: i32,       // The network ID the transaction belongs to.
    pub script_opt: Option<String>, // Optional script for the transaction.
    pub gas_amount: i32,       // The gas amount used for the transaction.
    pub gas_price: String,     // The price of gas for the transaction.
    pub inputs: Vec<AssetInput>, // The inputs of the transaction.
    pub fixed_outputs: Vec<FixedAssetOutput>, // The fixed outputs of the transaction.
}

/// Represents a transaction in the blockchain system.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub unsigned: UnsignedTx,               // The unsigned transaction.
    pub script_execution_ok: bool,          // Whether the script execution was successful.
    pub contract_inputs: Vec<OutputRef>,    // The contract inputs associated with the transaction.
    pub generated_outputs: Vec<Output>,     // The outputs generated from the transaction.
    pub input_signatures: Vec<String>,      // The signatures of the inputs.
    pub script_signatures: Vec<String>,     // The script signatures for the transaction.
}

/// Represents a reference to an output in a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputRef {
    pub hint: i32,        // The hint associated with the output reference.
    pub key: String,      // The key for the output reference.
}

/// Represents an output in a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Output {}

/// Represents a contract output in a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContractOutput {
    pub hint: i32,            // The hint for the contract output.
    pub key: String,          // The key for the contract output.
    pub atto_alph_amount: String, // The amount of atto alph associated with the output.
    pub address: String,      // The address associated with the output.
    pub tokens: Vec<Token>,   // The list of tokens associated with the output.
    pub typ: String,          // The type of the contract output.
}

/// Represents an asset input in a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetInput {
    pub output_ref: OutputRef,    // The output reference for the asset input.
    pub unlock_script: String,    // The unlock script for the asset input.
}

/// Represents an asset output in a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetOutput {
    pub hint: i32,            // The hint for the asset output.
    pub key: String,          // The key for the asset output.
    pub atto_alph_amount: String, // The amount of atto alph associated with the output.
    pub address: String,      // The address associated with the output.
    pub tokens: Vec<Token>,   // The tokens associated with the output.
    pub lock_time: i64,       // The lock time for the asset output.
    pub message: String,      // The message for the asset output.
    pub typ: String,          // The type of the asset output.
}

/// Represents a fixed asset output in a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixedAssetOutput {
    pub hint: i32,            // The hint for the fixed asset output.
    pub key: String,          // The key for the fixed asset output.
    pub atto_alph_amount: String, // The amount of atto alph associated with the output.
    pub address: String,      // The address associated with the fixed asset output.
    pub tokens: Vec<Token>,   // The tokens associated with the output.
    pub lock_time: i64,       // The lock time for the fixed asset output.
    pub message: String,      // The message for the fixed asset output.
}

/// Represents a token in a transaction.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,      // The token ID.
    pub amount: String,  // The amount of the token.
}

/// Tests for the module to ensure correct deserialization and display functionality.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Tests the display functionality for Hash and BlockHash types.
    #[test]
    fn test_hash_display() {
        let hash =
            Hash("00000000000006f8c2bcaac93c5a23df8fba7119ba139d80a49d0303bbf84850".to_string());
        assert_eq!(
            format!("{}", hash),
            "00000000000006f8c2bcaac93c5a23df8fba7119ba139d80a49d0303bbf84850"
        );

        let block_hash = BlockHash(
            "00000000000006f8c2bcaac93c5a23df8fba7119ba139d80a49d0303bbf84850".to_string(),
        );
  assert_eq!(
            format!("{}", block_hash),
            "00000000000006f8c2bcaac93c5a23df8fba7119ba139d80a49d0303bbf84850"
        );
    }    #[test]
    fn test_block_entry_deser() {
        let json_data = json!({
            "hash": "00000000000006f8c2bcaac93c5a23df8fba7119ba139d80a49d0303bbf84850",
            "timestamp": 1672531200,
            "chainFrom": 1,
            "chainTo": 2,
            "height": 1000,
            "deps": ["hash1", "hash2"],
            "transactions": [],
            "nonce": "nonce_value",
            "version": 1,
            "depStateHash": "dep_hash",
            "txsHash": "txs_hash",
            "target": "target_value",
            "ghostUncles": [
                {
                    "blockHash": "unclehash1",
                    "miner": "miner1"
                }
            ]
        });

        let block: BlockEntry = serde_json::from_value(json_data).unwrap();

        assert_eq!(
            block.hash.0,
            "00000000000006f8c2bcaac93c5a23df8fba7119ba139d80a49d0303bbf84850"
        );
        assert_eq!(block.timestamp, 1672531200);
        assert_eq!(block.chain_from, 1);
        assert_eq!(block.chain_to, 2);
        assert_eq!(block.height, 1000);
        assert_eq!(block.deps.len(), 2);
        assert_eq!(block.deps[0].0, "hash1");
        assert_eq!(block.nonce, "nonce_value");
        assert_eq!(block.version, 1);
        assert_eq!(block.dep_state_hash.0, "dep_hash");
        assert_eq!(block.txs_hash.0, "txs_hash");
        assert_eq!(block.target, "target_value");
        assert_eq!(block.ghost_uncles.len(), 1);
        assert_eq!(block.ghost_uncles[0].block_hash.0, "unclehash1");
        assert_eq!(block.ghost_uncles[0].miner, "miner1");
    }

    #[test]
    fn test_transaction_deser() {
        let json_data = json!({
            "unsigned": {
                "txId": "tx123",
                "version": 1,
                "networkId": 42,
                "scriptOpt": "script",
                "gasAmount": 1000,
                "gasPrice": "1000000000",
                "inputs": [],
                "fixedOutputs": []
            },
            "scriptExecutionOk": true,
            "contractInputs": [],
            "generatedOutputs": [],
            "inputSignatures": [],
            "scriptSignatures": []
        });

        let transaction: Transaction = serde_json::from_value(json_data).unwrap();

        assert_eq!(transaction.unsigned.tx_id, "tx123");
        assert!(transaction.script_execution_ok);
        assert_eq!(transaction.unsigned.version, 1);
    }

    #[test]
    fn test_blocks_and_events_deser() {
        let json_data = json!({
            "blocksAndEvents": [
                [
                    {
                        "block": {
                            "hash": "blockhash123",
                            "timestamp": 1672531200,
                            "chainFrom": 1,
                            "chainTo": 2,
                            "height": 1000,
                            "deps": ["hash1", "hash2"],
                            "transactions": [],
                            "nonce": "nonce_value",
                            "version": 1,
                            "depStateHash": "dep_hash",
                            "txsHash": "txs_hash",
                            "target": "target_value",
                            "ghostUncles": []
                        },
                        "events": []
                    }
                ]
            ]
        });

        let blocks_and_events: BlocksAndEventsPerTimestampRange =
            serde_json::from_value(json_data).unwrap();

        assert_eq!(blocks_and_events.blocks_and_events.len(), 1);
        let block_and_event = &blocks_and_events.blocks_and_events[0][0];
        assert_eq!(block_and_event.block.hash.0, "blockhash123");
    }
}  
