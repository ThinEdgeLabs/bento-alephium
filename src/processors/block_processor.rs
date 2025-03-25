use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use crate::models::convert_bwe_to_block_models;
use crate::{
    config::ProcessorConfig, db::DbPool, models::block::BlockModel, types::BlockAndEvents,
};

use super::{ProcessorOutput, ProcessorTrait};

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
    insert_into(crate::schema::blocks::table).values(&blocks).execute(&mut conn).await?;
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::db::new_db_pool;
//     use crate::types::{BlockEntry, GhostUncleBlockEntry};
//     use crate::utils;
//     use diesel::QueryDsl;
//     use diesel::{ExpressionMethods, SelectableHelper};
//     use tokio;

//     async fn mock_db_pool() -> Arc<DbPool> {
//         let db_url = "postgres://postgres:123456789@localhost/bento_alephium_test";
//         new_db_pool(db_url, None).await.unwrap()
//     }

//     async fn cleanup_database(db_pool: &Arc<DbPool>) {
//         let conn = &mut db_pool.get().await.unwrap();
//         diesel::delete(crate::schema::blocks::table).execute(conn).await.unwrap();
//     }
//     fn create_mock_block_and_events(height: i64) -> BlockAndEvents {
//         BlockAndEvents {
//             block: BlockEntry {
//                 hash: format!("hash_{}", height),
//                 timestamp: 1672531200,
//                 chain_from: 1,
//                 chain_to: 2,
//                 height,
//                 deps: vec!["dep1".to_string(), "dep2".to_string()],
//                 transactions: vec![],
//                 nonce: "test_nonce".to_string(),
//                 version: 1,
//                 dep_state_hash: "dep_state_hash".to_string(),
//                 txs_hash: "txs_hash".to_string(),
//                 target: "target".to_string(),
//                 parent: "parent_hash".to_string(),
//                 main_chain: true,
//                 ghost_uncles: vec![GhostUncleBlockEntry {
//                     block_hash: "uncle_hash".to_string(),
//                     miner: "miner_address".to_string(),
//                 }],
//             },
//             events: vec![],
//         }
//     }

//     fn create_mock_block_model(height: i64) -> BlockModel {
//         BlockModel {
//             hash: format!("hash_{}", height),
//             timestamp: utils::timestamp_millis_to_naive_datetime(1672531200),
//             chain_from: 1,
//             chain_to: 2,
//             height,
//             deps: vec![Some("dep1".to_string()), Some("dep2".to_string())],
//             nonce: "test_nonce".to_string(),
//             version: "1".to_string(),
//             dep_state_hash: "dep_state_hash".to_string(),
//             txs_hash: "txs_hash".to_string(),
//             tx_number: 0,
//             target: "target".to_string(),
//             main_chain: true,
//             ghost_uncles: serde_json::json!([{
//                 "blockHash": "uncle_hash",
//                 "miner": "miner_address"
//             }]),
//         }
//     }

//     #[tokio::test]
//     async fn test_process_blocks_empty() {
//         let processor = BlockProcessor::new(mock_db_pool().await);

//         let result = processor.process_blocks(0, 100, vec![]).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_process_blocks_single_block() {
//         let db_pool = mock_db_pool().await;
//         let processor = BlockProcessor::new(db_pool.clone());

//         let mock_block = create_mock_block_and_events(1);
//         let other_blocks = vec![vec![mock_block]];

//         let result = processor.process_blocks(0, 100, other_blocks).await;
//         assert!(result.is_ok());

//         // Verify database insertion
//         let conn = &mut db_pool.get().await.unwrap();
//         let inserted_block = crate::schema::blocks::table
//             .filter(crate::schema::blocks::height.eq(1))
//             .select(BlockModel::as_select())
//             .first::<BlockModel>(conn)
//             .await
//             .unwrap();

//         assert_eq!(inserted_block.hash, "hash_1");
//         assert_eq!(inserted_block.height, 1);
//         assert_eq!(inserted_block.chain_from, 1);
//         assert_eq!(inserted_block.chain_to, 2);

//         cleanup_database(&db_pool).await;
//     }

//     #[tokio::test]
//     async fn test_process_blocks_multiple_blocks() {
//         let db_pool = mock_db_pool().await;
//         let processor = BlockProcessor::new(db_pool.clone());

//         let mock_blocks = vec![
//             vec![create_mock_block_and_events(1)],
//             vec![create_mock_block_and_events(2)],
//             vec![create_mock_block_and_events(3)],
//         ];

//         let result = processor.process_blocks(0, 100, mock_blocks).await;
//         assert!(result.is_ok());

