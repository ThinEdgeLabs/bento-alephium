use std::sync::Arc;

use crate::{db::DbPool, models::event::EventModel};
use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

/// Insert events into the database.
pub async fn insert_events_to_db(db: Arc<DbPool>, events: Vec<EventModel>) -> Result<()> {
    let mut conn = db.get().await?;
    insert_into(crate::schema::events::table).values(&events).execute(&mut conn).await?;
    Ok(())
}

pub async fn get_events(db: Arc<DbPool>, limit: i64, offset: i64) -> Result<Vec<EventModel>> {
    use crate::schema::events::dsl::*;

    let mut conn = db.get().await?;

    let event_models: Vec<EventModel> =
        events.limit(limit).offset(offset).select(EventModel::as_select()).load(&mut conn).await?;

    Ok(event_models)
}

pub async fn get_events_by_contract(
    db: Arc<DbPool>,
    contract_address_value: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<EventModel>> {
    use crate::schema::events::dsl::*;

    let mut conn = db.get().await?;

    let event_models: Vec<EventModel> = events
        .filter(contract_address.eq(contract_address_value))
        .limit(limit)
        .offset(offset)
        .select(EventModel::as_select())
        .load(&mut conn)
        .await?;

    Ok(event_models)
}

pub async fn get_events_by_tx(
    db: Arc<DbPool>,
    tx_id_value: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<EventModel>> {
    use crate::schema::events::dsl::*;

    let mut conn = db.get().await?;

    let event_models: Vec<EventModel> = events
        .filter(tx_id.eq(tx_id_value))
        .limit(limit)
        .offset(offset)
        .select(EventModel::as_select())
        .load(&mut conn)
        .await?;

    Ok(event_models)
}
