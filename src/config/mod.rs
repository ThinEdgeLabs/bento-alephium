use std::sync::Arc;

use anyhow::{Context, Result};

use crate::{
    db::{new_db_pool, DbPool},
    processors::DynProcessor,
};

// Function type for processor factories
pub type ProcessorFactory = fn(Arc<DbPool>, Option<serde_json::Value>) -> DynProcessor;

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
            ProcessorConfig::BlockProcessor => crate::processors::new_processor(
                crate::processors::block_processor::BlockProcessor::new(db_pool),
            ),
            ProcessorConfig::EventProcessor => crate::processors::new_processor(
                crate::processors::event_processor::EventProcessor::new(db_pool),
            ),
            ProcessorConfig::TxProcessor => crate::processors::new_processor(
                crate::processors::tx_processor::TxProcessor::new(db_pool),
            ),
            ProcessorConfig::Custom { factory, args, .. } => factory(db_pool, args.clone()),
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub db_client: Arc<DbPool>,
    pub api_host: String,
    pub api_port: u16,
}

impl Config {
    pub async fn from_env() -> Result<Self> {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_client =
            new_db_pool(&db_url, None).await.context("Failed to create connection pool")?;

        let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
        let port = std::env::var("PORT").unwrap_or("3000".to_string());

        Ok(Self { db_client, api_host: host, api_port: port.parse().unwrap() })
    }

    pub fn api_endpoint(&self) -> String {
        format!("{}:{}", self.api_host, self.api_port)
    }
}
