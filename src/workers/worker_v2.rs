use crate::types::FetchStrategy;
use crate::types::{BlockBatch, BlockRange, StageMessage, MAX_TIMESTAMP_RANGE};
use crate::{
    client::{Client, Network},
    config::ProcessorConfig,
    db::{new_db_pool, DbPool},
    processors::{
        block_processor::BlockProcessor,
        default_processor::DefaultProcessor,
        event_processor::EventProcessor,
        lending_marketplace_processor::{
            insert_loan_actions_to_db, insert_loan_details_to_db, LendingContractProcessor,
        },
        tx_processor::TxProcessor,
        Processor, ProcessorOutput,
    },
    repository::{insert_blocks_to_db, insert_events_to_db, insert_txs_to_db},
    schema::processor_status,
};

use anyhow::{Context, Result};
use diesel::{insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

use crate::traits::StageHandler;
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::sleep as tokio_sleep};

use super::fetch::fetch_parallel;

pub struct ProcessorStage {
    processor: Processor,
}

#[async_trait::async_trait]
impl StageHandler for ProcessorStage {
    async fn handle(&self, input: StageMessage) -> Result<StageMessage> {
        match input {
            StageMessage::Batch(batch) => {
                let block_count = batch.blocks.len();
                let start = Instant::now();
                if block_count == 0 {
                    return Ok(StageMessage::Complete);
                }
                // Process blocks
                let output = self
                    .processor
                    .process_blocks(batch.range.from_ts, batch.range.to_ts, batch.blocks)
                    .await?;

                let elapsed = start.elapsed();

                tracing::info!(
                    "Processed {} blocks (range: {} to {}) in {:.2?}",
                    block_count,
                    batch.range.from_ts,
                    batch.range.to_ts,
                    elapsed
                );
                
                // Process blocks
                let output = self.processor.process_blocks(
                    batch.range.from_ts,
                    batch.range.to_ts,
                    batch.blocks,
                ).await?;
                
                Ok(StageMessage::Processed(output))
            }
            _ => Ok(StageMessage::Complete),
        }
    }
}
pub struct StorageStage {
    db_pool: Arc<DbPool>,
}

