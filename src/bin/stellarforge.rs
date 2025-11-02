// StellarForge CLI - Command-line interface for stellar cartography database

use anyhow::Result;

// Import the library crate
extern crate solarviewer;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable logging
    env_logger::init();

    // Run the CLI
    solarviewer::stellar_forge::cli::run_cli().await
}