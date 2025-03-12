use anyhow::{Context, Result};
use diesel::{insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    client::{Client, Network}, config::ProcessorConfig, db::{new_db_pool, DbPool}, models::{block::BlockModel, convert_bwe_to_block_models}, processors::{
        block_processor::BlockProcessor, default_processor::DefaultProcessor, event_processor::EventProcessor, lending_marketplace_processor::LendingContractProcessor, tx_processor::TxProcessor, Processor, ProcessorTrait
    }, repository::{get_block_by_hash, insert_blocks_to_db, update_main_chain}, schema::processor_status, traits::BlockProvider, types::REORG_TIMEOUT
};

/// Worker manages the lifecycle of a processor.
///
/// In the initialization phase, we make sure we get at least one timestamp other than the genesis one
///
/// The syncing algorithm goes as follow:
/// 1. Getting maximum timestamps from both the local chains and the remote ones.
/// 2. Build timestamp ranges of X minutes each, starting from local max to remote max.
/// 3. For each of those range, we get all the blocks inbetween that time.
/// 4. Insert all blocks (with `mainChain = false`).
/// 5. For each last block of each chains, mark it as part of the main chain and travel
///    down the parents recursively until we found back a parent that is part of the main chain.
/// 6. During step 5, if a parent is missing, we download it and continue the procces at 5.
///
/// TODO: Step 5 is costly, but it's an easy way to handle reorg. In step 3 we know we receive the current main chain
/// for that timerange, so in step 4 we could directly insert them as `mainChain = true`, but we need to sync
/// to a sanity check process, wich could be an external proccess, that regularly goes down the chain to make
/// sure we have the right one in DB.
pub struct Worker {
    pub db_pool: Arc<DbPool>,
    pub client: Arc<Client>,
    pub processor_config: ProcessorConfig,
    pub db_url: String,
    pub sync_opts: SyncOptions,
}



