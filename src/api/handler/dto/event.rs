use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::api::Pagination;
#[derive(Debug, Deserialize, Default, IntoParams, ToSchema, Serialize)]
#[into_params(style = Form, parameter_in = Query)]
pub struct EventByContractQuery {
    /// The contract ID to filter events by
    pub contract: String,

    // Include the pagination fields
    #[param(inline)]
    #[serde(flatten)]
    pub pagination: Pagination,
}
