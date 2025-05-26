use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{
    Selectable,
    prelude::{AsChangeset, Insertable, Queryable},
};
use serde::Serialize;

use crate::schema;

#[derive(Queryable, Selectable, Debug, Clone, Serialize, AsChangeset, PartialEq)]
#[diesel(table_name = schema::transfers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transfer {
    pub id: i32,
    pub token_id: String,
    pub from_address: String,
    pub to_address: String,
    pub from_group: i16,
    pub to_group: i16,
    pub amount: BigDecimal,
    pub timestamp: NaiveDateTime,
    pub tx_id: String,
}
#[derive(Insertable, Debug, Clone, Serialize, PartialEq)]
#[diesel(table_name = schema::transfers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransfer {
    pub token_id: String,
    pub from_address: String,
    pub to_address: String,
    pub from_group: i16,
    pub to_group: i16,
    pub amount: BigDecimal,
    pub timestamp: NaiveDateTime,
    pub tx_id: String,
}