#[async_trait::async_trait]
impl StageHandler for StorageStage {
    async fn handle(&self, input: StageMessage) -> Result<StageMessage> {
        match input {
            StageMessage::Processed(output) => {
                let start = Instant::now();

                let result = match output {
                    ProcessorOutput::Block(blocks) => {
                        let count = blocks.len();
                        match insert_blocks_to_db(self.db_pool.clone(), blocks).await {
                            Ok(_) => {
                                let elapsed = start.elapsed();
                                tracing::info!(
                                    "Successfully stored {} blocks in {:.2?}",
                                    count,
                                    elapsed
                                );
                                Ok(())
                            }
                            Err(e) => {
                                tracing::error!("Failed to store blocks: {}", e);
                                Err(e)
                            }
                        }
                    }
                    ProcessorOutput::Event(events) => {
                        let count = events.len();
                        match insert_events_to_db(self.db_pool.clone(), events).await {
                            Ok(_) => {
                                let elapsed = start.elapsed();
                                tracing::info!(
                                    "Successfully stored {} events in {:.2?}",
                                    count,
                                    elapsed
                                );
                                Ok(())
                            }
                            Err(e) => {
                                tracing::error!("Failed to store events: {}", e);
                                Err(e)
                            }
                        }
                    }
                    ProcessorOutput::LendingContract((loan_actions, loan_details)) => {
                        let action_count = loan_actions.len();
                        let details_count = loan_details.len();
                        let store_start = Instant::now();

                        let actions_result =
                            insert_loan_actions_to_db(self.db_pool.clone(), loan_actions).await;
                        let actions_elapsed = store_start.elapsed();

                        let details_start = Instant::now();
                        let details_result =
                            insert_loan_details_to_db(self.db_pool.clone(), loan_details).await;
                        let details_elapsed = details_start.elapsed();

                        match (actions_result, details_result) {
                            (Ok(_), Ok(_)) => {
                                let total_elapsed = start.elapsed();
                                tracing::info!(
                                    "Successfully stored lending contract data: {} actions in {:.2?}, {} details in {:.2?}, total {:.2?}",
                                    action_count,
                                    actions_elapsed,
                                    details_count,
                                    details_elapsed,
                                    total_elapsed
                                );
                                Ok(())
                            }
                            (Err(e), _) | (_, Err(e)) => {
                                tracing::error!("Failed to store lending contract data: {}", e);
                                Err(e)
                            }
                        }
                    }
                    ProcessorOutput::Tx(txs) => {
                        let count = txs.len();
                        match insert_txs_to_db(self.db_pool.clone(), txs).await {
                            Ok(_) => {
                                let elapsed = start.elapsed();
                                tracing::info!(
                                    "Successfully stored {} transactions in {:.2?}",
                                    count,
                                    elapsed
                                );
                                Ok(())
                          }
                            Err(e) => {
                                tracing::error!("Failed to store transactions: {}", e);
                                Err(e)
                            }
                        }
                    }
                    ProcessorOutput::Default(()) => {
                        let elapsed = start.elapsed();
                        tracing::info!("Processed default output in {:.2?}", elapsed);
                        Ok(())
                    }
                };

                match result {
                    Ok(_) => Ok(StageMessage::Complete),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(StageMessage::Complete),
        }
    }
}

pub struct Pipeline {
    client: Arc<Client>,
    processor: Arc<ProcessorStage>,
    storage: Arc<StorageStage>,
}

impl Pipeline {
    pub fn new(
        client: Arc<Client>,
        db_pool: Arc<DbPool>,
        processor: Processor,
        fetch_strategy: FetchStrategy,
        sync_opts: SyncOptions,
    ) -> Self {
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
            let sync_opts_clone = self.sync_opts.clone();
            let processor_config = processor_config.clone();

            let handle = tokio::spawn(async move {
                let processor = build_processor(&processor_config, pool_clone.clone());
                let processor_name = processor.name();

                let pipeline = Pipeline::new(
                    client_clone.clone(),
                    pool_clone.clone(),
                    processor,
                    fetch_strategy_clone.clone(),
                    sync_opts_clone.clone(),
                );

                let last_ts = get_last_timestamp(&pool_clone, processor_name).await?;
                let mut current_ts = sync_opts_clone.start_ts.unwrap_or(0).max(last_ts);
                let step = sync_opts_clone.step.unwrap_or(1000);
                let sync_duration =
                    Duration::from_secs(sync_opts_clone.sync_duration.unwrap_or(1) as u64);

                loop {
                    let to_ts = current_ts + step;
                    let range = BlockRange { from_ts: current_ts, to_ts };
                    let batches = fetch_parallel(client_clone.clone(), range, fetch_strategy_clone.num_workers())
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
pub fn build_processor(config: &ProcessorConfig, db_pool: Arc<DbPool>) -> Processor {
    match config {
        ProcessorConfig::DefaultProcessor => {
            Processor::DefaultProcessor(DefaultProcessor::new(db_pool))
        }
        ProcessorConfig::BlockProcessor => Processor::BlockProcessor(BlockProcessor::new(db_pool)),
        ProcessorConfig::EventProcessor => Processor::EventProcessor(EventProcessor::new(db_pool)),
        ProcessorConfig::LendingContractProcessor(contract_address) => {
            Processor::LendingContractProcessor(LendingContractProcessor::new(
                db_pool,
                contract_address.clone(),
            ))
        }

        ProcessorConfig::TxProcessor => Processor::TxProcessor(TxProcessor::new(db_pool)),
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SyncOptions {
    pub start_ts: Option<i64>,
    pub step: Option<i64>,
    pub back_step: Option<i64>,
    pub sync_duration: Option<i64>,
}
