use anyhow::Result;
use aibitatr::cli::runner::CliRunner;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with environment-based filter
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut runner = CliRunner::new();
    runner.run().await?;
    Ok(())
} 