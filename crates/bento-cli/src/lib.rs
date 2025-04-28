use crate::types::*;
pub mod types;

use anyhow::{Context, Result};
use bento_core::{
    client::Network,
    config::ProcessorConfig,
    workers::worker_v2::{SyncOptions, Worker},
    ProcessorFactory,
};
use std::sync::Arc;
use std::{fs, path::Path};

pub fn config_from_args(args: &CliArgs) -> Result<Config> {
    let config_path = args.config_path.clone();
    let config_str = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

pub async fn new_worker_from_config(config: &Config, factory: ProcessorFactory) -> Result<Worker> {
    // Get the worker configuration
    let worker_config = &config.worker;

    // Create processor configurations
    let mut processors = Vec::new();

    // Add processors from config
    if let Some(lending_config) = &config.processors.lending {
        let processor_config = ProcessorConfig::Custom {
            name: lending_config.name.clone(),
            factory: factory.clone(),
            args: Some(serde_json::json!({"contract_address": lending_config.contract_address})),
        };
        processors.push(processor_config);
    }

    // Determine network type (default to Testnet if not specified)
    let network = Network::Testnet; // You might want to make this configurable

    // Create and return the worker
    let worker = Worker::new(
        processors,
        worker_config.database_url.clone(),
        network,
        None, // Custom DB Schema
        None,
        None,
    )
    .await?;

    Ok(worker)
}

pub async fn run_worker(args: CliArgs, factory: ProcessorFactory) -> Result<()> {
    println!("⚙️  Running worker with config: {}", args.config_path);

    // Load config from args
    let config = config_from_args(&args)?;

    // Create worker from config
    let worker = new_worker_from_config(&config, factory).await?;

    // Run the worker
    println!("🚀 Starting worker...");
    worker.run().await?;

    Ok(())
}

pub async fn new_server_from_args(args: &CliArgs) -> Result<()> {
    // Server implementation would go here
    Ok(())
}

pub async fn run_server(args: CliArgs) -> Result<()> {
    println!("🟢 Running server with config: {}", args.config_path);
    // Add your async server logic here
    Ok(())
}

pub async fn run_backfill(args: CliArgs) -> Result<()> {
    println!("📦 Running backfill with config: {}", args.config_path);
    // Add your async backfill logic here
    Ok(())
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = fs::read_to_string(path).context("Failed to read config file")?;
    let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
    Ok(config)
}
