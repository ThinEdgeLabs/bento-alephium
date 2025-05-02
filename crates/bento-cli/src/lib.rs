pub mod constants;
pub mod types;
use crate::types::*;
use bento_types::{
    network::Network, repository::processor_status::get_last_timestamp, FetchStrategy,
};
use clap::Parser;

use anyhow::{Context, Result};
use bento_core::{
    config::ProcessorConfig, new_db_pool, worker::SyncOptions, workers::worker::Worker,
    ProcessorFactory,
};
use bento_server::{start, Config as ServerConfig};
use constants::{DEFAULT_BLOCK_PROCESSOR, DEFAULT_EVENT_PROCESSOR, DEFAULT_TX_PROCESSOR};
use std::{collections::HashMap, fs, path::Path};

pub fn config_from_args(args: &CliArgs) -> Result<Config> {
    let config_path = args.config_path.clone();
    let config_str = std::fs::read_to_string(config_path)?;
    let mut config: Config = toml::from_str(&config_str)?;

    // Override the network in the config with the one from args
    if args.network.is_some() {
        config.worker.network = args.network.clone().unwrap();
    }
    Ok(config)
}

async fn new_worker_from_config(
    config: &Config,
    processor_factories: &HashMap<String, ProcessorFactory>,
    fetch_strategy: Option<FetchStrategy>,
    sync_options: Option<SyncOptions>,
) -> Result<Worker> {
    // Get the worker configuration
    let worker_config = &config.worker;

    // Create processor configurations
    let mut processors = Vec::new();

    // Add processors from config based on what's available in processor_factories
    if let Some(processors_config) = &config.processors {
        for (processor_type, processor_config) in processors_config.processors.iter() {
            if let Some(factory) = processor_factories.get(processor_type) {
                let processor_config = ProcessorConfig::Custom {
                    name: processor_config.name.clone(),
                    factory: *factory,
                    args: Some(serde_json::to_value(processor_config)?),
                };
                processors.push(processor_config);
            }
        }
    }

    // Add default processors if exists in the factories
    if processor_factories.contains_key(DEFAULT_BLOCK_PROCESSOR) {
        processors.push(ProcessorConfig::BlockProcessor);
    }
    if processor_factories.contains_key(DEFAULT_EVENT_PROCESSOR) {
        processors.push(ProcessorConfig::EventProcessor);
    }
    if processor_factories.contains_key(DEFAULT_TX_PROCESSOR) {
        processors.push(ProcessorConfig::TxProcessor);
    }

    let network: Network;
    if let Some(rpc_url) = &worker_config.rpc_url {
        // Add the RPC URL to the network configuration
        network = Network::Custom(rpc_url.to_string(), worker_config.network.clone().into());
    } else {
        // Use the default network configuration
        network = worker_config.network.clone().into();
    }

    // Create and return the worker
    let worker = Worker::new(
        processors,
        worker_config.database_url.clone(),
        network,
        None, // Custom DB Schema
        sync_options,
        fetch_strategy,
    )
    .await?;
    Ok(worker)
}

pub async fn new_realtime_worker_from_config(
    config: &Config,
    processor_factories: &HashMap<String, ProcessorFactory>,
    fetch_strategy: Option<FetchStrategy>,
) -> Result<Worker> {
    let current_time = chrono::Utc::now().timestamp_millis() as u64;
    new_worker_from_config(
        config,
        processor_factories,
        fetch_strategy,
        Some(SyncOptions {
            start_ts: current_time,
            stop_ts: None,
            step: Some(1000),
            sync_duration: Some(1000),
        }),
    )
    .await
}

pub async fn new_backfill_worker_from_config(
    config: &Config,
    processor_factories: &HashMap<String, ProcessorFactory>,
) -> Result<Worker> {
    new_worker_from_config(
        config,
        processor_factories,
        Some(FetchStrategy::Parallel { num_workers: 10 }),
        Some(SyncOptions {
            start_ts: config.worker.start,
            stop_ts: config.worker.stop,
            step: Some(config.worker.step),
            sync_duration: Some(config.worker.sync_duration),
        }),
    )
    .await
}

pub async fn new_server_config_from_config(config: &Config) -> Result<ServerConfig> {
    let db_pool = new_db_pool(&config.worker.database_url, None).await?;
    let server_config = ServerConfig {
        db_client: db_pool,
        api_host: "localhost".into(),
        api_port: config.server.port.parse()?,
    };
    Ok(server_config)
}

