use std::{fs, path::Path};

use anyhow::Context;
use bento_core::{
    client::Client, config::ProcessorConfig, db::new_db_pool, types::FetchStrategy,
    workers::worker_v2::Worker,
};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "A CLI tool with server, worker, and backfill modes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunCommand),
}

#[derive(Subcommand)]
enum RunMode {
    Server(CliArgs),
    Worker(CliArgs),
    Backfill(CliArgs),
}

#[derive(Args)]
struct CliArgs {
    /// Path to the config file
    #[arg(short, long)]
    config_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    worker: WorkerConfig,
    server: ServerConfig,
    backfill: BackfillConfig,
    processors: ProcessorsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkerConfig {
    database_url: String,
    rpc_url: String,
    start: u64,
    step: u64,
    sync_duration: u64,
    stratery_name: String,
    workers: Option<u32>,
    chunk_size: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServerConfig {
    port: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct BackfillConfig {
    enabled: bool,
    start: u64,
    end: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessorsConfig {
    pub lending: LendingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LendingConfig {
    pub name: String,
    pub contract_address: String,
}

#[derive(Args)]
struct RunCommand {
    #[command(subcommand)]
    mode: RunMode,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(run) => match run.mode {
            RunMode::Server(args) => run_server(args).await?,
            RunMode::Worker(args) => run_worker(args).await?,
            RunMode::Backfill(args) => run_backfill(args).await?,
        },
    }

    Ok(())
}

fn config_from_args(args: &CliArgs) -> anyhow::Result<Config> {
    let config_path = args.config_path.clone();
    let config_str = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

async fn new_server_from_args(args: &CliArgs) -> anyhow::Result<()> {
    Ok(())
}

async fn new_worker_from_config(config: &Config) -> anyhow::Result<()> {
    Ok(())
}

async fn run_server(args: CliArgs) -> anyhow::Result<()> {
    println!("🟢 Running server with config: {}", args.config_path);
    // Add your async server logic here
    Ok(())
}

async fn run_worker(args: CliArgs) -> anyhow::Result<()> {
    println!("⚙️  Running worker with config: {}", args.config_path);
    // Add your async worker logic here
    Ok(())
}

async fn run_backfill(args: CliArgs) -> anyhow::Result<()> {
    println!("📦 Running backfill with config: {}", args.config_path);
    // Add your async backfill logic here
    Ok(())
}

fn load_config<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
    let content = fs::read_to_string(path).context("Failed to read config file")?;
    let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
    Ok(config)
}
