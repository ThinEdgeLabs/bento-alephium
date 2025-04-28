use lending_example::processor_factory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bento_cli::run_command(processor_factory()).await?;
    Ok(())
}
