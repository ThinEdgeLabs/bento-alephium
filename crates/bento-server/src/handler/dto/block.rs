use bento_types::{BlockModel, Order};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::Pagination;

#[derive(Debug, Serialize, ToSchema)]
pub struct BlockDto {
    pub hash: String,
    #[schema(example = "2023-01-01T00:00:00")]
    pub timestamp: String,
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

impl From<BlockModel> for BlockDto {
    fn from(model: BlockModel) -> Self {
        Self {
            hash: model.hash.to_string(),
            timestamp: model.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string(),
            chain_from: model.chain_from,
            chain_to: model.chain_to,
            height: model.height,
            deps: model.deps,
            nonce: model.nonce,
            version: model.version,
            dep_state_hash: model.dep_state_hash,
            txs_hash: model.txs_hash,
            tx_number: model.tx_number,
            target: model.target,
            main_chain: model.main_chain,
            ghost_uncles: model.ghost_uncles,
        }
    }
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct BlocksQuery {
    #[serde(flatten)]
    #[param(inline, example = json!({"offset": 0, "limit": 10}))]
    pub pagination: Pagination,

    #[param(inline)]
    pub order: Order,
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct BlockByHeightQuery {
    /// The block height to retrieve
    pub height: i64,
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct BlockByHashQuery {
    /// The block hash to retrieve
    pub hash: String,
}
