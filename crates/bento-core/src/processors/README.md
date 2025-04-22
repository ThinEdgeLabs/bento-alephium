# Guide to Implementing Custom Block Processors

This guide will walk you through implementing a custom processor for blockchain data processing, similar to the lending marketplace processor example. We'll cover the essential components and best practices for creating your own processor.

## Understanding the Base Architecture

The processing system is built around the `ProcessorTrait` trait, which defines the core functionality that all processors must implement. This trait provides a standardized interface for processing blockchain data and interacting with the database.

### Core Components

- **ProcessorTrait**: The fundamental trait that all processors must implement
- **DbPool**: A connection pool for database operations
- **BlockAndEvents**: The data structure containing block and event information
- **async_trait**: Enables async functions in traits

## Step-by-Step Implementation Guide

### 1. Create Your Processor Struct

First, define your processor struct with the necessary fields:

```rust
pub struct CustomProcessor {
    connection_pool: Arc<DbPool>,
    // Add any additional fields specific to your processor
    contract_address: String,  // Example additional field
}

impl CustomProcessor {
    pub fn new(connection_pool: Arc<DbPool>, contract_address: String) -> Self {
        Self {
            connection_pool,
            contract_address
        }
    }
}
```

### 2. Implement Debug Trait

The `Debug` trait is required for all processors. Implement it to provide useful debugging information:

```rust
impl Debug for CustomProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "CustomProcessor {{ connections: {:?}, idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}
```

### 3. Implement ProcessorTrait

The core functionality comes from implementing `ProcessorTrait`:

```rust
#[async_trait]
impl ProcessorTrait for CustomProcessor {
    fn name(&self) -> &'static str {
        // Return a unique identifier for your processor
        "custom_processor"
    }

    fn connection_pool(&self) -> &Arc<DbPool> {
        &self.connection_pool
    }

    async fn process_blocks(
        &self,
        from_ts: i64,
        to_ts: i64,
        blocks: Vec<BlockAndEvents>,
    ) -> Result<()> {
        // Implement your block processing logic here
        self.process_block_data(blocks).await
    }
}
```

### 4. Define Your Data Models

Create structs that represent your database tables:

```rust
#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = schema::your_table)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct YourModel {
    // Define your fields here
    pub id: String,
    pub timestamp: NaiveDateTime,
    // Add other fields as needed
}
```

### 5. Implement Data Processing Logic

Create helper functions to process your data:

```rust
impl CustomProcessor {
    async fn process_block_data(&self, blocks: Vec<Vec<BlockAndEvents>>) -> Result<()> {
        // Convert blockchain data to your models
        let models = self.convert_to_models(blocks)?;
        
        // Insert data into database
        if !models.is_empty() {
            self.insert_models_to_db(models).await?;
        }
        
        Ok(())
    }

    fn convert_to_models(&self, blocks: Vec<Vec<BlockAndEvents>>) -> Result<Vec<YourModel>> {
        let mut models = Vec::new();
        
        for block_group in blocks {
            for block in block_group {
                // Process events and create models
                // Add error handling and validation
            }
        }
        
        Ok(models)
    }

    async fn insert_models_to_db(&self, models: Vec<YourModel>) -> Result<()> {
        let mut conn = self.connection_pool.get().await?;
        
        insert_into(schema::your_table::table)
            .values(&models)
            .execute(&mut conn)
            .await?;
            
        Ok(())
    }
}
```

### 6. Add to Processor Enum

Update the main `Processor` enum to include your new processor:

```rust
#[derive(Debug)]
pub enum Processor {
    // Existing variants...
    CustomProcessor(CustomProcessor),
}
```

### 7. Implement ProcessorTrait for Enum

Update the `ProcessorTrait` implementation for the `Processor` enum:

```rust
#[async_trait]
impl ProcessorTrait for Processor {
    fn connection_pool(&self) -> &Arc<DbPool> {
        match self {
            // Existing matches...
            Processor::CustomProcessor(p) => p.connection_pool(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            // Existing matches...
            Processor::CustomProcessor(p) => p.name(),
        }
    }

    async fn process_blocks(
        &self,
        from_ts: i64,
        to_ts: i64,
        blocks: Vec<BlockAndEvents>,
    ) -> Result<()> {
        match self {
            // Existing matches...
            Processor::CustomProcessor(p) => p.process_blocks(from_ts, to_ts, blocks).await,
        }
    }
}
```

## Best Practices and Tips

### Error Handling

Use proper error handling throughout your processor. Consider creating custom error types for specific failure cases.

### Logging

Implement comprehensive logging using the `tracing` framework:

```rust
tracing::info!("Processing block group: {}", block_id);
tracing::warn!("Invalid event data: {:?}", event);
tracing::error!("Database error: {:?}", err);
```

### Database Operations

- Use transactions for related operations
- Implement batch processing for better performance
- Handle connection errors gracefully

### Testing

Create unit tests for your processor:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_blocks() {
        // Setup test data and environment
        // Run your processor
        // Assert expected outcomes
    }
}
```

## Common Pitfalls to Avoid

- Don't assume event data is always valid - implement proper validation
- Avoid processing blocks without proper error handling
- Don't forget to handle database connection failures
- Remember to implement proper cleanup and resource management

## Example Usage

Here's how to instantiate and use your custom processor:

```rust
let pool = Arc::new(create_connection_pool().await?);
let processor = CustomProcessor::new(pool, "contract_address".to_string());

// Use the processor
processor.process_blocks(
    from_timestamp,
    to_timestamp,
    blocks
).await?;
```

Remember to handle errors appropriately and implement proper logging throughout your processor implementation.
