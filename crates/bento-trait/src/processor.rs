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
    async fn store_output(&self, output: ProcessorOutput) -> Result<()>;
}

pub type DynProcessor = Box<dyn ProcessorTrait>;

pub fn new_processor(processor: impl ProcessorTrait) -> DynProcessor {
    Box::new(processor)
}
