use std::collections::HashMap;

use linx_indexer::{processors::transfer_processor, routers::AccountTransactionApiModule};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let mut processor_factories = HashMap::new();
    processor_factories.insert("transfers".to_string(), transfer_processor::processor_factory());
    let router = Some(AccountTransactionApiModule::register());
    bento_cli::run_command(processor_factories, router, true).await?;

    Ok(())
}
