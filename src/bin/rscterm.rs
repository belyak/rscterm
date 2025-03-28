use rscterm_cli::CliInterface;
use rscterm_provider::LMStudioProvider;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .init();

    // Create provider
    let provider = Box::new(LMStudioProvider::new(
        "http://localhost:1234".to_string(),
        "gemma-3-12b-it".to_string(),
    ));

    // Create and run CLI
    let mut cli = CliInterface::new(provider);
    cli.run().await?;

    Ok(())
} 