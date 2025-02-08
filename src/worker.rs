use anyhow::{Context, Result};
use diesel::{insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use std::{convert, sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    client::{Client, Network},
    config::ProcessorConfig,
    db::{new_db_pool, DbPool},
    processors::{
        block_processor::BlockProcessor, default_processor::DefaultProcessor,
        event_processor::EventProcessor, lending_marketplace_processor::LendingContractProcessor,
        transaction_processor::TransactionProcessor, Processor, ProcessorTrait,
    },
    schema::{processor_status},
};

#[derive(Debug, Default)]
pub struct SyncOptions {
    pub start_ts: Option<i64>,
    pub step: Option<i64>,
    pub back_step: Option<i64>,
    pub sync_duration: Option<i64>,
}

/// Worker manages the lifecycle of a processor.
///
/// In the initialization phase, we make sure we get at least one timestamp other than the genesis one
///
/// The syncing algorithm goes as follow:
/// 1. Getting maximum timestamps from both the local chains and the remote ones.
/// 2. Build timestamp ranges of X minutes each, starting from local max to remote max.
/// 3. For each of those range, we get all the blocks inbetween that time.
/// 4. Insert all blocks (with `mainChain = false`).
/// 5. For each last block of each chains, mark it as part of the main chain and travel
///    down the parents recursively until we found back a parent that is part of the main chain.
/// 6. During step 5, if a parent is missing, we download it and continue the procces at 5.
///
/// TODO: Step 5 is costly, but it's an easy way to handle reorg. In step 3 we know we receive the current main chain
/// for that timerange, so in step 4 we could directly insert them as `mainChain = true`, but we need to sync
/// to a sanity check process, wich could be an external proccess, that regularly goes down the chain to make
/// sure we have the right one in DB.
pub struct Worker {
    pub db_pool: Arc<DbPool>,
    pub client: Arc<Client>,
    pub processor_config: ProcessorConfig,
    pub db_url: String,
    pub sync_opts: SyncOptions,
}

impl Worker {
    pub async fn new(
        processor_config: ProcessorConfig,
        db_url: String,
        network: Network,
        db_pool_size: Option<u32>,
        sync_opts: Option<SyncOptions>,
    ) -> Result<Self> {
        let processor_name = processor_config.name();
        tracing::info!(processor_name = processor_name, "Creating worker");

        tracing::info!(processor_name = processor_name, "Creating connection pool");
        let db_pool =
            new_db_pool(&db_url, db_pool_size).await.context("Failed to create connection pool")?;
        tracing::info!(processor_name = processor_name, "Finish creating the connection pool");

        let sync_opts = sync_opts.unwrap_or_default();

        Ok(Self {
            db_pool,
            processor_config,
            db_url,
            sync_opts,
            client: Arc::new(Client::new(network)),
        })
    }

    pub async fn run(&mut self) {
        let processor_name = self.processor_config.name();
        tracing::info!(processor_name = processor_name, "Starting worker");

        tracing::info!(processor_name = processor_name, "Run migrations");
        let migration_time = std::time::Instant::now();
        self.run_migrations().await;
        tracing::info!(
            processor_name = processor_name,
            duration_in_secs = migration_time.elapsed().as_secs_f64(),
            "Finished migrations"
        );

        // Initialize sync parameters
        let last_ts = get_last_timestamp(&self.db_pool, processor_name).await.unwrap();
        tracing::info!(processor_name = processor_name, last_ts = last_ts, "Got last timestamp");
        let mut current_ts = self.sync_opts.start_ts.unwrap_or(0);
        if current_ts < last_ts {
            current_ts = last_ts;
        }

        let step = self.sync_opts.step.unwrap_or(1000);
        let sync_duration = Duration::from_secs(self.sync_opts.sync_duration.unwrap_or(1) as u64);

        let processor = build_processor(&self.processor_config, self.db_pool.clone());

        loop {
            let to_ts = current_ts + step;

            tracing::info!(
                processor_name = processor_name,
                from_ts = current_ts,
                to_ts = to_ts,
                "Syncing blocks"
            );
            // Fetch blocks
            match self.client.get_blocks_and_events(current_ts as u128, to_ts as u128).await {
                Ok(blocks) => {
                    tracing::info!(
                        processor_name = processor_name,
                        block_count = blocks.blocks_and_events.len(),
                        "Found blocks"
                    );
                    if let Err(err) =
                        processor.process_blocks(current_ts, to_ts, blocks.blocks_and_events).await
                    {
                        tracing::error!(
                            processor_name = processor_name,
                            error = ?err,
                            "Error processing blocks, retrying in {:?}",
                            sync_duration
                        );
                        sleep(sync_duration).await;
                        continue;
                    }
                    update_last_timestamp(&self.db_pool, processor_name, to_ts).await.unwrap();
                    current_ts = to_ts + 1;
                }
                Err(err) => {
                    tracing::error!(
                        processor_name = processor_name,
                        error = ?err,
                        "Error fetching blocks, retrying in {:?}",
                        sync_duration
                    );
                    sleep(sync_duration).await;
                    continue;
                }
            }

            tracing::info!(processor_name = processor_name, "Sleeping for {:?}", sync_duration);
            sleep(sync_duration).await;
        }
    }

    // For the normal processor build we just use standard Diesel with the postgres
    // feature enabled (which uses libpq under the hood, hence why we named the feature
    // this way).
    #[cfg(feature = "libpq")]
    async fn run_migrations(&self) {
        use diesel::{pg::PgConnection, Connection};

        use crate::db::run_pending_migrations;

        tracing::info!("Running migrations: {:?}", self.db_url);
        let mut conn = PgConnection::establish(&self.db_url).expect("migrations failed!");
        run_pending_migrations(&mut conn);
    }
}

/// Build a processor based on the configuration.
pub fn build_processor(config: &ProcessorConfig, db_pool: Arc<DbPool>) -> Processor {
    match config {
        ProcessorConfig::DefaultProcessor => {
            Processor::DefaultProcessor(DefaultProcessor::new(db_pool))
        }
        ProcessorConfig::TransactionProcessor => {
            Processor::TransactionProcessor(TransactionProcessor::new(db_pool))
        }
        ProcessorConfig::BlockProcessor => Processor::BlockProcessor(BlockProcessor::new(db_pool)),
        ProcessorConfig::EventProcessor => Processor::EventProcessor(EventProcessor::new(db_pool)),
        ProcessorConfig::LendingContractProcessor(contract_address) => {
            Processor::LendingContractProcessor(LendingContractProcessor::new(
                db_pool,
                contract_address.clone(),
            ))
        }
    }
}

async fn get_last_timestamp(db_pool: &Arc<DbPool>, processor_name: &str) -> Result<i64> {
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

async fn update_last_timestamp(
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
        .map_err(|e| anyhow::Error::new(e))
}
