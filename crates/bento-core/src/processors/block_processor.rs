use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bento_trait::processor::ProcessorTrait;
use bento_types::{
    convert_bwe_to_block_models, processors::ProcessorOutput, BlockAndEvents, BlockModel,
};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use crate::{config::ProcessorConfig, db::DbPool};

pub struct BlockProcessor {
    connection_pool: Arc<DbPool>,
}

impl BlockProcessor {
    pub fn new(connection_pool: Arc<DbPool>) -> Self {
        Self { connection_pool }
    }
}

impl Debug for BlockProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "BlockProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

#[async_trait]
impl ProcessorTrait for BlockProcessor {
    fn name(&self) -> &'static str {
        ProcessorConfig::BlockProcessor.name()
    }

    fn connection_pool(&self) -> &Arc<DbPool> {
        &self.connection_pool
    }

    fn get_processor(
        &self,
        pool: Arc<DbPool>,
        _args: Option<serde_json::Value>,
    ) -> Box<dyn ProcessorTrait> {
        Box::new(BlockProcessor::new(pool))
    }

    async fn process_blocks(
        &self,
        _from: i64,
        _to: i64,
        blocks: Vec<BlockAndEvents>,
    ) -> Result<ProcessorOutput> {
        let models = convert_bwe_to_block_models(blocks);
        Ok(ProcessorOutput::Block(models))
    }
}

/// Insert blocks into the database.
pub async fn insert_to_db(db: Arc<DbPool>, blocks: Vec<BlockModel>) -> Result<()> {
    let mut conn = db.get().await?;
    insert_into(bento_types::schema::blocks::dsl::blocks)
        .values(&blocks)
        .execute(&mut conn)
        .await?;
    Ok(())
}
