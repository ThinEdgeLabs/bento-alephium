use std::sync::Arc;

use crate::db::DbPool;

use crate::schema::processor_status;
use anyhow::Result;
use diesel::{insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

pub async fn get_last_timestamp(db_pool: &Arc<DbPool>, processor_name: &str) -> Result<i64> {
    tracing::info!(processor = processor_name, "Getting last timestamp");
    let mut conn = db_pool.get().await?;
    let ts = processor_status::table
        .filter(processor_status::processor.eq(processor_name))
        .select(processor_status::last_timestamp)
        .first::<i64>(&mut conn)
        .await
        .optional()?;
    Ok(ts.unwrap_or(0))
}

pub async fn update_last_timestamp(
    _db_pool: &Arc<DbPool>,
    processor_name: &str,
    last_timestamp: i64,
) -> Result<()> {
    tracing::info!(
        processor = processor_name,
        last_timestamp = last_timestamp,
        "Updating last timestamp"
    );
    let mut conn = _db_pool.get().await?;
    insert_into(processor_status::table)
        .values((
            processor_status::processor.eq(processor_name),
            processor_status::last_timestamp.eq(last_timestamp),
        ))
        .on_conflict(processor_status::processor)
        .do_update()
        .set(processor_status::last_timestamp.eq(last_timestamp))
        .execute(&mut conn)
        .await
        .map(|_| ())
        .map_err(anyhow::Error::new)
}