/// Main function to run the command line interface
///
/// This function serves as the entry point for the Bento application's CLI.
/// It handles parsing command-line arguments and executing the appropriate
/// functionality based on the provided commands and options.
///
/// # Arguments
///
/// * `processor_factories` - A HashMap containing custom processor factories,
///   where the key is the processor name and the value is the processor factory function.
/// * `include_default_processors` - A boolean flag indicating whether to include
///   the default processors (block, event, and tx) in addition to any custom processors.
///
/// # Returns
///
/// * `Result<()>` - Returns Ok(()) if successful, or an error if any operation fails.
///
/// # Commands
///
/// The function supports various subcommands through the CLI:
/// * `Run` - Executes the application in different modes:
///   * `Server` - Runs in server mode.
///   * `Worker` - Runs in worker mode with specified processors.
///   * `Backfill` - Performs data backfilling for specified processors.
///   * `BackfillStatus` - Displays backfill status for a specific processor.
///
/// # Examples
///
/// ```
// / let processor_factories = HashMap::new();
// / run_command(processor_factories, true).await?;
/// ```
pub async fn run_command(
    processor_factories: HashMap<String, ProcessorFactory>,
    include_default_processors: bool,
) -> Result<()> {
    let mut processor_factories = processor_factories;
    if include_default_processors {
        processor_factories.insert(
            DEFAULT_BLOCK_PROCESSOR.to_string(),
            bento_core::processors::block_processor::processor_factory(),
        );

        processor_factories.insert(
            DEFAULT_EVENT_PROCESSOR.to_string(),
            bento_core::processors::event_processor::processor_factory(),
        );

        processor_factories.insert(
            DEFAULT_TX_PROCESSOR.to_string(),
            bento_core::processors::tx_processor::processor_factory(),
        );
    }

    let cli = Cli::parse();
    match cli.command {
        Commands::Run(run) => match run.mode {
            RunMode::Server(args) => {
                // First convert args to config
                let config = config_from_args(&args)?;

                println!("Starting server...");

                // Create server config from config
                let server_config = new_server_config_from_config(&config).await?;

                println!("Server is ready and running on http://{}", server_config.api_endpoint());
                println!(
                    "Swagger UI is available at http://{}/swagger-ui",
                    server_config.api_endpoint()
                );

                // Start the server
                start(server_config).await?;
            }
            RunMode::Worker(args) => {
                tracing_subscriber::fmt::init();

                let config = config_from_args(&args)?;

                println!("⚙️  Running real-time indexer with config: {}", args.config_path);

                let worker = new_realtime_worker_from_config(
                    &config,
                    &processor_factories,
                    Some(FetchStrategy::Simple),
                )
                .await?;

                println!("🚀 Starting real-time indexer");

                worker.run().await?;
            }
            RunMode::Backfill(args) => {
                // First convert args to config
                let config = config_from_args(&args)?;

                // Run backfill worker
                println!("⚙️  Running backfill worker with config: {}", args.config_path);
                tracing_subscriber::fmt::init();

                // Create backfill worker from config
                let worker = new_backfill_worker_from_config(&config, &processor_factories).await?;

                println!("🚀 Starting backfill worker...");
                worker.run().await?;
            }
            RunMode::BackfillStatus(args) => {
                println!("Running backfill status...");

                if args.processor_name.is_empty() {
                    return Err(anyhow::anyhow!("Processor name is required for backfill status"));
                }

                // First convert args to config
                let config = config_from_args(&args.clone().into())?;

                // Create worker from config
                let worker =
                    new_realtime_worker_from_config(&config, &processor_factories, None).await?;

                // Get backfill status
                let backfill_height =
                    get_last_timestamp(&worker.db_pool, &args.processor_name, args.network)
                        .await
                        .context("Failed to get last timestamp")?;

                println!(
                    "Backfill status for processor {}: last timestamp = {}",
                    args.processor_name, backfill_height
                );
            }
        },
    }
    Ok(())
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = fs::read_to_string(path).context("Failed to read config file")?;
    let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_config_file(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
        let config_path = dir.join("test_config.toml");
        let mut file = File::create(&config_path).expect("Failed to create test config file");
        file.write_all(content.as_bytes()).expect("Failed to write to test config file");
        config_path
    }

    #[test]
    fn test_config_from_args() {
        // Create a temporary directory for our test config
        let temp_dir = tempdir().expect("Failed to create temp directory");

        // Define a sample config
        let config_content = r#"
            [worker]
            database_url = "postgres://user:password@localhost:5432/db"
            network = "testnet"
            start = 1000
            stop = 2000
            step = 100
            sync_duration = 500

            [server]
            port = "8080"

            [backfill]
            start = 500
            end = 1500

            [processors.custom_processor]
            name = "custom"
            field1 = "value1"
            field2 = 42
        "#;

        // Create the config file
        let config_path = create_test_config_file(temp_dir.path(), config_content);

        // Create CLI args with the path to our test config
        let args = CliArgs {
            config_path: config_path.to_string_lossy().to_string(),
            network: Some("testnet".to_string()),
        };

        // Call the function we're testing
        let config = config_from_args(&args).expect("Failed to load config from args");

        // Verify the config was loaded correctly
        assert_eq!(config.worker.database_url, "postgres://user:password@localhost:5432/db");
        assert_eq!(config.worker.network, "testnet");
        assert_eq!(config.worker.start, 1000);
        assert_eq!(config.worker.stop, Some(2000));
        assert_eq!(config.worker.step, 100);
        assert_eq!(config.worker.sync_duration, 500);

        assert_eq!(config.server.port, "8080");

        assert_eq!(config.backfill.start, 500);
        assert_eq!(config.backfill.end, 1500);

        // Check that the processors were loaded
        assert!(config.processors.is_some());
        let processors = config.processors.unwrap();
        assert!(processors.processors.contains_key("custom_processor"));
        let custom_processor = &processors.processors["custom_processor"];
        assert_eq!(custom_processor.name, "custom");
        assert_eq!(custom_processor.config["field1"], serde_json::json!("value1"));
        assert_eq!(custom_processor.config["field2"], serde_json::json!(42));
    }

    #[test]
    fn test_load_config() {
        // Create a temporary directory for our test config
        let temp_dir = tempdir().expect("Failed to create temp directory");

        // Define a sample config
        let config_content = r#"
            [worker]
            database_url = "postgres://user:password@localhost:5432/db"
            network = "mainnet"
            start = 1000
            step = 100
            sync_duration = 500

            [server]
            port = "3000"

            [backfill]
            start = 500
            end = 1500
        "#;

        // Create the config file
        let config_path = create_test_config_file(temp_dir.path(), config_content);

        // Call the function we're testing
        let config = load_config(config_path).expect("Failed to load config");

        // Verify the config was loaded correctly
        assert_eq!(config.worker.database_url, "postgres://user:password@localhost:5432/db");
        assert_eq!(config.worker.network, "mainnet");
        assert_eq!(config.worker.start, 1000);
        assert_eq!(config.worker.stop, None);
        assert_eq!(config.worker.step, 100);
        assert_eq!(config.worker.sync_duration, 500);

        assert_eq!(config.server.port, "3000");

        assert_eq!(config.backfill.start, 500);
        assert_eq!(config.backfill.end, 1500);
    }

    #[test]
    fn test_config_with_optional_fields() {
        // Create a temporary directory for our test config
        let temp_dir = tempdir().expect("Failed to create temp directory");

        // Define a sample config with optional fields
        let config_content = r#"
            [worker]
            database_url = "postgres://user:password@localhost:5432/db"
            rpc_url = "https://example.com/rpc"
            network = "devnet"
            start = 1000
            stop = 2000
            step = 100
            sync_duration = 500
            workers = 4
            chunk_size = 100

            [server]
            port = "8080"

            [backfill]
            start = 500
            end = 1500
        "#;

        // Create the config file
        let config_path = create_test_config_file(temp_dir.path(), config_content);

        // Call the function we're testing
        let config = load_config(config_path).expect("Failed to load config");

        // Verify the optional fields were loaded correctly
        assert_eq!(config.worker.rpc_url, Some("https://example.com/rpc".to_string()));
        assert_eq!(config.worker.workers, Some(4));
        assert_eq!(config.worker.chunk_size, Some(100));
    }

    #[test]
    fn test_error_on_invalid_config_format() {
        // Create a temporary directory for our test config
        let temp_dir = tempdir().expect("Failed to create temp directory");

        // Define an invalid config
        let config_content = r#"
            This is not a valid TOML format
            [worker
            database_url = "postgres://user:password@localhost:5432/db"
        "#;

        // Create the config file
        let config_path = create_test_config_file(temp_dir.path(), config_content);

        // Create CLI args with the path to our test config
        let args = CliArgs {
            config_path: config_path.to_string_lossy().to_string(),
            network: Some("testnet".to_string()),
        };

        // Call the function we're testing - it should fail
        let result = config_from_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_on_missing_config_file() {
        // Create CLI args with a non-existent config path
        let args = CliArgs {
            config_path: "non_existent_config.toml".to_string(),
            network: Some("testnet".to_string()),
        };

        // Call the function we're testing - it should fail
        let result = config_from_args(&args);
        assert!(result.is_err());
    }
}
