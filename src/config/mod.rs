use std::sync::Arc;

use anyhow::{Context, Result};

use crate::db::{new_db_pool, DbPool};
#[derive(Debug, Clone)]
pub enum ProcessorConfig {
    DefaultProcessor,
    BlockProcessor,
    EventProcessor,
    LendingContractProcessor(String),
    TxProcessor,
}

impl ProcessorConfig {
    pub fn name(&self) -> &'static str {
        match self {
            ProcessorConfig::DefaultProcessor => "default_processor",
            ProcessorConfig::BlockProcessor => "block_processor",
            ProcessorConfig::EventProcessor => "event_processor",
            ProcessorConfig::LendingContractProcessor(_) => "lending_contract_processor",
            ProcessorConfig::TxProcessor => "tx_processor",
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
