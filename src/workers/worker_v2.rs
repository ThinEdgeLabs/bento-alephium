use anyhow::{Context, Result};
use std::{sync::Arc, time::Duration};
use tokio::{sync::{mpsc}, time::sleep};
use futures::stream::{StreamExt, FuturesUnordered};
use diesel::{insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

use crate::{
    client::{Client, Network}, config::ProcessorConfig, db::{new_db_pool, DbPool}, processors::{
        block_processor::BlockProcessor, default_processor::DefaultProcessor, event_processor::EventProcessor, lending_marketplace_processor::{insert_loan_actions_to_db, insert_loan_details_to_db, LendingContractProcessor}, tx_processor::TxProcessor, Processor, ProcessorOutput, ProcessorTrait
    }, repository::{insert_blocks_to_db, insert_events_to_db, insert_txs_to_db}, schema::processor_status, traits::BlockProvider, types::BlockAndEvents
};

// Message types for different stages
#[derive(Clone)]
pub enum FetchStrategy {
    Simple,
    Chunked { chunk_size: i64 },
    Parallel { num_workers: usize },
}

#[derive(Clone)]
pub struct BlockRange {
    from_ts: i64,
    to_ts: i64,
}

#[derive(Clone)]
pub struct BlockBatch {
    blocks: Vec<BlockAndEvents>,
    range: BlockRange,
}

// Pipeline stage traits with message passing
#[async_trait::async_trait]
pub trait StageHandler: Send + 'static {
    async fn handle(&self, input: StageMessage) -> Result<StageMessage>;
}

#[derive(Clone)]
pub enum StageMessage {
    Range(BlockRange),
    Batch(BlockBatch),
    Processed(ProcessorOutput),
    Complete,
}

// Stage implementations
pub struct FetcherStage {
    client: Arc<Client>,
    strategy: FetchStrategy,
}

impl FetcherStage {
    pub fn new(client: Arc<Client>, strategy: FetchStrategy) -> Self {
        Self { client, strategy }
    }

    async fn fetch_chunk(&self, range: BlockRange) -> Result<BlockBatch> {
        let blocks = self.client
            .get_blocks_and_events(range.from_ts, range.to_ts)
            .await?
            .blocks_and_events.iter().flatten().cloned().collect();
        
        Ok(BlockBatch { blocks, range })
    }

    async fn fetch_parallel(&self, range: BlockRange, num_workers: usize) -> Result<Vec<BlockBatch>> {
        let total_time = range.to_ts - range.from_ts;
        let chunk_size = total_time / num_workers as i64;
        
        let mut futures = FuturesUnordered::new();
        
        for i in 0..num_workers {
            let from = range.from_ts + (i as i64 * chunk_size);
            let to = if i == num_workers - 1 {
                range.to_ts
            } else {
                from + chunk_size
            };
            
            let range = BlockRange { from_ts: from, to_ts: to };
            futures.push(self.fetch_chunk(range));
        }

        let mut results = Vec::new();
        while let Some(result) = futures.next().await {
            results.push(result?);
        }
        
        Ok(results)
    }
}

#[async_trait::async_trait]
impl StageHandler for FetcherStage {
    async fn handle(&self, input: StageMessage) -> Result<StageMessage> {
        match input {
            StageMessage::Range(range) => {
                match &self.strategy {
                    FetchStrategy::Simple => {
                        let batch = self.fetch_chunk(range).await?;
                        Ok(StageMessage::Batch(batch))
                    }
                    FetchStrategy::Chunked { chunk_size } => {
                        let total_time = range.to_ts - range.from_ts;
                        let num_chunks = (total_time / chunk_size) + 1;
                        let mut batches = Vec::new();
                        
                        for i in 0..num_chunks {
                            let from = range.from_ts + (i * chunk_size);
                            let to = (from + chunk_size).min(range.to_ts);
                            let chunk_range = BlockRange { from_ts: from, to_ts: to };
                            let batch = self.fetch_chunk(chunk_range).await?;
                            batches.push(batch);
                        }
                        
                        Ok(StageMessage::Batch(batches.remove(0))) // Send first batch
                    }
                    FetchStrategy::Parallel { num_workers } => {
                        let mut batches = self.fetch_parallel(range, *num_workers).await?;
                        Ok(StageMessage::Batch(batches.remove(0))) // Send first batch
                    }
                }
            }
            _ => Ok(StageMessage::Complete),
        }
    }
}

pub struct ProcessorStage {
    processor: Processor,
    db_pool: Arc<DbPool>,
}

