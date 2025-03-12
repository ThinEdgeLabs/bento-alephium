use bento_alephium::{
    client::Network,
    config::ProcessorConfig, workers::worker_v2::{SyncOptions, Worker},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Setup logger
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let worker = Worker::new(
        vec![ProcessorConfig::BlockProcessor],
        database_url,
        Network::Testnet,
        None,
        Some(SyncOptions {
            start_ts: Some(1716560632750),
            step: Some(1000),
            back_step: None,
            sync_duration: None,
        }),
        None
    )
    .await?;

    worker.run().await;
    Ok(())
}
