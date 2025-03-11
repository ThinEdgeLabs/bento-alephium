use crate::{
    db::{DbPool, DbPoolConnection},
    types::BlockAndEvents,
};
use anyhow::Result;
use async_trait::async_trait;
use block_processor::BlockProcessor;
use default_processor::DefaultProcessor;
use diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection};
use event_processor::EventProcessor;
use lending_marketplace_processor::LendingContractProcessor;
use tx_processor::TxProcessor;
use std::{fmt::Debug, sync::Arc};

pub mod block_processor;
pub mod default_processor;
pub mod event_processor;
pub mod lending_marketplace_processor;
pub mod tx_processor;

/// Base trait for all processors
#[async_trait]
pub trait ProcessorTrait: Send + Sync + Debug {
    fn name(&self) -> &'static str;

    fn connection_pool(&self) -> &Arc<DbPool>;

    fn get_pool(&self) -> Arc<Pool<AsyncPgConnection>> {
        self.connection_pool().clone()
    }

    async fn get_conn(&self) -> DbPoolConnection {
        let pool = self.connection_pool();
        loop {
            match pool.get().await {
                Ok(conn) => {
                    return conn;
                }
                Err(err) => {
                    tracing::error!(
                        "Could not get DB connection from pool, will retry. Err: {:?}",
                        err
                    );
                }
            };
        }
    }

    async fn process_blocks(
        &self,
        from_ts: i64,
        to_ts: i64,
        blocks: Vec<Vec<BlockAndEvents>>,
    ) -> Result<()>;
}

#[derive(Debug)]
pub enum Processor {
    BlockProcessor(BlockProcessor),
    DefaultProcessor(DefaultProcessor),
    EventProcessor(EventProcessor),
    LendingContractProcessor(LendingContractProcessor),
    TxProcessor(TxProcessor)
}

#[async_trait]
impl ProcessorTrait for Processor {
    fn connection_pool(&self) -> &Arc<DbPool> {
        match self {
            Processor::DefaultProcessor(p) => p.connection_pool(),
            Processor::BlockProcessor(p) => p.connection_pool(),
            Processor::EventProcessor(p) => p.connection_pool(),
            Processor::LendingContractProcessor(p) => p.connection_pool(),
            Processor::TxProcessor(p) => p.connection_pool()
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Processor::DefaultProcessor(p) => p.name(),
            Processor::BlockProcessor(p) => p.name(),
            Processor::EventProcessor(p) => p.name(),
            Processor::LendingContractProcessor(p) => p.name(),
            Processor::TxProcessor(p) => p.name()
        }
    }

    async fn process_blocks(
        &self,
        from_ts: i64,
        to_ts: i64,
        blocks: Vec<Vec<BlockAndEvents>>,
    ) -> Result<()> {
        match self {
            Processor::DefaultProcessor(p) => p.process_blocks(from_ts, to_ts, blocks).await,
            Processor::BlockProcessor(p) => p.process_blocks(from_ts, to_ts, blocks).await,
            Processor::EventProcessor(p) => p.process_blocks(from_ts, to_ts, blocks).await,
            Processor::LendingContractProcessor(p) => {
                p.process_blocks(from_ts, to_ts, blocks).await
            },
            Processor::TxProcessor(p) => p.process_blocks(from_ts, to_ts, blocks).await
        }
    }
}
