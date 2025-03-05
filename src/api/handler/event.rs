use axum::extract::{Query, State};
use axum::Json;

use crate::api::error::AppError;
use crate::api::handler::dto::event::EventByContractQuery;
use crate::api::AppState;
use crate::api::Pagination;
use crate::models::event::EventModel;
use crate::repository::{get_events, get_events_by_contract, get_events_by_tx};
use axum::response::IntoResponse;
use serde::Serialize;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::dto::{EventByTxIdQuery, EventDto};
pub struct EventApiModule;

impl EventApiModule {
    pub fn register() -> OpenApiRouter<crate::api::AppState> {
        OpenApiRouter::new()
            .routes(routes!(get_events_handler))
            .routes(routes!(get_events_by_contract_handler))
            .routes(routes!(get_events_by_tx_id_handler))
    }
}

#[derive(Debug, IntoParams, ToSchema, Serialize)]
pub struct EventsQuery {
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[utoipa::path(get, path = "/",params(EventsQuery), tag = "Events", responses((status = OK, body = Vec<EventModel>)))]
pub async fn get_events_handler(
    pagination: Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<Vec<EventModel>>, AppError> {
    let db = state.db;

    let event_models = get_events(db, pagination.get_limit(), pagination.get_offset()).await?;
    Ok(Json(event_models))
}

#[utoipa::path(get, path = "/contract", params(EventByContractQuery),  tag = "Events", responses((status = OK, body = Vec<EventModel>)))]
pub async fn get_events_by_contract_handler(
    Query(query): Query<EventByContractQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<EventModel>>, AppError> {
    let EventByContractQuery { contract, pagination } = query;
    let db = state.db;
    let event_models = get_events_by_contract(
        db,
        contract.to_string(),
        pagination.get_limit(),
        pagination.get_offset(),
    )
    .await?;
    Ok(Json(event_models))
}

#[utoipa::path(get, path = "/tx", params(EventByTxIdQuery),  tag = "Events", responses((status = OK, body = EventModel)))]
pub async fn get_events_by_tx_id_handler(
    Query(query): Query<EventByTxIdQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db;

    let EventByTxIdQuery { tx_id, pagination } = query;
    let event_models =
        get_events_by_tx(db, tx_id.to_string(), pagination.get_limit(), pagination.get_offset())
            .await?;

    let events: Vec<EventDto> = event_models.into_iter().map(|event| event.into()).collect();
    Ok(Json(events))
}
