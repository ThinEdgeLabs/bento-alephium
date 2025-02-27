use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

pub mod error;
pub mod handler;
pub mod index;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
}

// The query parameters for todos index
#[derive(Debug, Deserialize, ToSchema, Serialize, Default, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct Pagination {
    /// The number of items to skip (optional)
    #[param(style = Form, explode, example = json!(0), default=json!(0))]
    pub offset: i64,

    /// The maximum number of items to return (optional)
    #[param(style = Form, explode, allow_reserved, example = json!(10), default=json!(0))]
    pub limit: i64,
}

impl Pagination {
    // Helper method to parse offset with fallback to default
    pub fn get_offset(&self) -> i64 {
        self.offset
    }

    // Helper method to parse limit with fallback to default
    pub fn get_limit(&self) -> i64 {
        self.limit
    }
}
