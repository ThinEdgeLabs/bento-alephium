use axum::extract::{Query, State};
use axum::Json;

use crate::api::error::AppError;
use crate::api::AppState;
use crate::api::Pagination;
use crate::models::transaction::TransactionModel;
use crate::repository::{get_tx_by_hash, get_txs};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionDto {
    pub tx_hash: String,
    pub unsigned: serde_json::Value,
    pub script_execution_ok: bool,
    pub contract_inputs: serde_json::Value,
    pub generated_outputs: serde_json::Value,
    pub input_signatures: Vec<Option<String>>,
    pub script_signatures: Vec<Option<String>>,
    #[schema(example = "2023-01-01T00:00:00", nullable = true)]
    pub created_at: Option<String>,
    #[schema(example = "2023-01-01T00:00:00", nullable = true)]
    pub updated_at: Option<String>,
    pub block_hash: String,
}

impl From<TransactionModel> for TransactionDto {
    fn from(model: TransactionModel) -> Self {
        Self {
            tx_hash: model.tx_hash,
            unsigned: model.unsigned,
            script_execution_ok: model.script_execution_ok,
            contract_inputs: model.contract_inputs,
            generated_outputs: model.generated_outputs,
            input_signatures: model.input_signatures,
            script_signatures: model.script_signatures,
            created_at: model.created_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()),
            updated_at: model.updated_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()),
            block_hash: model.block_hash.to_string(),
        }
    }
}

pub struct TransactionApiModule;

impl TransactionApiModule {
    pub fn register() -> OpenApiRouter<crate::api::AppState> {
        OpenApiRouter::new()
            .routes(routes!(get_txs_handler))
            .routes(routes!(get_tx_by_hash_handler))
    }
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct TransactionHashQuery {
    /// The transaction hash to retrieve
    pub hash: String,
}

#[utoipa::path(
    get,
    path = "/",
    tag = "Transactions",
    params(Pagination),
    responses(
        (status = 200, description = "List of transactions retrieved successfully", body = Vec<TransactionDto>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_txs_handler(
    pagination: Query<Pagination>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db;
    let tx_models = get_txs(db, pagination.get_limit(), pagination.get_offset()).await?;
    Ok(Json(tx_models))
}

#[utoipa::path(
    get,
    path = "/hash",
    tag = "Transactions",
    params(TransactionHashQuery),
    responses(
        (status = 200, description = "Transaction retrieved successfully", body = TransactionDto),
        (status = 404, description = "Transaction not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_tx_by_hash_handler(
    Query(query): Query<TransactionHashQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db;
    let tx_model = get_tx_by_hash(db, &query.hash).await?;
    Ok(Json(tx_model))
}
