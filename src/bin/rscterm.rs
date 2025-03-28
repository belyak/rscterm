use rscterm_cli::CliInterface;
use rscterm_provider::LMStudioProvider;
use rscterm_core::Provider;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .init();

    // Initialize the provider with LM Studio configuration
    let provider = Box::new(LMStudioProvider::new(
        "http://10.6.1.238:1234".to_string(),
        "granite-3.2-8b-instruct".to_string(),
    ));

    // Create and run the CLI interface
    let mut cli = CliInterface::new(provider);
    cli.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use std::sync::atomic::{AtomicBool, Ordering};

    static INIT: Once = Once::new();
    static LOGGING_INITIALIZED: AtomicBool = AtomicBool::new(false);

    fn setup_logging() {
        INIT.call_once(|| {
            fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_ansi(true)
                .init();
            LOGGING_INITIALIZED.store(true, Ordering::SeqCst);
        });
    }

    #[test]
    fn test_logging_initialization() {
        setup_logging();
        assert!(LOGGING_INITIALIZED.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_provider_initialization() {
        let provider = Box::new(LMStudioProvider::new(
            "http://10.6.1.238:1234".to_string(),
            "granite-3.2-8b-instruct".to_string(),
        ));
        assert_eq!(provider.get_current_model(), "granite-3.2-8b-instruct");
    }

    #[tokio::test]
    async fn test_cli_interface_creation() {
        let provider = Box::new(LMStudioProvider::new(
            "http://10.6.1.238:1234".to_string(),
            "granite-3.2-8b-instruct".to_string(),
        ));
        let cli = CliInterface::new(provider);
        assert!(cli.get_current_team().is_none());
        assert!(cli.get_show_progress());
    }
} 