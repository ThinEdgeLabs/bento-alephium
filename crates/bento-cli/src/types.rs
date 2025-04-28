use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "A CLI tool with server, worker, and backfill modes", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run(RunCommand),
}

#[derive(Subcommand)]
pub enum RunMode {
    Server(CliArgs),
    Worker(CliArgs),
    Backfill(CliArgs),
}

#[derive(Args)]
pub struct CliArgs {
    /// Path to the config file
    #[arg(short, long)]
    pub config_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub worker: WorkerConfig,
    pub server: ServerConfig,
    pub backfill: BackfillConfig,
    pub processors: ProcessorsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkerConfig {
    pub database_url: String,
    pub rpc_url: String,
    pub start: u64,
    pub step: u64,
    pub sync_duration: u64,
    pub workers: Option<u32>,
    pub chunk_size: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BackfillConfig {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessorsConfig {
    pub lending: Option<LendingConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LendingConfig {
    pub name: String,
    pub contract_address: String,
}

#[derive(Args)]
pub struct RunCommand {
    #[command(subcommand)]
    pub mode: RunMode,
}
