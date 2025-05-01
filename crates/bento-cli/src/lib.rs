pub mod constants;
pub mod types;
use crate::types::*;
use bento_types::{repository::processor_status::get_last_timestamp, FetchStrategy};
use clap::Parser;

use anyhow::{Context, Result};
use bento_core::{
    client::Network, config::ProcessorConfig, new_db_pool, worker::SyncOptions,
    workers::worker::Worker, ProcessorFactory,
};
use bento_server::{start, Config as ServerConfig};
use constants::{DEFAULT_BLOCK_PROCESSOR, DEFAULT_EVENT_PROCESSOR, DEFAULT_TX_PROCESSOR};
use std::{collections::HashMap, fs, path::Path};
pub fn config_from_args(args: &CliArgs) -> Result<Config> {
    let config_path = args.config_path.clone();
    let config_str = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

async fn new_job_from_config(
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
    for (processor_type, processor_config) in config.processors.processors.iter() {
        if let Some(factory) = processor_factories.get(processor_type) {
            let processor_config = ProcessorConfig::Custom {
                name: processor_config.name.clone(),
                factory: *factory,
                args: Some(serde_json::to_value(processor_config)?),
            };
            processors.push(processor_config);
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

    let network = match worker_config.rpc_url.as_str() {
        "mainnet" => Network::Mainnet,
        "testnet" => Network::Testnet,
        "devnet" => Network::Devnet,
        rpc_url => Network::Custom(rpc_url.to_string()),
    };

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
    new_job_from_config(
        config,
        processor_factories,
        fetch_strategy,
        Some(SyncOptions {
            start_ts: None,
            step: Some(1000),
            back_step: None,
            sync_duration: Some(1000),
        }),
    )
    .await
}

pub async fn new_backfill_worker_from_config(
    config: &Config,
    processor_factories: &HashMap<String, ProcessorFactory>,
) -> Result<Worker> {
    new_job_from_config(
        config,
        processor_factories,
        Some(FetchStrategy::Parallel { num_workers: 10 }),
        Some(SyncOptions {
            start_ts: Some(config.worker.start),
            step: Some(config.worker.step),
            back_step: None,
            sync_duration: Some(config.worker.sync_duration),
        }),
    )
    .await
}

pub async fn run_worker(
    args: CliArgs,
    processor_factories: &HashMap<String, ProcessorFactory>,
) -> Result<()> {
    println!("⚙️  Running real-time indexer with config: {}", args.config_path);

    // Load config from args
    let config = config_from_args(&args)?;

    // Get the current timestamp for real-time indexing start point
    let current_time = chrono::Utc::now().timestamp() as u64;

    let worker = new_realtime_worker_from_config(&config, processor_factories, None).await?;

    // Run the worker
    println!("🚀 Starting real-time indexer from current time: {}", current_time);
    worker.run().await?;

    Ok(())
}

pub async fn new_server_from_args(args: &CliArgs) -> Result<ServerConfig> {
    // Load config from args
    let config = config_from_args(args)?;
    let db_pool = new_db_pool(&config.worker.database_url, None).await?;
    let server_config = ServerConfig {
        db_client: db_pool,
        api_host: "localhost".into(),
        api_port: config.server.port.parse()?,
    };
    Ok(server_config)
}

pub async fn run_server(args: CliArgs) -> Result<()> {
    dotenvy::dotenv().ok();
    println!("Starting server...");
    let config = new_server_from_args(&args).await?;
    println!("Server is ready and running on http://{}", config.api_endpoint());
    println!("Swagger UI is available at http://{}/swagger-ui", config.api_endpoint());
    start(config).await?;
    Ok(())
}

pub async fn run_backfill(
    args: CliArgs,
    processor_factories: &HashMap<String, ProcessorFactory>,
) -> Result<()> {
    println!("⚙️  Running backfill worker with config: {}", args.config_path);
    tracing_subscriber::fmt::init();

    // Load config from args
    let config = config_from_args(&args)?;

    // Create worker from config
    let worker = new_backfill_worker_from_config(&config, processor_factories).await?;

    // Run the worker
    println!("🚀 Starting backfill worker...");
    worker.run().await?;

    Ok(())
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
/// let processor_factories = HashMap::new();
/// run_command(processor_factories, true).await?;
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
            RunMode::Server(args) => run_server(args).await?,
            RunMode::Worker(args) => run_worker(args, &processor_factories).await?,
            RunMode::Backfill(args) => run_backfill(args, &processor_factories).await?,
            RunMode::BackfillStatus(args) => {
                if args.processor_name.is_empty() {
                    return Err(anyhow::anyhow!("Processor name is required for backfill status"));
                }
                let config = config_from_args(&args.clone().into())?;

                let worker =
                    new_realtime_worker_from_config(&config, &processor_factories, None).await?;

                // worker.processor_configs.iter().for_each(|p| {
                //     println!("Processor: {}", p.name());
                // });

                let backfill_height = get_last_timestamp(&worker.db_pool, &args.processor_name)
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
