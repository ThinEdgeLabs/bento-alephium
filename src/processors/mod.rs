use crate::{
    db::{DbPool, DbPoolConnection},
    models::{block::BlockModel, event::EventModel, transaction::TransactionModel},
    types::BlockAndEvents,
};
use anyhow::Result;
use async_trait::async_trait;
use block_processor::BlockProcessor;
use default_processor::DefaultProcessor;
use diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection};
use event_processor::EventProcessor;
use lending_marketplace_processor::{LendingContractProcessor, LoanActionModel, LoanDetailModel};

use std::{fmt::Debug, sync::Arc};
use tx_processor::TxProcessor;

pub mod block_processor;
pub mod default_processor;
pub mod event_processor;
pub mod lending_marketplace_processor;
pub mod tx_processor;

// Define a variant for the processor return types
#[derive(Debug, Clone)]
pub enum ProcessorOutput {
    Block(Vec<BlockModel>),
    Event(Vec<EventModel>),
    LendingContract((Vec<LoanActionModel>, Vec<LoanDetailModel>)),
    Tx(Vec<TransactionModel>),
    Default(()),
}

/// Base trait for all processors
#[async_trait]
pub trait ProcessorTrait: Send + Sync + Debug {
    type Output;
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
        blocks: Vec<BlockAndEvents>,
    ) -> Result<Self::Output>;

    // Convert the native output to the common ProcessorOutput enum
    fn wrap_output(&self, output: Self::Output) -> ProcessorOutput;
}

#[derive(Debug)]
pub enum Processor {
    BlockProcessor(BlockProcessor),
    DefaultProcessor(DefaultProcessor),
    EventProcessor(EventProcessor),
    LendingContractProcessor(LendingContractProcessor),
    TxProcessor(TxProcessor),
}

impl Processor {
    pub fn connection_pool(&self) -> &Arc<DbPool> {
        match self {
            Processor::DefaultProcessor(p) => p.connection_pool(),
            Processor::BlockProcessor(p) => p.connection_pool(),
            Processor::EventProcessor(p) => p.connection_pool(),
            Processor::LendingContractProcessor(p) => p.connection_pool(),
            Processor::TxProcessor(p) => p.connection_pool(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Processor::DefaultProcessor(p) => p.name(),
            Processor::BlockProcessor(p) => p.name(),
            Processor::EventProcessor(p) => p.name(),
            Processor::LendingContractProcessor(p) => p.name(),
            Processor::TxProcessor(p) => p.name(),
        }
    }

    pub async fn process_blocks(
        &self,
        from_ts: i64,
        to_ts: i64,
        blocks: Vec<BlockAndEvents>,
    ) -> Result<ProcessorOutput> {
        match self {
            Processor::DefaultProcessor(p) => {
                p.process_blocks(from_ts, to_ts, blocks).await?;
                Ok(p.wrap_output(()))
            }
            Processor::BlockProcessor(p) => {
                let output = p.process_blocks(from_ts, to_ts, blocks).await?;
                Ok(p.wrap_output(output))
            }
            Processor::EventProcessor(p) => {
                let output = p.process_blocks(from_ts, to_ts, blocks).await?;
                Ok(p.wrap_output(output))
            }
            Processor::LendingContractProcessor(p) => {
                let output = p.process_blocks(from_ts, to_ts, blocks).await?;
                Ok(p.wrap_output(output))
            }
            Processor::TxProcessor(p) => {
                let output = p.process_blocks(from_ts, to_ts, blocks).await?;
                Ok(p.wrap_output(output))
            }
        }
    }
}
