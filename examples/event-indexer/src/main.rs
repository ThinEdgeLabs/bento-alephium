use bento_core::{
    config::ProcessorConfig,
    workers::worker::{SyncOptions, Worker},
};
use bento_types::network::Network;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let processor_config = ProcessorConfig::EventProcessor;

    let worker = Worker::new(
        vec![processor_config],
        database_url,
        Network::Mainnet,
        None,
        Some(SyncOptions {
            start_ts: Some(1716560632750),
            stop_ts: None,
            step: 1000,
            request_interval: 1000,
        }),
        2,
    )
    .await?;

    let _ = worker.run().await;
    Ok(())
}
