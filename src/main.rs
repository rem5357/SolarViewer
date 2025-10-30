use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod schema;

#[derive(Parser)]
#[command(name = "solarviewer")]
#[command(about = "Extract and visualize stellar cartography data from Astrosynthesis", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Explore and document the schema of an Astrosynthesis .AstroDB file
    Schema {
        /// Path to the .AstroDB file
        #[arg(short, long)]
        file: String,

        /// Output path for schema documentation
        #[arg(short, long, default_value = "docs/SCHEMA.md")]
        output: String,
    },

    /// Extract data from an Astrosynthesis file and load into PostgreSQL
    Import {
        /// Path to the .AstroDB file
        #[arg(short, long)]
        file: String,

        /// Name for this dataset in the database
        #[arg(short, long)]
        name: String,

        /// PostgreSQL connection string
        #[arg(short, long, default_value = "postgresql://localhost/solarviewer")]
        database: String,
    },

    /// Generate a 2D map from a stored dataset
    Map {
        /// Name of the dataset to visualize
        #[arg(short, long)]
        name: String,

        /// Output file path (SVG or PNG)
        #[arg(short, long)]
        output: String,

        /// Layout algorithm: pca, force, mds, or hybrid
        #[arg(short, long, default_value = "hybrid")]
        algorithm: String,

        /// PostgreSQL connection string
        #[arg(short, long, default_value = "postgresql://localhost/solarviewer")]
        database: String,
    },
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Schema { file, output } => {
            info!("Exploring schema of: {}", file);
            info!("Output will be written to: {}", output);

            // Open database and explore schema
            let explorer = schema::SchemaExplorer::new(&file)?;
            info!("Connected to database");

            // Discover all tables and their structure
            let tables = explorer.explore()?;
            info!("Discovered {} tables", tables.len());

            // Generate markdown documentation
            schema::generate_markdown(&tables, &output, &file)?;
            info!("Schema documentation written to: {}", output);

            println!("✓ Schema exploration complete!");
            println!("  Tables discovered: {}", tables.len());
            println!("  Documentation: {}", output);
        }

        Commands::Import { file, name, database } => {
            info!("Importing {} as '{}'", file, name);
            info!("Target database: {}", database);

            // TODO: Implement data import
            println!("Data import not yet implemented");
            println!("This will:");
            println!("  1. Read all data from the .AstroDB file");
            println!("  2. Transform coordinates if needed");
            println!("  3. Validate data integrity");
            println!("  4. Load into PostgreSQL with source tracking");
        }

        Commands::Map { name, output, algorithm, database } => {
            info!("Generating map for dataset: {}", name);
            info!("Using {} algorithm", algorithm);
            info!("Output file: {}", output);

            // TODO: Implement map generation
            println!("Map generation not yet implemented");
            println!("This will:");
            println!("  1. Query star positions from PostgreSQL");
            println!("  2. Apply {} projection algorithm", algorithm);
            println!("  3. Resolve overlaps");
            println!("  4. Render to {}", output);
        }
    }

    Ok(())
}
