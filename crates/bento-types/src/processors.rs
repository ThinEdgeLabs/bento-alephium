use std::sync::Arc;

use crate::CustomProcessorOutput;

/// Common processor output enum
#[derive(Debug)]
pub enum ProcessorOutput {
    Block(Vec<crate::models::block::BlockModel>),
    Event(Vec<crate::models::event::EventModel>),
    Tx(Vec<crate::models::transaction::TransactionModel>),
    Custom(Arc<dyn CustomProcessorOutput>),
}
