use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]

/// Represents a block in the blockchain.
///
/// A block contains information related to its position within the blockchain, the transactions it contains,
/// and other associated metadata. The `Block` struct is used to interact with the `blocks` table in the database.
pub struct Block {
    pub hash: String,
    pub timestamp: NaiveDateTime,
    pub chain_from: i64,
    pub chain_to: i64,
    pub height: i64,
    pub deps: Option<Vec<Option<String>>>,
    pub nonce: String,
    pub version: String,
    pub dep_state_hash: String,
    pub txs_hash: String,
    pub tx_number: i64,
    pub target: String,
    pub main_chain: bool,
    pub hash_rate: BigDecimal,
    pub parent_hash: Option<String>,
    pub uncles: serde_json::Value,
}
/// Represents an event related to a blockchain transaction.
///
/// An event is tied to a specific transaction and contract address, with additional metadata in `fields` 
/// that can vary depending on the event's nature. The `Event` struct interacts with the `events` table.
#[derive(Queryable, Selectable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Event {
    pub tx_id: String,
    pub contract_address: String,
    pub event_index: i32,
    pub fields: serde_json::Value,
}
/// Represents a blockchain transaction.
///
/// A transaction includes metadata such as its hash, script execution status, input and output details,
/// and signatures associated with the transaction. The `Transaction` struct interacts with the `transactions` table.
#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub tx_hash: String,
    pub unsigned: serde_json::Value,
    pub script_execution_ok: bool,
    pub contract_inputs: serde_json::Value,
    pub generated_outputs: serde_json::Value,
    pub input_signatures: Vec<Option<String>>,
    pub script_signatures: Vec<Option<String>>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
