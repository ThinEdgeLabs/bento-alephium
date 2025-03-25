use crate::types::FetchStrategy;
use crate::types::{BlockBatch, BlockRange, StageMessage};
use crate::{
    client::{Client, Network},
    config::ProcessorConfig,
    db::{new_db_pool, DbPool},
    processors::{
        block_processor::BlockProcessor, event_processor::EventProcessor,
        lending_marketplace_processor::LendingContractProcessor, new_processor,
        tx_processor::TxProcessor, DynProcessor, ProcessorOutput,
    },
    repository::{insert_blocks_to_db, insert_events_to_db, insert_txs_to_db},
    schema::processor_status,
};

use anyhow::{Context, Result};
use diesel::{insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

use crate::traits::StageHandler;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::sleep as tokio_sleep};

use super::fetch::fetch_parallel;

pub struct ProcessorStage {
    processor: DynProcessor,
}

impl ProcessorStage {
    pub fn new(processor: DynProcessor) -> Self {
        Self { processor }
    }
}

#[async_trait::async_trait]
impl StageHandler for ProcessorStage {
    async fn handle(&self, msg: StageMessage) -> Result<StageMessage> {
        match msg {
            StageMessage::Batch(batch) => {
                let output = self
                    .processor
                    .process_blocks(batch.range.from_ts, batch.range.to_ts, batch.blocks)
                    .await?;
                Ok(StageMessage::Processed(output))
            }
            _ => Ok(msg),
        }
    }
}

pub struct StorageStage {
    db_pool: Arc<DbPool>,
}

impl StorageStage {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl StageHandler for StorageStage {
    async fn handle(&self, msg: StageMessage) -> Result<StageMessage> {
        match msg {
            StageMessage::Processed(output) => {
                match output {
                    ProcessorOutput::Block(blocks) => {
                        if !blocks.is_empty() {
                            insert_blocks_to_db(self.db_pool.clone(), blocks).await?;
                        }
                    }
                    ProcessorOutput::Event(events) => {
                        if !events.is_empty() {
                            insert_events_to_db(self.db_pool.clone(), events).await?;
                        }
                    }
                    ProcessorOutput::Tx(txs) => {
                        if !txs.is_empty() {
                            insert_txs_to_db(self.db_pool.clone(), txs).await?;
                        }
                    }
                    ProcessorOutput::Custom(_) => {
                        // Custom processor outputs need to handle their own storage
                        tracing::info!(
                            "Custom processor output received - storage handled by processor"
                        );
                    }
                }
                Ok(StageMessage::Complete)
            }
            _ => Ok(msg),
        }
    }
}

#[allow(dead_code)]
pub struct Pipeline {
    client: Arc<Client>,
    processor: Arc<ProcessorStage>,
    storage: Arc<StorageStage>,
}

impl Pipeline {
    pub fn new(client: Arc<Client>, db_pool: Arc<DbPool>, processor: DynProcessor) -> Self {
        Self {
            client,
            processor: Arc::new(ProcessorStage { processor }),
            storage: Arc::new(StorageStage { db_pool }),
        }
    }

