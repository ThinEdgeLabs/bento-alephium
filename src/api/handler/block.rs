use axum::extract::{Query, State};
use axum::Json;

use crate::api::error::AppError;
use crate::api::handler::transaction::TransactionDto;
use crate::api::Pagination;
use crate::models::block::BlockModel;
use crate::models::transaction::TransactionModel;
use crate::repository::{get_block_by_hash, get_block_by_height, get_block_transactions};
use crate::{api::AppState, repository::get_blocks};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

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
pub struct BlockApiModule;

impl BlockApiModule {
    pub fn register() -> OpenApiRouter<crate::api::AppState> {
        OpenApiRouter::new()
            .routes(routes!(get_blocks_handler))
            .routes(routes!(get_block_by_hash_handler))
            .routes(routes!(get_block_by_height_handler))
            .routes(routes!(get_block_transactions_handler))
    }
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct BlockHashQuery {
    /// The block hash to retrieve
    pub hash: String,
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct BlockHeightQuery {
    /// The block height to retrieve
    pub height: i64,
}

#[utoipa::path(
    get,
    path = "/",
    tag = "Blocks",
    params(Pagination),
    responses(
        (status = 200, description = "List of blocks retrieved successfully", body = Vec<BlockDto>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_blocks_handler(
    pagination: Query<Pagination>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db;
    let block_models = get_blocks(db, pagination.get_limit(), pagination.get_offset()).await?;
    Ok(Json(block_models))
}

#[utoipa::path(
    get,
    path = "/hash",
    tag = "Blocks",
    params(BlockHashQuery),
    responses(
        (status = 200, description = "Block retrieved successfully", body = BlockDto),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_block_by_hash_handler(
    Query(query): Query<BlockHashQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db;
    let block_model = get_block_by_hash(db, &query.hash).await?;
    Ok(Json(block_model))
}

#[utoipa::path(
    get,
    path = "/height",
    tag = "Blocks",
    params(BlockHeightQuery),
    responses(
        (status = 200, description = "Block retrieved successfully", body = BlockDto),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_block_by_height_handler(
    Query(query): Query<BlockHeightQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db;
    let block_model = get_block_by_height(db, query.height).await?;
    Ok(Json(block_model))
}

#[utoipa::path(
    get,
    path = "/transactions",
    tag = "Blocks",
    params(BlockHashQuery),
    responses(
        (status = 200, description = "Block transactions retrieved successfully", body = Vec<TransactionDto>),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_block_transactions_handler(
    Query(query): Query<BlockHashQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db;
    let transaction_models = get_block_transactions(db, query.hash).await?;
    Ok(Json(transaction_models))
}
