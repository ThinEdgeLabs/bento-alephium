use std::collections::HashMap;

use linx_indexer::{
    processors::contract_call_processor, processors::dex_processor, processors::transfer_processor,
    routers::AccountTransactionApiModule,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let mut processor_factories = HashMap::new();
    processor_factories.insert("transfers".to_string(), transfer_processor::processor_factory());
    processor_factories
        .insert("contract_calls".to_string(), contract_call_processor::processor_factory());
    processor_factories.insert("dex".to_string(), dex_processor::processor_factory());
    let router = Some(AccountTransactionApiModule::register());
    bento_cli::run_command(processor_factories, router, true).await?;

    Ok(())
}
