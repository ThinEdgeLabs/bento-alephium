use std::{fmt::Debug, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use bento_types::{processors::ProcessorOutput, BlockAndEvents, DbPool};
/// Base trait for all processors that includes both processing and storage
#[async_trait]
pub trait ProcessorTrait: Send + Sync + Debug + 'static {
    /// A unique name for this processor
    fn name(&self) -> &'static str;

    /// Access to the connection pool
    fn connection_pool(&self) -> &Arc<DbPool>;

    /// Process a batch of blocks and produce output
    async fn process_blocks(
        &self,
        from_ts: i64,
        to_ts: i64,
        blocks: Vec<BlockAndEvents>,
    ) -> Result<ProcessorOutput>;

    /// Store the processing output
    /// Default implementation for built-in processors. Custom processors need to override this method.
    async fn store_output(&self, output: ProcessorOutput) -> Result<()> {
        match output {
            ProcessorOutput::Block(blocks) => {
                if !blocks.is_empty() {
                    bento_types::repository::insert_blocks_to_db(
                        self.connection_pool().clone(),
                        blocks,
                    )
                    .await?;
                }
            }
            ProcessorOutput::Event(events) => {
                if !events.is_empty() {
                    bento_types::repository::insert_events_to_db(
                        self.connection_pool().clone(),
                        events,
                    )
                    .await?;
                }
            }
            ProcessorOutput::Tx(txs) => {
                if !txs.is_empty() {
                    bento_types::repository::insert_txs_to_db(self.connection_pool().clone(), txs)
                        .await?;
                }
            }
            ProcessorOutput::Custom(_) => {
                // Custom processors need to override this method to handle custom output
                tracing::warn!("Custom processor output with no storage implementation");
            }
        }
        Ok(())
    }
}

/// Trait for custom processor outputs
pub trait CustomProcessorOutput: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn CustomProcessorOutput>;
}

pub type DynProcessor = Box<dyn ProcessorTrait>;

pub fn new_processor(processor: impl ProcessorTrait) -> DynProcessor {
    Box::new(processor)
}
