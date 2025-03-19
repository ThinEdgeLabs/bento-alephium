use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

pub mod error;
pub mod handler;
pub mod index;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
}
use std::str::FromStr;

#[derive(Debug, Clone, Default, Deserialize, ToSchema, Serialize)]
pub struct Pagination {
    #[serde(
        default = "Pagination::default_offset",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub offset: i64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub limit: i64,
}

// Custom deserializer for string to i64 conversion
pub fn deserialize_number_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(0);
    }
    i64::from_str(&s).map_err(serde::de::Error::custom)
}

impl Pagination {
    pub fn get_offset(&self) -> i64 {
        if self.offset < 0 {
            return 0;
        }
        self.offset
    }

    pub fn get_limit(&self) -> i64 {
        if self.limit <= 0 || self.limit > 100 {
            return 10;
        }
        self.limit
    }

    pub fn default_offset() -> i64 {
        0
    }

    pub fn default_limit() -> i64 {
        10
    }
}
