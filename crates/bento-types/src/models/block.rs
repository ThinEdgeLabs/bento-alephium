use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

use crate::{BlockHash, DEFAULT_GROUP_NUM};

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset, Identifiable)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(hash))]
pub struct BlockModel {
    pub hash: BlockHash,
    pub timestamp: NaiveDateTime,
    pub chain_from: i64,
    pub chain_to: i64,
    pub height: i64,
    pub deps: Vec<Option<String>>,
    pub nonce: String,
    pub version: String,
    pub dep_state_hash: String,
    pub txs_hash: String,
    pub tx_number: i64,
    pub target: String,
    pub main_chain: bool,
    pub ghost_uncles: serde_json::Value,
}

impl BlockModel {
    // fix the group number optionality
    pub fn parent(&self, group_num: Option<i64>) -> Option<BlockHash> {
        if self.height == 0 {
            None
        } else {
            Some(self.get_deps()[group_num.unwrap_or(DEFAULT_GROUP_NUM) as usize].clone())
        }
    }

    pub fn get_deps(&self) -> Vec<BlockHash> {
        self.deps.iter().map(|x| x.clone().unwrap()).collect()
    }
}
