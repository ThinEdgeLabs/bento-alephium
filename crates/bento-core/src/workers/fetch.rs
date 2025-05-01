use std::{sync::Arc, time::Instant};

use anyhow::Result;
use futures::{stream::FuturesOrdered, StreamExt};

use crate::client::Client;

use bento_trait::stage::BlockProvider;

use bento_types::{BlockAndEvents, BlockBatch, BlockRange, MAX_TIMESTAMP_RANGE};

pub async fn fetch_parallel(
    client: Arc<Client>,
    range: BlockRange,
    num_workers: usize,
) -> Result<Vec<BlockBatch>> {
    let total_time: i64 = range.to_ts - range.from_ts;
    let chunk_size = total_time / num_workers as i64;

    tracing::info!(
        "Starting parallel fetch with {} workers for range {}-{}",
        num_workers,
        range.from_ts,
        range.to_ts
    );

    let mut futures = FuturesOrdered::new();

    for i in 0..num_workers {
        let from = range.from_ts + (i as i64 * chunk_size);
        let to = if i == num_workers - 1 { range.to_ts } else { from + chunk_size };

        let range = BlockRange { from_ts: from, to_ts: to };

        tracing::debug!(worker_id = i, from_ts = from, to_ts = to, "Dispatching worker");

        futures.push_back(fetch_chunk(client.clone(), range));
    }

    let mut results = Vec::new();
    let mut completed_workers = 0;

    while let Some(result) = futures.next().await {
        match result {
            Ok(batch) => {
                tracing::debug!(
                    worker_id = completed_workers,
                    batch_size = batch.blocks.len(),
                    "Worker completed successfully"
                );
                results.push(batch);
            }
            Err(err) => {
                let err_ctx =
                    format!("Failed to fetch chunk (worker {}/{})", completed_workers, num_workers);

                tracing::error!(
                    error = %err,
                    worker_id = completed_workers,
                    "Worker failed"
                );

                return Err(err.context(err_ctx));
            }
        }

        completed_workers += 1;
    }

    tracing::info!("Parallel fetch completed successfully, retrieved {} batches", results.len());

    Ok(results)
}

/// Fetch blocks in a given timestamp range
/// Will return an error if the timestamp range exceeds the maximum limit of the nodes
pub async fn fetch_chunk(client: Arc<Client>, range: BlockRange) -> Result<BlockBatch> {
    if (range.to_ts - range.from_ts) > MAX_TIMESTAMP_RANGE {
        return Err(anyhow::anyhow!(
            "Timestamp range exceeds maximum limit, maximum {}, got {}",
            MAX_TIMESTAMP_RANGE,
            range.to_ts - range.from_ts,
        ));
    }

    let start = Instant::now();
    let blocks: Vec<BlockAndEvents> = client
        .get_blocks_and_events(range.from_ts, range.to_ts)
        .await?
        .blocks_and_events
        .iter()
        .flatten()
        .cloned()
        .collect();

    let elapsed = start.elapsed();

    tracing::info!(
        "Fetched {} blocks from timestamp {} to timestamp {} in {:.2?}",
        blocks.clone().len(),
        range.from_ts,
        range.to_ts,
        elapsed
    );
    Ok(BlockBatch { blocks, range })
}
