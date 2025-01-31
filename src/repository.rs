//! This module provides repository implementations for interacting with the database,
//! specifically handling blocks, events, and transactions using Diesel ORM.
//! 
//! The repositories allow CRUD operations and querying based on various parameters.

use crate::{
    db::DbPool,
    models::{self, Block, Event, Transaction},
};
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;

/// Repository for handling block-related database operations.
pub struct BlockRepository {
    pub pool: DbPool,
}

impl BlockRepository {
    /// Retrieves multiple blocks by their hashes.
    ///
    /// # Arguments
    /// * `hashes` - A vector of block hashes.
    ///
    /// # Returns
    /// * `Result<Vec<Block>>` - A vector of blocks matching the given hashes.
    pub fn find_by_hashes(&self, hashes: Vec<String>) -> Result<Vec<Block>> {
        use crate::schema::blocks::dsl::*;
        let mut conn = self.pool.get().unwrap();
        blocks.filter(hash.eq_any(hashes)).load::<Block>(&mut conn).map_err(anyhow::Error::msg)
    }

    /// Retrieves a block by its hash.
    ///
    /// # Arguments
    /// * `hash_val` - The hash value of the block.
    ///
    /// # Returns
    /// * `Result<Option<Block>>` - The block if found, otherwise `None`.
    pub fn find_by_hash(&self, hash_val: &str) -> Result<Option<Block>> {
        use crate::schema::blocks::dsl::*;
        let mut conn = self.pool.get().unwrap();
        blocks
            .filter(hash.eq(hash_val))
            .first::<Block>(&mut conn)
            .optional()
            .map_err(anyhow::Error::msg)
    }

    /// Retrieves a block by its height.
    ///
    /// # Arguments
    /// * `height_val` - The height of the block.
    ///
    /// # Returns
    /// * `Result<Option<Block>>` - The block if found, otherwise `None`.
    pub fn find_by_height(&self, height_val: i64) -> Result<Option<Block>> {
        use crate::schema::blocks::dsl::*;
        let mut conn = self.pool.get().unwrap();
        blocks
            .filter(height.eq(height_val))
            .first::<Block>(&mut conn)
            .optional()
            .map_err(anyhow::Error::msg)
    }

    /// Stores a new block or updates an existing one based on the hash.
    ///
    /// # Arguments
    /// * `block` - The block to store.
    ///
    /// # Returns
    /// * `Result<Block>` - The stored or updated block.
    pub fn store_block(&self, block: Block) -> Result<Block> {
        use crate::schema::blocks::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(blocks)
            .values(&block)
            .on_conflict(hash)
            .do_update()
            .set(&block)
            .get_result(&mut conn)
            .map_err(anyhow::Error::msg)
    }

    /// Updates a block by its hash.
    ///
    /// # Arguments
    /// * `hash_val` - The hash of the block to update.
    /// * `block` - The new block data.
    ///
    /// # Returns
    /// * `Result<Block>` - The updated block.
    pub fn update_block(&self, hash_val: &str, block: Block) -> Result<Block> {
        use crate::schema::blocks::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::update(blocks.filter(hash.eq(hash_val)))
            .set(&block)
            .get_result(&mut conn)
            .map_err(anyhow::Error::msg)
    }

    /// Deletes a block by its hash.
    ///
    /// # Arguments
    /// * `hash_val` - The hash of the block to delete.
    ///
    /// # Returns
    /// * `Result<usize>` - The number of deleted rows.
    pub fn delete_block(&self, hash_val: &str) -> Result<usize> {
        use crate::schema::blocks::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::delete(blocks.filter(hash.eq(hash_val)))
            .execute(&mut conn)
            .map_err(anyhow::Error::msg)
    }

    /// Lists blocks with pagination.
    ///
    /// # Arguments
    /// * `limit` - The number of blocks to retrieve.
    /// * `offset` - The starting position.
    ///
    /// # Returns
    /// * `Result<Vec<Block>>` - A vector of blocks ordered by height in descending order.
    pub fn list_blocks(&self, limit: i64, offset: i64) -> Result<Vec<Block>> {
        use crate::schema::blocks::dsl::*;
        let mut conn = self.pool.get().unwrap();
        blocks
            .order(height.desc())
            .limit(limit)
            .offset(offset)
            .load::<Block>(&mut conn)
            .map_err(anyhow::Error::msg)
    }

}

