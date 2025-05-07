use std::sync::Arc;

use crate::DbPool;

use crate::schema::processor_status;
use anyhow::Result;
use diesel::{insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

pub async fn get_last_timestamp(
    db_pool: &Arc<DbPool>,
    processor_name: &str,
    network: String,
    is_backward: bool,
) -> Result<i64> {
    tracing::info!(processor = processor_name, "Getting last timestamp");

    // TODO: refactor this key construction to a function
    let mut procesor_key = String::from(processor_name);
    procesor_key.push('_');
    procesor_key.push_str(&network);

    if is_backward {
        procesor_key.push_str("_backward");
    }

    let mut conn = db_pool.get().await?;
    let ts = processor_status::table
        .filter(processor_status::processor.eq(procesor_key))
        .select(processor_status::last_timestamp)
        .first::<i64>(&mut conn)
        .await
        .optional()?;

    match ts {
        Some(ts) => {
            tracing::info!(processor = processor_name, last_timestamp = ts, "Found last timestamp");
            Ok(ts)
        }
        None => {
            tracing::info!(processor = processor_name, "No last timestamp found");
            Err(anyhow::anyhow!("No last timestamp found for processor: {}", processor_name))
        }
    }
}

pub async fn update_last_timestamp(
    _db_pool: &Arc<DbPool>,
    processor_name: &str,
    network: String,
    last_timestamp: i64,
    is_backward: bool,
) -> Result<()> {
    tracing::debug!(
        processor = processor_name,
        network = network,
        last_timestamp = last_timestamp,
        "Updating last timestamp"
    );

    // Construct the processor key
    let mut processor_key = String::from(processor_name);
    processor_key.push('_');
    processor_key.push_str(&network);

    if is_backward {
        processor_key.push_str("_backward");
    }

    let mut conn = _db_pool.get().await?;
    insert_into(processor_status::table)
        .values((
            processor_status::processor.eq(processor_key),
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
