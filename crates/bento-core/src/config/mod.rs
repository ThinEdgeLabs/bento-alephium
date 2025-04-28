use std::sync::Arc;

use anyhow::{Context, Result};
use bento_trait::processor::{new_processor, DynProcessor, ProcessorTrait};

use crate::db::{new_db_pool, DbPool};

// Function type for processor factories
pub type ProcessorFactory = fn(Arc<DbPool>, Option<serde_json::Value>) -> Box<dyn ProcessorTrait>;

/// Extensible processor configuration with support for custom processors
#[derive(Debug, Clone)]
pub enum ProcessorConfig {
    /// Built-in processors
    BlockProcessor,
    EventProcessor,
    TxProcessor,

    /// Custom processors with arguments
    Custom {
        name: String,
        factory: ProcessorFactory,
        args: Option<serde_json::Value>,
    },
}

impl ProcessorConfig {
    pub fn name(&self) -> &str {
        match self {
            ProcessorConfig::BlockProcessor => "block_processor",
            ProcessorConfig::EventProcessor => "event_processor",
            ProcessorConfig::TxProcessor => "tx_processor",
            ProcessorConfig::Custom { name, .. } => name,
        }
    }

    /// Create a new custom processor config
    pub fn custom<S: Into<String>>(
        name: S,
        factory: ProcessorFactory,
        args: Option<serde_json::Value>,
    ) -> Self {
        Self::Custom { name: name.into(), factory, args }
    }

    /// Build a processor from this config
    pub fn build_processor(&self, db_pool: Arc<DbPool>) -> DynProcessor {
        match self {
            ProcessorConfig::BlockProcessor => {
                new_processor(crate::processors::block_processor::BlockProcessor::new(db_pool))
            }
            ProcessorConfig::EventProcessor => {
                new_processor(crate::processors::event_processor::EventProcessor::new(db_pool))
            }
            ProcessorConfig::TxProcessor => {
                new_processor(crate::processors::tx_processor::TxProcessor::new(db_pool))
            }
            ProcessorConfig::Custom { factory, args, .. } => factory(db_pool, args.clone()),
        }
    }
}