//         // Verify database insertions
//         let conn = &mut db_pool.get().await.unwrap();
//         let inserted_blocks = crate::schema::blocks::table
//             .order_by(crate::schema::blocks::height.asc())
//             .execute(conn)
//             .await
//             .unwrap();

//         assert_eq!(inserted_blocks, 3);
//         cleanup_database(&db_pool).await;
//     }

//     #[tokio::test]
//     async fn test_process_blocks_with_deps() {
//         let db_pool = mock_db_pool().await;
//         let processor = BlockProcessor::new(db_pool.clone());

//         let mut mock_block = create_mock_block_and_events(1);
//         mock_block.block.deps = vec!["parent1".to_string(), "parent2".to_string()];

//         let result = processor.process_blocks(0, 100, vec![vec![mock_block]]).await;
//         assert!(result.is_ok());

//         // Verify block dependencies
//         let conn = &mut db_pool.get().await.unwrap();
//         let inserted_block = crate::schema::blocks::table
//             .filter(crate::schema::blocks::height.eq(1))
//             .select(BlockModel::as_select())
//             .first::<BlockModel>(conn)
//             .await
//             .unwrap();

//         assert_eq!(inserted_block.get_deps(), vec!["parent1", "parent2"]);
//         cleanup_database(&db_pool).await;
//     }

//     #[tokio::test]
//     async fn test_mass_block_insertion() {
//         use std::time::Instant;

//         let db_pool = mock_db_pool().await;
//         let processor = BlockProcessor::new(db_pool.clone());

//         // Create a large number of blocks
//         const BLOCK_COUNT: usize = 100;
//         const BATCH_SIZE: usize = 100;

//         let mut all_blocks = Vec::with_capacity(BLOCK_COUNT / BATCH_SIZE);

//         // Create blocks in batches to simulate real-world data structure
//         for batch_idx in 0..(BLOCK_COUNT / BATCH_SIZE) {
//             let mut batch = Vec::with_capacity(BATCH_SIZE);
//             for i in 0..BATCH_SIZE {
//                 let height = (batch_idx * BATCH_SIZE + i) as i64;
//                 batch.push(create_mock_block_and_events(height));
//             }
//             all_blocks.push(batch);
//         }

//         // Measure insertion time
//         let start = Instant::now();
//         processor.process_blocks(0, BLOCK_COUNT as i64, all_blocks).await.unwrap();
//         let duration = start.elapsed();

//         println!(
//             "Inserted {} blocks in {:?} ({} blocks/sec)",
//             BLOCK_COUNT,
//             duration,
//             BLOCK_COUNT as f64 / duration.as_secs_f64()
//         );

//         // Verify database state
//         let conn = &mut db_pool.get().await.unwrap();

//         // Count total blocks
//         let total_count: i64 = crate::schema::blocks::table.count().get_result(conn).await.unwrap();
//         assert_eq!(total_count, BLOCK_COUNT as i64);

//         // Verify block range
//         let min_height: i64 = crate::schema::blocks::table
//             .select(diesel::dsl::min(crate::schema::blocks::height))
//             .first::<Option<i64>>(conn)
//             .await
//             .unwrap()
//             .unwrap();

//         let max_height: i64 = crate::schema::blocks::table
//             .select(diesel::dsl::max(crate::schema::blocks::height))
//             .first::<Option<i64>>(conn)
//             .await
//             .unwrap()
//             .unwrap();

//         assert_eq!(min_height, 0);
//         assert_eq!(max_height, (BLOCK_COUNT - 1) as i64);

//         // Verify random samples for data integrity
//         let sample_heights = vec![
//             0,
//             BLOCK_COUNT as i64 / 4,
//             BLOCK_COUNT as i64 / 2,
//             3 * BLOCK_COUNT as i64 / 4,
//             BLOCK_COUNT as i64 - 1,
//         ];

//         for height in sample_heights {
//             let block = crate::schema::blocks::table
//                 .filter(crate::schema::blocks::height.eq(height))
//                 .select(BlockModel::as_select())
//                 .first(conn)
//                 .await
//                 .unwrap();

//             assert_eq!(block.hash, format!("hash_{}", height));
//             assert_eq!(block.height, height);
//             assert!(block.deps.len() >= 2);
//         }

//         cleanup_database(&db_pool).await;
//     }

//     #[tokio::test]
//     async fn test_block_model_parent() {
//         let block_model = create_mock_block_model(1);

//         // Test with default group number
//         let parent = block_model.parent(None);
//         assert_eq!(parent, Some("dep1".to_string()));

//         // Test with specific group number
//         let parent = block_model.parent(Some(1));
//         assert_eq!(parent, Some("dep2".to_string()));

//         // Test genesis block
//         let mut genesis_block = block_model;
//         genesis_block.height = 0;
//         assert_eq!(genesis_block.parent(None), None);
//     }
// }
