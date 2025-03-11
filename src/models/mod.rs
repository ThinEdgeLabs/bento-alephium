use crate::types::BlockAndEvents;

pub mod block;
pub mod event;
pub mod processor_status;
pub mod transaction;

use block::BlockModel;
use event::EventModel;

pub fn convert_bwe_to_block_models(blocks: Vec<Vec<BlockAndEvents>>) -> Vec<BlockModel> {
    let mut models = Vec::new();
    for bes in blocks {
        for be in bes {
            let b = be.block;
            models.push(BlockModel {
                hash: b.hash,
                timestamp: crate::utils::timestamp_millis_to_naive_datetime(b.timestamp),
                chain_from: b.chain_from,
                chain_to: b.chain_to,
                height: b.height,
                deps: b.deps.iter().map(|x| Some(x.clone())).collect(),
                nonce: b.nonce,
                version: b.version.to_string(),
                dep_state_hash: b.dep_state_hash,
                txs_hash: b.txs_hash.to_string(),
                tx_number: b.transactions.len() as i64,
                target: b.target,
                main_chain: b.main_chain.unwrap_or(false),
                ghost_uncles: serde_json::to_value(b.ghost_uncles).unwrap_or_default(),
            });
        }
    }
    models
}

pub fn convert_bwe_to_event_models(blocks: Vec<Vec<BlockAndEvents>>) -> Vec<EventModel> {
    let mut models = Vec::new();
    for bes in blocks {
        for be in bes {
            for e in be.events {
                models.push(EventModel {
                    id: uuid::Uuid::new_v4().to_string(),
                    tx_id: e.tx_id,
                    contract_address: e.contract_address,
                    event_index: e.event_index,
                    fields: serde_json::to_value(e.fields).unwrap_or_default(), // TODO: need error handling here for retry?
                });
            }
        }
    }
    models
}
