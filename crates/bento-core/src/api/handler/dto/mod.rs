pub mod block;
pub mod event;
pub mod transaction;

pub use block::*;
pub use event::*;
pub use transaction::*;

pub struct Paginated<T> {
    pub data: Vec<T>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}