pub struct EventRepository {
    pub pool: DbPool,
}
/// Repository for handling event-related database operations.
impl EventRepository {
    /// Retrieves an event by its transaction ID.
    pub fn find_by_tx_id(&self, tx_id_val: &str) -> Result<Option<Event>> {
        use crate::schema::events::dsl::*;
        let mut conn = self.pool.get().unwrap();
        events
            .filter(tx_id.eq(tx_id_val))
            .select(Event::as_select())
            .first::<Event>(&mut conn)
            .optional()
            .map_err(anyhow::Error::msg)
    }
/// Stores or updates an event.
    pub fn store_event(&self, event: Event) -> Result<Event> {
        use crate::schema::events::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(events)
            .values(&event)
            .on_conflict((tx_id, event_index))
            .do_update()
            .set(&event)
            .returning(Event::as_select())
            .get_result(&mut conn)
            .map_err(anyhow::Error::msg)
    }

    pub fn update_event(
        &self,
        tx_id_val: &str,
        event_index_val: i32,
        event: Event,
    ) -> Result<Event> {
        use crate::schema::events::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::update(events.filter(tx_id.eq(tx_id_val).and(event_index.eq(event_index_val))))
            .set(&event)
            .returning(Event::as_select())
            .get_result(&mut conn)
            .map_err(anyhow::Error::msg)
    }

    pub fn delete_event(&self, tx_id_val: &str, event_index_val: i32) -> Result<usize> {
        use crate::schema::events::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::delete(events.filter(tx_id.eq(tx_id_val).and(event_index.eq(event_index_val))))
            .execute(&mut conn)
            .map_err(anyhow::Error::msg)
    }

    pub fn list_events(&self, limit: i64, offset: i64) -> Result<Vec<Event>> {
        use crate::schema::events::dsl::*;
        let mut conn = self.pool.get().unwrap();
        events
            .select(Event::as_select())
            .order(tx_id.desc())
            .limit(limit)
            .offset(offset)
            .load::<Event>(&mut conn)
            .map_err(anyhow::Error::msg)
    }
}

pub struct TransactionRepository {
    pub pool: DbPool,
}

impl TransactionRepository {
    /// Retrieves a transaction by its hash.
    pub fn find_by_hash(&self, hash_val: &str) -> Result<Option<Transaction>> {
        use crate::schema::transactions::dsl::*;
        let mut conn = self.pool.get().unwrap();
        transactions
            .filter(tx_hash.eq(hash_val))
            .first::<Transaction>(&mut conn)
            .optional()
            .map_err(anyhow::Error::msg)
    }
 /// Stores or updates a transaction.
    pub fn store_transaction(&self, transaction: Transaction) -> Result<Transaction> {
        use crate::schema::transactions::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(transactions)
            .values(&transaction)
            .on_conflict(tx_hash)
            .do_update()
            .set(&transaction)
            .get_result(&mut conn)
            .map_err(anyhow::Error::msg)
    }
 /// Updates a transaction by its hash.
    pub fn update_transaction(
        &self,
        hash_val: &str,
        transaction: Transaction,
    ) -> Result<Transaction> {
        use crate::schema::transactions::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::update(transactions.filter(tx_hash.eq(hash_val)))
            .set(&transaction)
            .get_result(&mut conn)
            .map_err(anyhow::Error::msg)
    }
 /// Deletes a transaction by its hash.
    pub fn delete_transaction(&self, hash_val: &str) -> Result<usize> {
        use crate::schema::transactions::dsl::*;
        let mut conn = self.pool.get().unwrap();
        diesel::delete(transactions.filter(tx_hash.eq(hash_val)))
            .execute(&mut conn)
            .map_err(anyhow::Error::msg)
    }
 /// Lists transactions with pagination.
    pub fn list_transactions(&self, limit: i64, offset: i64) -> Result<Vec<Transaction>> {
        use crate::schema::transactions::dsl::*;
        let mut conn = self.pool.get().unwrap();
        transactions
            .order(tx_hash.desc())
            .limit(limit)
            .offset(offset)
            .load::<Transaction>(&mut conn)
            .map_err(anyhow::Error::msg)
    }
}
