use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::{config::ProcessorConfig, db::DbPool, types::BlockAndEvents};

use super::{ProcessorOutput, ProcessorTrait};

pub struct DefaultProcessor {
    connection_pool: Arc<DbPool>,
}

impl DefaultProcessor {
    pub fn new(connection_pool: Arc<DbPool>) -> Self {
        Self { connection_pool }
    }
}

impl Debug for DefaultProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "DefaultProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

#[async_trait]
impl ProcessorTrait for DefaultProcessor {
    type Output = ();
    fn name(&self) -> &'static str {
        ProcessorConfig::DefaultProcessor.name()
    }

    fn connection_pool(&self) -> &Arc<DbPool> {
        &self.connection_pool
    }

    async fn process_blocks(
        &self,
        _from: i64,
        _to: i64,
        _blocks: Vec<BlockAndEvents>,
    ) -> Result<Self::Output> {
        // Process blocks and events
        Ok(())
    }

    fn wrap_output(&self, output: Self::Output) -> ProcessorOutput {
        ProcessorOutput::Default(output)
    }
}
