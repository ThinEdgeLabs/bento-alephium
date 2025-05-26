use std::collections::HashMap;

use linx_indexer::processors::transfer_processor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut processor_factories = HashMap::new();
    processor_factories.insert("transfers".to_string(), transfer_processor::processor_factory());

    //processor_factories.insert("swaps".to_string(), swaps::processor_factory());
    //processor_factories.insert("transfers".to_string(), swaps::processor_factory());
    //processor_factories.insert("lending".to_string(), linx_lending::processor_factory());
    dotenvy::dotenv().ok();
    bento_cli::run_command(processor_factories, true).await?;
    Ok(())
}
