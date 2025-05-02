# Bento Alephium - Blockchain Indexer Framework

Bento is a powerful indexer framework for the Alephium blockchain, designed to make it easy to build custom processors that track and store blockchain events. The framework provides a flexible, type-safe architecture for processing different types of contract events and persisting them in a PostgreSQL database using Diesel ORM.

## Key Features

- **Modular Architecture**: Create custom processors for different contract types
- **Type-Safe Data Handling**: Leverages Rust's type system with Diesel ORM
- **Flexible Event Processing**: Process events by contract address and event type
- **Parallel Block Processing**: Configurable concurrent execution strategies
- **Custom Output Types**: Define specialized data models for your specific needs
- **Factory Pattern Support**: Dynamic processor creation with dependency injection

## Getting Started

### Prerequisites

- Rust and Cargo
- PostgreSQL database
- Access to an Alephium node (Mainnet or Testnet)

### Installation

Add Bento to your Cargo.toml:

```toml
[dependencies]
bento_core = "0.1.0"
bento_trait = "0.1.0"
bento_types = "0.1.0"
bento_cli = "0.1.0"
diesel = { version = "2.0.0", features = ["postgres", "chrono", "serde_json"] }
diesel-async = { version = "0.3.0", features = ["postgres", "deadpool"] }
diesel_enum = "0.1.0"
```

### Basic Usage

Create a `main.rs` file that registers your custom processors:

```rust
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut processor_factories = HashMap::new();
    processor_factories.insert("lending".to_string(), lending_example::processor_factory());
    bento_cli::run_command(processor_factories, true).await?;
    Ok(())
}
```

## Creating a Custom Processor

The framework allows you to create specialized processors for different contract types. Here's a walkthrough using the Lending Marketplace Processor example:

### 1. Define Your Data Models

First, define the database models that represent the data you want to store:

```rust
#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = bento_types::schema::loan_actions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LoanActionModel {
    loan_subcontract_id: String,
    loan_id: Option<BigDecimal>,
    by: String,
    timestamp: NaiveDateTime,
    action_type: LoanActionType,
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = bento_types::schema::loan_details)]
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
```

### 2. Define Event Types

Create enums for the different event types your processor will handle:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, DbEnum, Serialize, AsExpression)]
#[diesel(sql_type = SmallInt)]
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
}
```

### 3. Create a Custom Output Type

Define a custom output type that will hold the processed data:

```rust
#[derive(Debug, Clone)]
pub struct LendingContractOutput {
    pub loan_actions: Vec<LoanActionModel>,
    pub loan_details: Vec<LoanDetailModel>,
}

impl CustomProcessorOutput for LendingContractOutput {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn CustomProcessorOutput> {
        Box::new(self.clone())
    }
}
```

### 4. Implement Your Processor

Create a processor struct and implement the `ProcessorTrait`:

```rust
pub struct LendingContractProcessor {
    connection_pool: Arc<DbPool>,
    contract_address: String,
}

impl LendingContractProcessor {
    pub fn new(connection_pool: Arc<DbPool>, args: serde_json::Value) -> Self {
        let contract_address = args
            .get("contract_address")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("Missing contract address argument"))
            .to_string();
        Self { connection_pool, contract_address }
    }
}

#[async_trait]
impl ProcessorTrait for LendingContractProcessor {
    fn name(&self) -> &'static str {
        "lending"
    }

    fn connection_pool(&self) -> &Arc<DbPool> {
        &self.connection_pool
    }

    async fn process_blocks(
        &self,
        _from: i64,
        _to: i64,
        blocks: Vec<BlockAndEvents>,
    ) -> Result<ProcessorOutput> {
        // Process blocks and convert to models
        let (loan_actions, loan_details) = convert_to_model(blocks, &self.contract_address);

        // Return custom output
        Ok(ProcessorOutput::Custom(Arc::new(LendingContractOutput { 
            loan_actions, 
            loan_details 
        })))
    }

    // Override storage method to handle custom output
    async fn store_output(&self, output: ProcessorOutput) -> Result<()> {
        if let ProcessorOutput::Custom(custom) = output {
            if let Some(lending_output) = custom.as_any().downcast_ref::<LendingContractOutput>() {
                // Store data in database
                if !lending_output.loan_actions.is_empty() {
                    insert_loan_actions_to_db(
                        self.connection_pool.clone(), 
                        lending_output.loan_actions.clone()
                    ).await?;
                }

                if !lending_output.loan_details.is_empty() {
                    insert_loan_details_to_db(
                        self.connection_pool.clone(), 
                        lending_output.loan_details.clone()
                    ).await?;
                }
            }
        }
        Ok(())
    }
}
```

### 5. Create a Factory Function

Define a factory function that creates your processor instance:

```rust
pub fn processor_factory() -> ProcessorFactory {
    |db_pool, args: Option<serde_json::Value>| {
        Box::new(LendingContractProcessor::new(db_pool, args.unwrap_or_default()))
    }
}
```

### 6. Event Processing Functions

Implement functions to handle specific event types:

```rust
fn handle_loan_action_event(
    models: &mut Vec<LoanActionModel>,
    event: &ContractEventByBlockHash,
    action: LoanActionType,
) {
    if event.fields.len() < 3 {
        tracing::warn!("Invalid event fields length: {}, skipping", event.fields.len());
        return;
    }

    match action {
        LoanActionType::LoanCreated => {
            models.push(LoanActionModel {
                loan_subcontract_id: event.fields[0].value.clone().to_string(),
                action_type: action,
                by: event.fields[2].value.clone().to_string(),
                timestamp: timestamp_millis_to_naive_datetime(
                    event.fields[3].value.as_str().unwrap().parse::<i64>().unwrap(),
                ),
                loan_id: Some(
                    BigDecimal::from_str(event.fields[1].value.as_str().unwrap()).unwrap(),
                ),
            });
        }
        _ => {
            // Handle other action types
            // ...
        }
    }
}
```

## Running Your Indexer

Use the Bento CLI to run your indexer:

```bash
DATABASE_URL=postgres://username:password@localhost/mydb cargo run -- sync --network testnet
```

The CLI supports various commands and options:

```bash
# Start indexing from a specific timestamp
cd examples/lending-indexer

# Start the backfill process
cargo run run backfill --network testnet

# Start the real-time sync
cargo run run worker --network testnet  

# Start the server
cargo run run server

# Query the latest timestamp
cargo run run backfill-status -processor-name lending_processor

```

## Configuration

Configuration is handled through environment variables and command-line arguments:

1. **Database**: Set via `DATABASE_URL` environment variable
2. **Network**: Choose between `mainnet` and `testnet`
3. **Sync Options**: Configure start time, step size, etc.
4. **Processor Args**: Pass contract addresses and other parameters

Example `.env` file:

```
DATABASE_URL=postgres://username:password@localhost/bento_db
RUST_LOG=info
```

## Best Practices

### Error Handling

Use proper error types and handling:

```rust
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
```

### Event Validation

Always validate event fields before processing:

```rust
if event.fields.len() < expected_length {
    tracing::warn!("Invalid event fields length: {}, skipping", event.fields.len());
    return;
}
```

### Type Conversion

Handle type conversions safely:

```rust
BigDecimal::from_f64(
    event.fields[3].value.as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_default(),
)
.unwrap_or_default()
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.