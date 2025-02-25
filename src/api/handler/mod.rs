use axum::Router;

use super::AppState;

pub mod block;
pub mod dto;
pub mod event;
pub mod transaction;

pub use block::BlockApiModule;
pub use event::EventApiModule;
pub use transaction::TransactionApiModule;
pub trait ApiModule {
    fn register() -> Router<AppState>;
}
