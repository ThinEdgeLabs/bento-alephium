use crate::models::{
    account_transactions::AccountTransaction, new_account_transactions::NewTransferTransactionDto,
};
use anyhow::Result;
use bento_types::DbPool;
use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::sync::Arc;

pub struct AccountTransactionRepository {
    db_pool: Arc<DbPool>,
}

impl AccountTransactionRepository {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    pub async fn insert_transfers(&self, dtos: &Vec<NewTransferTransactionDto>) -> Result<()> {
        if dtos.is_empty() {
            return Ok(());
        }

        let mut conn = self.db_pool.get().await?;

        let result = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                async move {
                    use crate::schema::{account_transactions, transfers};

                    let account_txs: Vec<_> = dtos
                        .iter()
                        .map(|dto| {
                            let mut account_tx = dto.account_transaction.clone();
                            account_tx.tx_type = "transfer".to_string();
                            account_tx
                        })
                        .collect();

                    let inserted_account_txs: Vec<AccountTransaction> =
                        diesel::insert_into(account_transactions::table)
                            .values(&account_txs)
                            .on_conflict_do_nothing() // Skip duplicates
                            .returning(AccountTransaction::as_returning())
                            .get_results(conn)
                            .await?;

                    let transfer_values: Vec<_> = inserted_account_txs
                        .iter()
                        .zip(dtos.iter())
                        .map(|(inserted_tx, dto)| {
                            (
                                transfers::account_transaction_id.eq(inserted_tx.id),
                                transfers::token_id.eq(&dto.transfer.token_id),
                                transfers::from_address.eq(&dto.transfer.from_address),
                                transfers::to_address.eq(&dto.transfer.to_address),
                                transfers::amount.eq(&dto.transfer.amount),
                                transfers::tx_id.eq(&inserted_tx.tx_id),
                            )
                        })
                        .collect();

                    diesel::insert_into(transfers::table)
                        .values(&transfer_values)
                        .on_conflict((
                            transfers::tx_id,
                            transfers::token_id,
                            transfers::from_address,
                            transfers::to_address,
                            transfers::amount,
                        ))
                        .do_nothing()
                        .execute(conn)
                        .await?;

                    Ok(())
                }
                .scope_boxed()
            })
            .await?;

        Ok(result)
    }
}
