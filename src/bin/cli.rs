use clap::Parser;
use bento_alephium::{
    client::Network,
    config::ProcessorConfig,
    workers::worker_v2::{SyncOptions, Worker},
    processors::ProcessorTrait,
    db::DbPool,
};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;
use std::io::Read;
use regex::Regex;

// Add additional processor modules here

/// CLI tool for managing the Alephium indexer
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start timestamp of the range
    #[arg(long, short)]
    from: i64,

    /// End timestamp of the range
    #[arg(long, short)]
    to: i64,

    /// Step size for processing blocks (in milliseconds)
    #[arg(long, short, default_value = "1000")]
    step: i64,

    /// Number of parallel workers for fetching data
    #[arg(long, short, default_value = "10")]
    workers: usize,

    /// Network to connect to (mainnet/testnet)
    #[arg(long, short, default_value = "testnet")]
    network: String,

    /// Contract address for custom processor (e.g. lending contract)
    #[arg(long, short)]
    contract_address: Option<String>,

    /// Processor type to use (block/tx/event/custom)
    #[arg(long, short, default_value = "all")]
    processor: String,

    /// Name of the processor to use (required for custom processor)
    #[arg(long)]
    processor_name: Option<String>,

    /// Directory to scan for processor modules (default: current directory)
    #[arg(long)]
    processors_dir: Option<PathBuf>,
}

// Define a function pointer type for processor factory
type ProcessorFactory = fn(Arc<DbPool>, Option<serde_json::Value>) -> Box<dyn ProcessorTrait>;

// Factory registry to hold all available processor factories
struct FactoryRegistry {
    factories: std::collections::HashMap<String, ProcessorFactory>,
    // Map of processor name to source file path
    source_files: std::collections::HashMap<String, PathBuf>,
}

impl FactoryRegistry {
    fn new() -> Self {
        let mut registry = FactoryRegistry {
            factories: std::collections::HashMap::new(),
            source_files: std::collections::HashMap::new(),
        };
        
        // Register built-in factories
        registry.register("lending", create_lending_processor);
        
        registry
    }
    
    fn register(&mut self, name: &str, factory: ProcessorFactory) {
        self.factories.insert(name.to_string(), factory);
    }
    
    fn get(&self, name: &str) -> Option<&ProcessorFactory> {
        self.factories.get(name)
    }
    
    // Get the source file path for a processor
    fn get_source_file(&self, name: &str) -> Option<&PathBuf> {
        self.source_files.get(name)
    }
    

    // Discover processor factories from a directory
    fn list_available_processors(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
    
    fn list_all_processors(&self) -> Vec<String> {
        let mut all_processors = self.factories.keys().cloned().collect::<Vec<_>>();
        all_processors.extend(self.source_files.keys().cloned());
        all_processors
    }

     /// Discover processor factories from a directory
     fn discover_factories(&mut self, dir: Option<&Path>) {
        let search_dir = match dir {
            Some(path) => path.to_path_buf(),
            None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };

        tracing::info!("Scanning for processor modules in {:?}", search_dir);

        // Walk the directory and find .rs files
        if let Ok(entries) = fs::read_dir(&search_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                
                // Only consider .rs files
                if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    // Skip the main file itself
                    if path.file_name().map_or(false, |name| name == "main.rs") {
                        continue;
                    }

                    if let Some(processor_name) = self.extract_processor_name(&path) {
                        tracing::info!("Found potential processor: {} at {:?}", processor_name, path);
                        self.source_files.insert(processor_name, path);
                    }
                }
            }
        } else {
            tracing::warn!("Failed to read directory: {:?}", search_dir);
        }
    }

     /// Extract processor name from file path by analyzing the file content
     fn extract_processor_name(&self, path: &Path) -> Option<String> {
        // Read file content
        let mut file = match fs::File::open(path) {
            Ok(file) => file,
            Err(err) => {
                tracing::warn!("Failed to open file {:?}: {}", path, err);
                return None;
            }
        };

        let mut content = String::new();
        if let Err(err) = file.read_to_string(&mut content) {
            tracing::warn!("Failed to read file {:?}: {}", path, err);
            return None;
        }

        // Extract processor name using regular expressions
        // Look for struct implementations that might be processors
        let struct_regex = Regex::new(r"struct\s+(\w+)(?:Processor)?\s*(?:\{|:|$)").ok()?;
        let impl_regex = Regex::new(r"impl\s+(?:dyn\s+)?ProcessorTrait\s+for\s+(\w+)").ok()?;
        
        // First try to find implementations of ProcessorTrait
        if let Some(captures) = impl_regex.captures(&content) {
            if let Some(name) = captures.get(1) {
                let processor_name = name.as_str().to_lowercase();
                // If the name ends with "processor", strip it off
                return Some(processor_name.trim_end_matches("processor").to_string());
            }
        }
        
        // Fallback to struct definitions that might be processors
        if let Some(captures) = struct_regex.captures(&content) {
            if let Some(name) = captures.get(1) {
                let processor_name = name.as_str().to_lowercase();
                return Some(processor_name.trim_end_matches("processor").to_string());
            }
        }

        // Extract filename without extension as a last resort
        path.file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_lowercase())
    }
}

