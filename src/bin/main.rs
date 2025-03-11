use bento_alephium::{api::index::start, config::Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    println!("Starting server...");
    let config = Config::from_env().await?;
    println!("Server is ready and running on http://{}", config.api_endpoint());
    println!("Swagger UI is available at http://{}/swagger-ui", config.api_endpoint());
    start(config).await?;

    Ok(())
}
