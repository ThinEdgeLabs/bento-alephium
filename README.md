# Bento Alephium - Custom Blockchain Data Processor Framework

This framework allows you to create custom processors for indexing and processing Alephium blockchain data. The framework provides a flexible architecture for handling different types of blockchain events and storing them in a PostgreSQL database using Diesel ORM.

## Example: Lending Marketplace Processor

The Lending Marketplace Processor demonstrates how to create a custom processor that tracks lending contract events. It processes two types of events:
1. Loan Actions (Create, Cancel, Pay, Accept, Liquidate)
2. Loan Details (lending and collateral information)

### Core Components

1. **Data Models**: Database tables represented as Rust structs using Diesel ORM
   - `LoanActionModel`: Tracks loan lifecycle events
   - `LoanDetailModel`: Stores loan terms and conditions

2. **Custom Output Type**: Define how processor output is handled
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

3. **Event Types**: Enumeration of supported contract events
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
   ```

### Implementation Guide

1. **Define Your Data Models**

```rust
#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = schema::loan_actions)]
pub struct LoanActionModel {
    loan_subcontract_id: String,
    loan_id: Option<BigDecimal>,
    by: String,
    timestamp: NaiveDateTime,
    action_type: LoanActionType,
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, AsChangeset)]
#[diesel(table_name = schema::loan_details)]
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

2. **Create Your Processor**

```rust
pub struct LendingContractProcessor {
    connection_pool: Arc<DbPool>,
    contract_address: String,
}

impl LendingContractProcessor {
    pub fn new(connection_pool: Arc<DbPool>, contract_address: String) -> Self {
        Self { connection_pool, contract_address }
    }
}

#[async_trait]
impl ProcessorTrait for LendingContractProcessor {
    fn name(&self) -> &'static str {
        "lending_contract_processor"
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

        tracing::info!(
            "Processed {} loan actions and {} loan details",
            loan_actions.len(),
            loan_details.len()
        );

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
                // Store loan actions
                if !lending_output.loan_actions.is_empty() {
                    insert_loan_actions_to_db(
                        self.connection_pool.clone(), 
                        lending_output.loan_actions.clone()
                    ).await?;
                }

                // Store loan details
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

3. **Create Factory Function and Register Processor**

The factory function is a crucial part that creates and configures your processor. It's used by the worker to instantiate your processor with the correct configuration:

```rust
// Factory function that creates your processor instance
fn register_lending_contract(
    pool: Arc<DbPool>, 
    args: Option<serde_json::Value>
) -> Box<dyn ProcessorTrait> {
    // Extract contract address from args
    let contract_address = if let Some(args) = args {
        args.get("contract_address")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string()
    } else {
        panic!("Missing contract address argument")
    };

    // Create and return the processor
    Box::new(LendingContractProcessor::new(pool, contract_address))
}

// Register the processor with the worker
let processor_config = ProcessorConfig::Custom { 
    name: "lending processor".to_string(),
    factory: register_lending_contract,  // Pass the factory function
    args: Some(serde_json::json!({
        "contract_address": "yuF1Sum4ricLFBc86h3RdjFsebR7ZXKBHm2S5sZmVsiF"
    }))
};

// Create worker with the processor
let worker = Worker::new(
    vec![processor_config],  // Can register multiple processors
    database_url,
    Network::Testnet,
    None,
    Some(SyncOptions {
        start_ts: Some(1716560632750),
        step: Some(1800000 * 10),
        back_step: None,
        sync_duration: None,
    }),
    Some(FetchStrategy::Parallel { num_workers: 10 }),
).await?;
```

The factory function pattern allows for:
- Dynamic processor creation based on configuration
- Dependency injection (database pool)
- Configuration validation at startup
- Multiple processor instances with different configurations
- Clean separation between processor creation and usage

### Event Field Structure

1. **Loan Action Events**:
   - Field 0: loan_subcontract_id (String)
   - For LoanCreated:
     - Field 1: loan_id (BigDecimal)
     - Field 2: by (String)
     - Field 3: timestamp (i64)
   - For Other Actions:
     - Field 1: by (String)
     - Field 2: timestamp (i64)

2. **Loan Detail Events** (event_index = 1):
   - Field 0: loan_subcontract_id (String)
   - Field 1: lending_token_id (String)
   - Field 2: collateral_token_id (String)
   - Field 3: lending_amount (f64)
   - Field 4: collateral_amount (f64)
   - Field 5: interest_rate (f64)
   - Field 6: duration (f64)
   - Field 7: lender (String)

### Running the Processor

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let processor_config = ProcessorConfig::Custom { 
        name: "lending processor".to_string(),
        factory: register_lending_contract,
        args: Some(serde_json::json!({
            "contract_address": "yuF1Sum4ricLFBc86h3RdjFsebR7ZXKBHm2S5sZmVsiF"
        }))
    };

    let worker = Worker::new(
        vec![processor_config],
        database_url,
        Network::Testnet,
        None,
        Some(SyncOptions {
            start_ts: Some(1716560632750),
            step: Some(1800000 * 10), // Process blocks in 5-hour chunks
            back_step: None,
            sync_duration: None,
        }),
        Some(FetchStrategy::Parallel { num_workers: 10 }),
    ).await?;

    worker.run().await?;
    Ok(())
}
```

### Best Practices

1. **Event Processing**
   - Validate event field count before processing
   - Use proper type conversion with error handling
   - Filter events by contract address
   - Handle different event types appropriately

2. **Custom Output Handling**
   - Implement `CustomProcessorOutput` trait for your output type
   - Use `ProcessorOutput::Custom` to wrap your output
   - Override `store_output` to handle custom data storage
   - Use proper type downcasting with error handling

3. **Error Handling**
   - Use custom error types for specific failures
   - Implement comprehensive logging
   - Handle all potential error cases
   - Validate input data thoroughly

4. **Configuration**
   - Use environment variables for database configuration
   - Pass contract address through processor args
   - Configure appropriate sync options
   - Use parallel processing when possible

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Insert your license information here]
