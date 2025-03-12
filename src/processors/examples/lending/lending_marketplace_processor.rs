use std::fmt::Debug;
use std::sync::Arc;

use crate::config::{ProcessorConfig, ProcessorConfigTrait};
use crate::processors::ProcessorTrait;
use crate::types::ContractEventByBlockHash;
use crate::utils::timestamp_millis_to_naive_datetime;
use crate::{db::DbPool, types::BlockAndEvents};
use anyhow::Result;
use async_trait::async_trait;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::NaiveDateTime;
use diesel::expression::AsExpression;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::SmallInt;
use diesel_async::RunQueryDsl;
use diesel_enum::DbEnum;
use serde::Serialize;

use diesel::FromSqlRow;

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = crate::schema::loan_actions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LoanActionModel {
    loan_subcontract_id: String,
    loan_id: Option<BigDecimal>,
    by: String,
    timestamp: NaiveDateTime,
    action_type: LoanActionType,
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = crate::schema::loan_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LoanDetailModel {
    loan_subcontract_id: String,
    lending_token_id: String,
    collateral_token_id: String,
    lending_amount: BigDecimal,
    collateral_amount: BigDecimal,
    interest_rate: BigDecimal,
    duration: BigDecimal,
    lender: String,
}

pub struct LendingContractProcessor {
    connection_pool: Arc<DbPool>,
    config: ProcessorConfig,
}

impl LendingContractProcessor {
    pub fn new(connection_pool: Arc<DbPool>, config: &ProcessorConfig) -> Self {
        Self { connection_pool, config: config.clone() }
    }
}

impl Debug for LendingContractProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "LoanActionProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

#[async_trait]
impl ProcessorTrait for LendingContractProcessor {
    fn name(&self) -> &'static str {
        self.config.name()
    }

    fn connection_pool(&self) -> &Arc<DbPool> {
        &self.connection_pool
    }

    // Process blocks and insert to db
    async fn process_blocks(
        &self,
        _from: i64,
        _to: i64,
        blocks: Vec<BlockAndEvents>,
    ) -> Result<()> {
        // Sanity check
        if !self.config.is_lending_contract_processor() {
            return Err(anyhow::anyhow!("Invalid processor type"));
        }

        let (loan_actions, loan_details) =
            convert_to_model(blocks, &self.config.get_lending_contract_address().unwrap());
        if !loan_actions.is_empty() {
            insert_loan_actions_to_db(self.connection_pool.clone(), loan_actions).await?;
        }
        if !loan_details.is_empty() {
            insert_loan_details_to_db(self.connection_pool.clone(), loan_details).await?;
        }
        Ok(())
    }
}

/// Insert loan actions into the database.
pub async fn insert_loan_actions_to_db(
    db: Arc<DbPool>,
    actions: Vec<LoanActionModel>,
) -> Result<()> {
    let mut conn = db.get().await?;
    insert_into(crate::schema::loan_actions::table).values(&actions).execute(&mut conn).await?;
    Ok(())
}

/// Insert loan details into the database.
pub async fn insert_loan_details_to_db(
    db: Arc<DbPool>,
    details: Vec<LoanDetailModel>,
) -> Result<()> {
    let mut conn = db.get().await?;
    insert_into(crate::schema::loan_details::table).values(&details).execute(&mut conn).await?;
    Ok(())
}

pub fn convert_to_model(
    blocks: Vec<BlockAndEvents>,
    contract_address: &str,
) -> (Vec<LoanActionModel>, Vec<LoanDetailModel>) {
    let mut loan_actions = Vec::new();
    let mut loan_details = Vec::new();
    for bes in blocks {
        for be in bes {
            let events = be.events;
            for event in events {
                if event.contract_address.eq(&contract_address) {
                    if let Some(action) = LoanActionType::from_event_index(event.event_index) {
                        handle_loan_action_event(&mut loan_actions, &event, action);
                    } else if event.event_index == 1 {
                        handle_loan_detail_event(&event, &mut loan_details);
                    }
                }
            }
        }
    }
    (loan_actions, loan_details)
}

#[derive(Debug, thiserror::Error)]
#[error("CustomError: {msg}, {status}")]
pub struct CustomError {
    msg: String,
    status: u16,
}

impl CustomError {
    fn not_found(msg: String) -> Self {
        Self { msg, status: 404 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, DbEnum, Serialize, AsExpression)]
#[diesel(sql_type = SmallInt)]
#[diesel_enum(error_fn = CustomError::not_found)]
#[diesel_enum(error_type = CustomError)]
pub enum LoanActionType {
    LoanCreated,
    LoanCancelled,
    LoanPaid,
    LoanAccepted,
    LoanLiquidated,
}

impl LoanActionType {
    pub fn from_event_index(event_index: i32) -> Option<Self> {
        match event_index {
            2 => Some(Self::LoanCreated),
            3 => Some(Self::LoanCancelled),
            4 => Some(Self::LoanPaid),
            5 => Some(Self::LoanAccepted),
            6 => Some(Self::LoanLiquidated),
            _ => None,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::LoanCreated => "LoanCreated".to_string(),
            Self::LoanCancelled => "LoanCancelled".to_string(),
            Self::LoanPaid => "LoanPaid".to_string(),
            Self::LoanAccepted => "LoanAccepted".to_string(),
            Self::LoanLiquidated => "LoanLiquidated".to_string(),
        }
    }
}

fn handle_loan_action_event(
    models: &mut Vec<LoanActionModel>,
    event: &ContractEventByBlockHash,
    action: LoanActionType,
) {
    // Sanity check
    if event.fields.len() < 3 {
        tracing::warn!("Invalid event fields length: {}, skipping", event.fields.len());
    }

    match action {
        LoanActionType::LoanCreated => {
            models.push(LoanActionModel {
                loan_subcontract_id: event.fields[0].value.clone(),

                action_type: action,
                by: event.fields[2].value.clone(),
                timestamp: timestamp_millis_to_naive_datetime(
                    event.fields[3].value.parse::<i64>().unwrap(),
                ),
                loan_id: Some(
                    BigDecimal::from_f64(event.fields[1].value.parse::<f64>().unwrap()).unwrap(),
                ),
            });
        }
        _ => {
            models.push(LoanActionModel {
                loan_subcontract_id: event.fields[0].value.clone(),
                action_type: action,
                by: event.fields[1].value.clone(),
                timestamp: timestamp_millis_to_naive_datetime(
                    event.fields[2].value.parse::<i64>().unwrap(),
                ),
                loan_id: None, // Other actions does not need this field
            });
        }
    }
}

fn handle_loan_detail_event(event: &ContractEventByBlockHash, models: &mut Vec<LoanDetailModel>) {
    // Sanity check
    if event.fields.len() != 8 {
        tracing::warn!("Invalid event fields length: {}, skipping", event.fields.len());
    }

    models.push(LoanDetailModel {
        loan_subcontract_id: event.fields[0].value.clone(),
        lending_token_id: event.fields[1].value.clone(),
        collateral_token_id: event.fields[2].value.clone(),
        lending_amount: BigDecimal::from_f64(event.fields[3].value.parse::<f64>().unwrap())
            .unwrap(),
        collateral_amount: BigDecimal::from_f64(event.fields[4].value.parse::<f64>().unwrap())
            .unwrap(),
        interest_rate: BigDecimal::from_f64(event.fields[5].value.parse::<f64>().unwrap()).unwrap(),
        duration: BigDecimal::from_f64(event.fields[6].value.parse::<f64>().unwrap()).unwrap(),
        lender: event.fields[7].value.clone(),
    });
}
