use crate::{
    client::Client,
    config::ProcessorConfig,
    db::{new_db_pool, DbPool},
};
use bento_types::{
    network::Network, repository::processor_status::update_last_timestamp, BlockRange,
    FetchStrategy,
};

use anyhow::Result;

use super::{fetch::fetch_parallel, pipeline::Pipeline};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep as tokio_sleep;

pub struct Worker {
    pub db_pool: Arc<DbPool>,
    pub client: Arc<Client>,
    pub processor_configs: Vec<ProcessorConfig>,
    pub db_url: String,
    pub sync_opts: SyncOptions,
    pub fetch_strategy: FetchStrategy,
}

impl Worker {
    pub async fn new(
        processor_configs: Vec<ProcessorConfig>,
        db_url: String,
        network: Network,
        db_pool_size: Option<u32>,
        sync_opts: Option<SyncOptions>,
        fetch_strategy: Option<FetchStrategy>,
    ) -> Result<Self> {
        let db_pool = new_db_pool(&db_url, db_pool_size).await?;
        Ok(Self {
            db_pool: db_pool.clone(),
            processor_configs,
            db_url,
            sync_opts: sync_opts.unwrap_or_default(),
            client: Arc::new(Client::new(network)),
            fetch_strategy: fetch_strategy.unwrap_or(FetchStrategy::Simple),
        })
    }

    pub async fn run(&self) -> Result<()> {
        self.run_migrations().await;

        let mut current_ts = self.sync_opts.start_ts;
        let step = self.sync_opts.step;
        let request_interval = Duration::from_millis(self.sync_opts.request_interval);

        loop {
            let to_ts = match self.sync_opts.stop_ts {
                Some(stop_ts) if current_ts + step > stop_ts => stop_ts,
                _ => current_ts + step,
            };

            let range = BlockRange {
                from_ts: current_ts.try_into().unwrap(),
                to_ts: to_ts.try_into().unwrap(),
            };

            tracing::info!("Fetching blocks from {} to {}...", current_ts, to_ts);

            // Fetch blocks once for all processors
            let batches =
                match fetch_parallel(self.client.clone(), range, self.fetch_strategy.num_workers())
                    .await
                {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!(
                            error = ?e,
                            "Failed to fetch blocks, retrying in {:?}",
                            request_interval
                        );
                        tokio_sleep(request_interval).await;
                        continue;
                    }
                };
            // Process batches with each processor
            let mut processors_results = Vec::new();

            for processor_config in &self.processor_configs {
                let pool_clone = self.db_pool.clone();
                let client_clone = self.client.clone();
                let processor = processor_config.build_processor(pool_clone.clone());
                let processor_name = processor.name().to_string();
                let batches_clone = batches.clone();

                // Create a pipeline for each processor
                let pipeline = Pipeline::new(client_clone.clone(), pool_clone.clone(), processor);

                // Run the pipeline with the fetched batches
                let result = tokio::spawn(async move {
                    match pipeline.run(batches_clone).await {
                        Ok(_) => {
                            // Update the timestamp for this processor
                            if let Err(update_err) = update_last_timestamp(
                                &pool_clone,
                                &processor_name,
                                (client_clone.network.clone()).into(),
                                to_ts.try_into().unwrap(),
                            )
                            .await
                            {
                                tracing::error!(
                                    processor_name = processor_name,
                                    error = ?update_err,
                                    "Failed to update timestamp"
                                );
                                return Err(anyhow::anyhow!(
                                    "Failed to update timestamp: {}",
                                    update_err
                                ));
                            }
                            Ok(())
                        }
                        Err(err) => {
                            tracing::error!(
                                processor_name = processor_name,
                                error = ?err,
                                "Pipeline execution failed"
                            );
                            Err(anyhow::anyhow!("Pipeline execution failed: {}", err))
                        }
                    }
                });

                processors_results.push(result);
            }

            // Wait for all processors to complete
            let mut all_successful = true;
            for result in futures::future::join_all(processors_results).await {
                match result {
                    Ok(Ok(_)) => {} // Processor completed successfully
                    Ok(Err(e)) => {
                        tracing::error!(error = ?e, "Processor failed");
                        all_successful = false;
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "Task panicked");
                        all_successful = false;
                    }
                }
            }

            // Only advance to next block range if all processors succeeded
            if all_successful {
                current_ts = to_ts;

                // Check if we've reached the stop_ts (if specified)
                if let Some(stop_ts) = self.sync_opts.stop_ts {
                    if current_ts >= stop_ts {
                        tracing::info!("Reached stop timestamp {}, exiting", stop_ts);
                        break;
                    }
                }
            }

            tracing::debug!("Sleeping for {:?}...", request_interval);
            tokio_sleep(request_interval).await;
        }

        Ok(())
    }

    // For the normal processor build we just use standard Diesel with the postgres
    // feature enabled (which uses libpq under the hood, hence why we named the feature
    // this way).
    #[cfg(feature = "libpq")]
    async fn run_migrations(&self) {
        use diesel::{pg::PgConnection, Connection};

        use crate::db::run_pending_migrations;

        tracing::info!("Running migrations: {:?}", self.db_url);
        let mut conn = PgConnection::establish(&self.db_url).expect("migrations failed!");
        run_pending_migrations(&mut conn);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SyncOptions {
    pub start_ts: u64,
    pub stop_ts: Option<u64>,
    pub step: u64,
    pub request_interval: u64,
}
