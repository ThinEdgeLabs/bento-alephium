use axum::extract::{Query, State};
use axum::Json;

use crate::api::error::AppError;
use crate::api::handler::dto::TransactionDto;
use crate::api::AppState;
use crate::api::Pagination;
use crate::repository::{get_tx_by_hash, get_txs};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

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
