use crate::models::{AccountTransaction, NewContractCallTransactionDto, NewTransferTransactionDto};
use crate::models::{AccountTransactionDetails, TransferDetails, TransferTransactionDto};
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
                            .on_conflict_do_nothing()
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

    pub async fn insert_contract_calls(
        &self,
        dtos: &Vec<NewContractCallTransactionDto>,
    ) -> Result<()> {
        if dtos.is_empty() {
            return Ok(());
        }

        let mut conn = self.db_pool.get().await?;

        let result = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                async move {
                    use crate::schema::{account_transactions, contract_calls};

                    let account_txs: Vec<_> = dtos
                        .iter()
                        .map(|dto| {
                            let mut account_tx = dto.account_transaction.clone();
                            account_tx.tx_type = "contract_call".to_string();
                            account_tx
                        })
                        .collect();

                    let inserted_account_txs: Vec<AccountTransaction> =
                        diesel::insert_into(account_transactions::table)
                            .values(&account_txs)
                            .on_conflict_do_nothing()
                            .returning(AccountTransaction::as_returning())
                            .get_results(conn)
                            .await?;

                    let contract_call_values: Vec<_> = inserted_account_txs
                        .iter()
                        .zip(dtos.iter())
                        .map(|(inserted_tx, dto)| {
                            (
                                contract_calls::account_transaction_id.eq(inserted_tx.id),
                                contract_calls::contract_address
                                    .eq(&dto.contract_call.contract_address),
                                contract_calls::tx_id.eq(&inserted_tx.tx_id),
                            )
                        })
                        .collect();

                    diesel::insert_into(contract_calls::table)
                        .values(&contract_call_values)
                        .on_conflict((contract_calls::tx_id, contract_calls::contract_address))
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

    pub async fn get_account_transactions(
        &self,
        address: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AccountTransactionDetails>> {
        let mut conn = self.db_pool.get().await?;

        use crate::schema::{account_transactions, transfers};

        let account_txs: Vec<AccountTransaction> = account_transactions::table
            .filter(account_transactions::address.eq(address))
            .order_by(account_transactions::timestamp.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await?;

        if account_txs.is_empty() {
            return Ok(Vec::new());
        }

        let tx_ids: Vec<i64> = account_txs.iter().map(|tx| tx.id).collect();

        let transfers_map: std::collections::HashMap<i64, TransferDetails> = transfers::table
            .filter(transfers::account_transaction_id.eq_any(&tx_ids))
            .load::<(i64, i64, String, String, String, bigdecimal::BigDecimal, String)>(&mut conn)
            .await?
            .into_iter()
            .map(|(id, account_tx_id, token_id, from_addr, to_addr, amount, _tx_id)| {
                (
                    account_tx_id,
                    TransferDetails {
                        id,
                        token_id,
                        from_address: from_addr,
                        to_address: to_addr,
                        amount,
                    },
                )
            })
            .collect();

        let mut transaction_details = Vec::new();
        for account_tx in account_txs {
            match account_tx.tx_type.as_str() {
                "transfer" => {
                    if let Some(transfer) = transfers_map.get(&account_tx.id) {
                        transaction_details.push(AccountTransactionDetails::Transfer(
                            TransferTransactionDto {
                                account_transaction: account_tx,
                                transfer: transfer.clone(),
                            },
                        ));
                    }
                }
                // TODO: Add other transaction types
                _ => continue,
            }
        }

        Ok(transaction_details)
    }
}
