use std::sync::Arc;

use axum::Router;

use crate::db::DbPool;

use super::AppState;

pub mod block;
pub mod event;
pub mod transaction;

pub use block::BlockApiModule;
pub use event::EventApiModule;
pub use transaction::TransactionApiModule;
pub trait ApiModule {
    fn register() -> Router<AppState>;
}