    pub async fn run(&self, batches: Vec<BlockBatch>) -> Result<()> {
        let channel_capacity = 100;
        let (process_tx, process_rx) = mpsc::channel(channel_capacity);
        let (storage_tx, storage_rx) = mpsc::channel(channel_capacity);

        // Send the fetched batches to the processor
        for batch in batches {
            process_tx.send(StageMessage::Batch(batch)).await?;
        }

        drop(process_tx);

        // Spawn stage handlers
        let processor = self.processor.clone();
        let storage = self.storage.clone();

        // Processor stage
        let process_handle = tokio::spawn(async move {
            let mut rx = process_rx;

            while let Some(msg) = rx.recv().await {
                if let StageMessage::Batch(batch) = msg {
                    let blocks_count = batch.blocks.len();
                    let range = batch.range;

                    tracing::info!(
                        "Processor processing batch with {} blocks (range: {} to {})",
                        blocks_count,
                        range.from_ts,
                        range.to_ts
                    );

                    let result = processor.handle(StageMessage::Batch(batch)).await?;

                    if let StageMessage::Processed(output) = result {
                        storage_tx.send(StageMessage::Processed(output)).await?;
                    }
                }
            }

            // Close storage channel when processor is done
            drop(storage_tx);

            Ok::<_, anyhow::Error>(())
        });

        // Storage stage
        let storage_handle = tokio::spawn(async move {
            let mut rx = storage_rx;

            while let Some(msg) = rx.recv().await {
                if let StageMessage::Processed(output) = msg {
                    storage.handle(StageMessage::Processed(output)).await?;
                }
            }

            Ok::<_, anyhow::Error>(())
        });

        let (process_result, storage_result) = tokio::join!(process_handle, storage_handle);

        process_result??;
        storage_result??;

        tracing::info!("Pipeline execution completed successfully");
        Ok(())
    }
}

pub struct Worker {
    pub db_pool: Arc<DbPool>,
    pub client: Arc<Client>,
    pub processor_configs: Vec<ProcessorConfig>,
    pub db_url: String,
    pub sync_opts: SyncOptions,
    pub fetch_strategy: FetchStrategy,
}

impl Worker {
    pub async fn new(
        processor_configs: Vec<ProcessorConfig>,
        db_url: String,
        network: Network,
        db_pool_size: Option<u32>,
        sync_opts: Option<SyncOptions>,
        fetch_strategy: Option<FetchStrategy>,
    ) -> Result<Self> {
        let db_pool =
            new_db_pool(&db_url, db_pool_size).await.context("Failed to create connection pool")?;

        Ok(Self {
            db_pool: db_pool.clone(),
            processor_configs,
            db_url,
            sync_opts: sync_opts.unwrap_or_default(),
            client: Arc::new(Client::new(network)),
            fetch_strategy: fetch_strategy.unwrap_or(FetchStrategy::Simple),
        })
    }

    pub async fn run(&self) -> Result<()> {
        self.run_migrations().await;
        let mut handles = Vec::new();

        for processor_config in self.processor_configs.clone() {
            let pool_clone = self.db_pool.clone();
            let client_clone = self.client.clone();
            let fetch_strategy_clone = self.fetch_strategy.clone();
            let sync_opts_clone = self.sync_opts;
            let processor_config = processor_config.clone();

            let handle = tokio::spawn(async move {
                let processor = build_processor(&processor_config, pool_clone.clone());
                let processor_name = processor.name();

                let pipeline = Pipeline::new(client_clone.clone(), pool_clone.clone(), processor);

                let last_ts = get_last_timestamp(&pool_clone, processor_name).await?;
                let mut current_ts = sync_opts_clone.start_ts.unwrap_or(0).max(last_ts);
                let step = sync_opts_clone.step.unwrap_or(1000);
                let sync_duration =
                    Duration::from_secs(sync_opts_clone.sync_duration.unwrap_or(1) as u64);

                loop {
                    let to_ts = current_ts + step;
                    let range = BlockRange { from_ts: current_ts, to_ts };
                    let batches = fetch_parallel(
                        client_clone.clone(),
                        range,
                        fetch_strategy_clone.num_workers(),
                    )
                    .await?;
                    if let Err(err) = pipeline.run(batches).await {
                        tracing::error!(
                            processor_name = processor_name,
                            error = ?err,
                            "Pipeline execution failed, retrying in {:?}",
                            sync_duration
                        );
                    } else {
                        update_last_timestamp(&pool_clone, processor_name, to_ts).await?;
                        current_ts = to_ts + 1;
                    }

                    tokio_sleep(sync_duration).await;
                }

                #[allow(unreachable_code)]
                Ok::<(), anyhow::Error>(())
            });

            handles.push(handle);
        }

        // Start all handlers by infinite loop.
        for handle in handles {
            match handle.await {
                Ok(result) => result?,
                Err(e) => return Err(anyhow::anyhow!("Task panicked: {}", e)),
            }
        }

        Ok(())
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

/// Build a processor based on the configuration.
pub fn build_processor(config: &ProcessorConfig, db_pool: Arc<DbPool>) -> DynProcessor {
    match config {
        ProcessorConfig::BlockProcessor => new_processor(BlockProcessor::new(db_pool)),
        ProcessorConfig::EventProcessor => new_processor(EventProcessor::new(db_pool)),
        ProcessorConfig::LendingContractProcessor(contract_address) => {
            new_processor(LendingContractProcessor::new(db_pool, contract_address.clone()))
        }
        ProcessorConfig::TxProcessor => new_processor(TxProcessor::new(db_pool)),
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SyncOptions {
    pub start_ts: Option<i64>,
    pub step: Option<i64>,
    pub back_step: Option<i64>,
    pub sync_duration: Option<i64>,
}
