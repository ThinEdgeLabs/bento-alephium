use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{api::Pagination, models::transaction::TransactionModel};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionDto {
    pub tx_hash: String,
    pub unsigned: serde_json::Value,
    pub script_execution_ok: bool,
    pub contract_inputs: serde_json::Value,
    pub generated_outputs: serde_json::Value,
    pub input_signatures: Vec<Option<String>>,
    pub script_signatures: Vec<Option<String>>,
    pub block_hash: String,
}

impl From<TransactionModel> for TransactionDto {
    fn from(model: TransactionModel) -> Self {
        Self {
            tx_hash: model.tx_hash,
            unsigned: model.unsigned,
            script_execution_ok: model.script_execution_ok,
            contract_inputs: model.contract_inputs,
            generated_outputs: model.generated_outputs,
            input_signatures: model.input_signatures,
            script_signatures: model.script_signatures,
            block_hash: model.block_hash.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct TransactionHashQuery {
    /// The transaction hash to retrieve
    pub hash: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct TransactionsQuery {
    #[serde(flatten)]
    #[param(inline, example = json!({"offset": 0, "limit": 10}))]
    pub pagination: Pagination,
}
