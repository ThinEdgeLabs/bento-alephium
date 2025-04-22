use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = crate::schema::processor_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProcessorStatusModel {
    pub processor: String,
    pub last_timestamp: i64,
}
