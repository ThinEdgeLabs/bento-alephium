# Repository Module Documentation

The `repository.rs` module provides repository structs and methods for interacting with the database using Diesel ORM. It defines repositories for Blocks, Events, and Transactions, supporting operations such as querying, inserting, updating, and deleting records.

---

## BlockRepository
Manages database operations related to `Block`.

| Function Name        | Parameters                        | Return Type           | Description |
|---------------------|--------------------------------|---------------------|-------------|
| `find_by_hashes`   | `hashes: Vec<String>`          | `Result<Vec<Block>>` | Retrieves multiple blocks by their hashes. |
| `find_by_hash`     | `hash_val: &str`               | `Result<Option<Block>>` | Finds a block by its hash. |
| `find_by_height`   | `height_val: i64`             | `Result<Option<Block>>` | Retrieves a block by its height. |
| `store_block`      | `block: Block`                | `Result<Block>`     | Inserts a new block or updates an existing one if a conflict occurs. |
| `update_block`     | `hash_val: &str, block: Block` | `Result<Block>`     | Updates a block by its hash. |
| `delete_block`     | `hash_val: &str`              | `Result<usize>`     | Deletes a block by its hash. |
| `list_blocks`      | `limit: i64, offset: i64`     | `Result<Vec<Block>>` | Retrieves a paginated list of blocks. |

---

## EventRepository
Manages database operations related to `Event`.

| Function Name      | Parameters                                      | Return Type           | Description |
|-------------------|------------------------------------------------|---------------------|-------------|
| `find_by_tx_id`  | `tx_id_val: &str`                               | `Result<Option<Event>>` | Finds an event by transaction ID. |
| `store_event`    | `event: Event`                                  | `Result<Event>`     | Inserts a new event or updates an existing one if a conflict occurs. |
| `update_event`   | `tx_id_val: &str, event_index_val: i32, event: Event` | `Result<Event>`     | Updates an event by transaction ID and index. |
| `delete_event`   | `tx_id_val: &str, event_index_val: i32`         | `Result<usize>`     | Deletes an event by transaction ID and index. |
| `list_events`    | `limit: i64, offset: i64`                       | `Result<Vec<Event>>` | Retrieves a paginated list of events. |

---

## TransactionRepository
Manages database operations related to `Transaction`.

| Function Name        | Parameters                        | Return Type           | Description |
|---------------------|--------------------------------|---------------------|-------------|
| `find_by_hash`     | `hash_val: &str`               | `Result<Option<Transaction>>` | Finds a transaction by its hash. |
| `store_transaction` | `transaction: Transaction`    | `Result<Transaction>` | Inserts a new transaction or updates an existing one if a conflict occurs. |
| `update_transaction` | `hash_val: &str, transaction: Transaction` | `Result<Transaction>` | Updates a transaction by its hash. |
| `delete_transaction` | `hash_val: &str`              | `Result<usize>`     | Deletes a transaction by its hash. |
| `list_transactions` | `limit: i64, offset: i64`     | `Result<Vec<Transaction>>` | Retrieves a paginated list of transactions. |
