// StellarForge CLI - Command-line interface for stellar cartography database

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable logging
    env_logger::init();

    // Run the CLI
    solarviewer::stellar_forge::cli::run_cli().await
}