// // Function to create a lending processor
// fn create_lending_processor(pool: Arc<DbPool>, args: Option<serde_json::Value>) -> Box<dyn ProcessorTrait> {
//     let contract_address = match args {
//         Some(v) => match v.get("contract_address") {
//             Some(addr) => match addr.as_str() {
//                 Some(s) => s.to_string(),
//                 None => panic!("Contract address must be a string")
//             },
//             None => panic!("Contract address is required")
//         },
//         None => panic!("Args is required for custom processor")
//     };
    
//     Box::new(lending_example::LendingContractProcessor::new(pool, contract_address))
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Setup logger
    tracing_subscriber::fmt().init();

    let args = Args::parse();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Convert network string to Network enum
    let network = match args.network.to_lowercase().as_str() {
        "mainnet" => Network::Mainnet,
        "testnet" => Network::Testnet,
        _ => return Err("Invalid network. Must be either 'mainnet' or 'testnet'".into()),
    };

    // Initialize the factory registry
    let mut registry = FactoryRegistry::new();
    
    // Discover additional processors if a directory is specified
    if let Some(dir) = args.processors_dir.as_ref() {
        registry.discover_factories(Some(dir));
    }

    // Determine which processors to use based on the processor argument
    let processor_configs = match args.processor.to_lowercase().as_str() {
        "all" => vec![
            ProcessorConfig::BlockProcessor,
            ProcessorConfig::TxProcessor,
            ProcessorConfig::EventProcessor,
        ],
        "block" => vec![ProcessorConfig::BlockProcessor],
        "tx" => vec![ProcessorConfig::TxProcessor],
        "event" => vec![ProcessorConfig::EventProcessor],
        "custom" => {
            if let Some(processor_name) = args.processor_name.as_ref() {
                if let Some(factory) = registry.get(processor_name) {
                    // Use an existing registered factory
                    if let Some(contract_address) = args.contract_address.clone() {
                        vec![ProcessorConfig::Custom {
                            name: format!("{} processor", processor_name),
                            factory: *factory,
                            args: Some(json!({
                                "contract_address": contract_address
                            })),
                        }]
                    } else {
                        return Err("Contract address is required for custom processor".into());
                    }
                } else if let Some(source_file) = registry.get_source_file(processor_name) {
                    // We found the source file but need to compile and load it
                    tracing::info!("Found processor source at {:?}, attempting to use it", source_file);
                    
                    // For now, we'll use a placeholder approach
                    // In a real implementation, you'd need to import and use the module
                    // This would require a more complex build system integration
                    
                    if processor_name == "lending_contract" || processor_name == "lending" {
                        // As a fallback, use our built-in lending processor
                        // This simulates finding and using the processor from the source
                        if let Some(contract_address) = args.contract_address.clone() {
                            tracing::info!("Using lending processor from discovered source");
                            vec![ProcessorConfig::Custom {
                                name: format!("{} processor", processor_name),
                                factory: create_lending_processor,
                                args: Some(json!({
                                    "contract_address": contract_address
                                })),
                            }]
                        } else {
                            return Err("Contract address is required for custom processor".into());
                        }
                    } else {
                        // In a real implementation, you would:
                        // 1. Add the source file to your build
                        // 2. Import the module
                        // 3. Use the factory function
                        
                        return Err(format!(
                            "Found processor '{}' at {:?} but dynamic loading is not yet implemented",
                            processor_name, source_file
                        ).into());
                    }
                } else {
                    let available = registry.list_available_processors();
                    let available_sources: Vec<String> = registry.source_files.keys().cloned().collect();
                    
                    return Err(format!(
                        "Processor '{}' not found. Available registered processors: {:?}\nAvailable source processors: {:?}",
                        processor_name, available, available_sources
                    ).into());
                }
            } else {
                return Err("Processor name is required for custom processor. Use --processor-name".into());
            }
        },
        _ => return Err("Invalid processor type. Must be one of: all, block, tx, event, custom".into()),
    };

    // Create worker with selected processor types
    let worker = Worker::new(
        processor_configs.clone(),
        database_url,
        network,
        None,
        Some(SyncOptions {
            start_ts: Some(args.from),
            step: Some(args.step),
            back_step: None,
            sync_duration: None,
        }),
        Some(bento_alephium::types::FetchStrategy::Parallel {
            num_workers: args.workers,
        }),
    )
    .await?;

    tracing::info!(
        "Starting backfill from {} to {} with step size {}ms and {} workers using processors: {:?}",
        args.from,
        args.to,
        args.step,
        args.workers,
        processor_configs
    );

    let _ = worker.run().await;
    Ok(())
}