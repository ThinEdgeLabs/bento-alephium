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
pub struct Pagination {
    /// The number of items to skip (optional)
    #[schema(default = 0)]
    pub offset: Option<String>,

    /// The maximum number of items to return (optional)
    #[schema(default = 10)]
    pub limit: Option<String>,
}

impl Pagination {
    // Helper method to parse offset with fallback to default
    pub fn get_offset(&self) -> i64 {
        self.offset.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(10)
    }

    // Helper method to parse limit with fallback to default
    pub fn get_limit(&self) -> i64 {
        self.limit.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(10)
    }
}
