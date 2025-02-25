use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::BlockModel;
use crate::{schema::transactions, types::BlockHash};
#[derive(
    Queryable,
    Selectable,
    Insertable,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    AsChangeset,
    Associations,
    Identifiable,
)]
#[diesel(table_name = transactions)]
#[diesel(primary_key(tx_hash))]
#[diesel(belongs_to(BlockModel, foreign_key = block_hash))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionModel {
    pub tx_hash: String,
    pub unsigned: serde_json::Value,
    pub script_execution_ok: bool,
    pub contract_inputs: serde_json::Value,
    pub generated_outputs: serde_json::Value,
    pub input_signatures: Vec<Option<String>>,
    pub script_signatures: Vec<Option<String>>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub block_hash: BlockHash,
}
