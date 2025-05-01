use std::sync::Arc;

use anyhow::Result;
use bento_trait::{processor::DynProcessor, stage::StageHandler};
use bento_types::{
    processors::ProcessorOutput,
    repository::{insert_blocks_to_db, insert_events_to_db, insert_txs_to_db},
    DbPool, StageMessage,
};

pub struct ProcessorStage {
    pub processor: DynProcessor,
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
    pub db_pool: Arc<DbPool>,
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
