use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::models::AccountTransactionDetails;
use crate::repository::AccountTransactionRepository;

use bento_server::AppState;
use bento_server::error::AppError;

pub struct AccountTransactionApiModule;

impl AccountTransactionApiModule {
    pub fn register() -> OpenApiRouter<AppState> {
        OpenApiRouter::new().routes(routes!(get_account_transactions_handler))
    }
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct AccountTransactionsQuery {
    pub address: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[utoipa::path(
    get,
    path = "/account-transactions",
    tag = "Account Transactions",
    params(AccountTransactionsQuery),
    responses(
        (status = 200, description = "List of account transactions retrieved successfully", body = Vec<AccountTransactionDetails>),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_account_transactions_handler(
    Query(query): Query<AccountTransactionsQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if query.limit <= 0 || query.limit > 100 {
        return Err(AppError::BadRequest("Limit must be between 1 and 100".to_string()));
    }

    if query.offset < 0 {
        return Err(AppError::BadRequest("Offset must be non-negative".to_string()));
    }

    let address = match query.address {
        Some(addr) if !addr.is_empty() => addr,
        Some(_) => {
            return Err(AppError::BadRequest("Address parameter cannot be empty".to_string()));
        }
        None => return Err(AppError::BadRequest("Missing parameter address".to_string())),
    };

    let account_tx_repo = AccountTransactionRepository::new(state.db.clone());

    let transactions =
        account_tx_repo.get_account_transactions(&address, query.limit, query.offset).await?;

    Ok(Json(transactions))
}