#[async_trait::async_trait]
impl StageHandler for ProcessorStage {
    async fn handle(&self, input: StageMessage) -> Result<StageMessage> {
        match input {
            StageMessage::Batch(batch) => {
                
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

                match output {
                    ProcessorOutput::Block(blocks) => {
                        insert_blocks_to_db(self.db_pool.clone(), blocks).await?;
                    }
                    ProcessorOutput::Event(events) => {
                        insert_events_to_db(self.db_pool.clone(), events).await?;
                    }
                    ProcessorOutput::LendingContract((loan_actions, loan_details)) => {
                        insert_loan_actions_to_db(self.db_pool.clone(), loan_actions).await?;
                        insert_loan_details_to_db(self.db_pool.clone(), loan_details).await?;
                    }
                    ProcessorOutput::Tx(txs) => {
                        insert_txs_to_db(self.db_pool.clone(), txs).await?;
                    }
                    _ => unimplemented!(),
                }
                // if chrono::Utc::now().timestamp_millis() - block.timestamp <= REORG_TIMEOUT {
                //     update_main_chain(
                //         self.db_pool.clone(),
                //         block.hash,
                //         block.chain_from,
                //         block.chain_to,
                //         None,
                //     ).await?;
                // }
                Ok(StageMessage::Complete)
            }
            _ => Ok(StageMessage::Complete),
        }
    }
}

// Pipeline coordinator
pub struct Pipeline {
    fetcher: Arc<FetcherStage>,
    processor: Arc<ProcessorStage>,
    storage: Arc<StorageStage>,
}

impl Pipeline {
    pub fn new(
        client: Arc<Client>,
        db_pool: Arc<DbPool>,
        processor: Processor,
        fetch_strategy: FetchStrategy,
    ) -> Self {
        Self {
            fetcher: Arc::new(FetcherStage::new(client, fetch_strategy)),
            processor: Arc::new(ProcessorStage {
                processor,
                db_pool: db_pool.clone(),
            }),
            storage: Arc::new(StorageStage { db_pool }),
        }
    }

    pub async fn run(&self, range: BlockRange) -> Result<()> {
        let (fetch_tx, fetch_rx) = mpsc::channel(100);
        let (process_tx, process_rx) = mpsc::channel(100);
        let (storage_tx, storage_rx) = mpsc::channel(100);

        // Spawn stage handlers
        let fetcher = self.fetcher.clone();
        let processor = self.processor.clone();
        let storage = self.storage.clone();

        let fetch_handle = tokio::spawn(async move {
            let mut rx = fetch_rx;
            while let Some(msg) = rx.recv().await {
                let result = fetcher.handle(msg).await?;
                if let StageMessage::Complete = result {
                    break;
                }
                process_tx.send(result).await?;
            }
            Ok::<_, anyhow::Error>(())
        });

        let process_handle = tokio::spawn(async move {
            let mut rx = process_rx;
            while let Some(msg) = rx.recv().await {
                let result = processor.handle(msg).await?;
                if let StageMessage::Complete = result {
                    break;
                }
                storage_tx.send(result).await?;
            }
            Ok::<_, anyhow::Error>(())
        });

        let storage_handle = tokio::spawn(async move {
            let mut rx = storage_rx;
            while let Some(msg) = rx.recv().await {
                let result = storage.handle(msg).await?;
                if let StageMessage::Complete = result {
                    break;
                }
            }
            Ok::<_, anyhow::Error>(())
        });

        // Start the pipeline
        fetch_tx.send(StageMessage::Range(range)).await?;

        // Wait for all stages to complete
        tokio::try_join!(fetch_handle, process_handle, storage_handle)?;

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
        let db_pool = new_db_pool(&db_url, db_pool_size)
            .await
            .context("Failed to create connection pool")?;
        
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
            
            // Specify the error type explicitly with ::<(), anyhow::Error>
            let handle = tokio::spawn(async move {
                let processor = build_processor(&processor_config, pool_clone.clone());
                let processor_name = processor.name();
                
                let pipeline = Pipeline::new(
                    client_clone,
                    pool_clone.clone(),
                    processor,
                    fetch_strategy_clone,
                );
    
                let last_ts = get_last_timestamp(&pool_clone, processor_name).await?;
                let mut current_ts = sync_opts_clone.start_ts.unwrap_or(0).max(last_ts);
                let step = sync_opts_clone.step.unwrap_or(1000);
                let sync_duration = Duration::from_secs(
                    sync_opts_clone.sync_duration.unwrap_or(1) as u64
                );
    
                loop {
                    let to_ts = current_ts + step;
                    let range = BlockRange {
                        from_ts: current_ts,
                        to_ts,
                    };
    
                    if let Err(err) = pipeline.run(range).await {
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
    
                    sleep(sync_duration).await;
                }
                
                #[allow(unreachable_code)]
                Ok::<(), anyhow::Error>(())
            });
    
            handles.push(handle);
        }
    
        // Wait for all tasks to complete (they won't due to infinite loops)
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
        },
        ProcessorConfig::TxProcessor => Processor::TxProcessor(TxProcessor::new(db_pool)),
    }
}

#[derive(Debug, Default, Clone)]
pub struct SyncOptions {
    pub start_ts: Option<i64>,
    pub step: Option<i64>,
    pub back_step: Option<i64>,
    pub sync_duration: Option<i64>,
}
