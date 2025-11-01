use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod schema;
mod extraction;
mod visualization;
mod stellar_forge;

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

    /// Extract stars and save to CSV
    Extract {
        /// Path to the .AstroDB file
        #[arg(short, long)]
        file: String,

        /// Output CSV file path
        #[arg(short, long, default_value = "stars.csv")]
        output: String,
    },

    /// Analyze multi-star systems in an Astrosynthesis file
    Multistar {
        /// Path to the .AstroDB file
        #[arg(short, long)]
        file: String,
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

    /// Render a star map to PNG from an Astrosynthesis file
    Render {
        /// Path to the .AstroDB file
        #[arg(short, long)]
        file: String,

        /// Center star name (e.g., "Amateru")
        #[arg(short, long)]
        star: String,

        /// Radius around center star in light-years
        #[arg(short, long, default_value = "25")]
        radius: f64,

        /// Output PNG file path
        #[arg(short, long, default_value = "star_map.png")]
        output: String,

        /// Image width in pixels
        #[arg(long, default_value = "5000")]
        width: u32,

        /// Image height in pixels
        #[arg(long, default_value = "5000")]
        height: u32,

        /// Maximum distance for star connections in light-years
        #[arg(long, default_value = "7")]
        connection_distance: f64,
    },

    /// Create a new StellarForge galaxy
    CreateGalaxy {
        /// Name of the galaxy
        #[arg(short, long)]
        name: String,

        /// Output file path for the galaxy data
        #[arg(short, long, default_value = "galaxy.json")]
        output: String,

        /// Number of random systems to generate
        #[arg(long, default_value = "100")]
        systems: usize,

        /// Random seed for generation
        #[arg(long, default_value = "42")]
        seed: u64,
    },

    /// Create a Sol-like system using StellarForge
    CreateSol {
        /// Output file path for the system data
        #[arg(short, long, default_value = "sol_system.json")]
        output: String,
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

        Commands::Extract { file, output } => {
            info!("Extracting stars from: {}", file);
            info!("Output will be written to: {}", output);

            // Open database and read stars
            let reader = extraction::StarReader::new(&file)?;
            info!("Connected to database");

            // Count stars
            let count = reader.count_stars()?;
            info!("Found {} stars", count);

            // Read all stars
            let stars = reader.read_all_stars()?;
            info!("Read {} stars", stars.len());

            // Export to CSV
            extraction::export_stars_to_csv(&stars, &output)?;
            info!("Stars exported to CSV: {}", output);

            println!("✓ Star extraction complete!");
            println!("  Stars extracted: {}", stars.len());
            println!("  CSV file: {}", output);
        }

        Commands::Multistar { file } => {
            info!("Analyzing multi-star systems in: {}", file);
            extraction::analyze_multistar_systems(&file)?;
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

        Commands::Render { file, star, radius, output, width, height, connection_distance } => {
            info!("Rendering star map for: {}", star);
            info!("File: {}", file);
            info!("Radius: {} ly", radius);
            info!("Output: {}", output);

            visualization::render_star_map(
                &file,
                &star,
                radius,
                &output,
                width,
                height,
                connection_distance,
            )?;

            println!("✓ Star map rendering complete!");
            println!("  Center star: {}", star);
            println!("  Search radius: {} ly", radius);
            println!("  Output: {}", output);
        }

        Commands::CreateGalaxy { name, output, systems, seed } => {
            use stellar_forge::{builders::GalaxyBuilder, storage::{StellarForgeDataset, FileStorage}};

            info!("Creating new galaxy: {}", name);
            info!("Generating {} systems with seed {}", systems, seed);

            let galaxy = GalaxyBuilder::new(&name)
                .with_size(100000.0, 100000.0, 1000.0)
                .with_random_systems(systems, seed)
                .build();

            let mut dataset = StellarForgeDataset::new(galaxy);
            dataset.metadata.name = name.clone();
            dataset.metadata.description = Some(format!("Generated galaxy with {} systems", systems));
            dataset.update_statistics();

            FileStorage::save_json(&dataset, &output)?;

            println!("✓ Galaxy creation complete!");
            println!("  Galaxy name: {}", name);
            println!("  Systems generated: {}", systems);
            println!("  Total stars: {}", dataset.metadata.statistics.total_stars);
            println!("  Output: {}", output);
        }

        Commands::CreateSol { output } => {
            use stellar_forge::{
                builders::create_sol_like_system,
                containers::Galaxy,
                storage::{StellarForgeDataset, FileStorage},
                services::StellarForgeService,
            };

            info!("Creating Sol-like system");

            // Create the Sol system
            let sol_system = create_sol_like_system()
                .at_position(8000.0, 0.0, 0.0)  // ~8 kpc from galactic center
                .build();

            // Create a galaxy and add the system
            let mut galaxy = Galaxy::new("Milky Way");
            galaxy.add_star_system(sol_system)?;

            // Create dataset
            let mut dataset = StellarForgeDataset::new(galaxy);
            dataset.metadata.name = "Sol System".to_string();
            dataset.metadata.description = Some("Sol system with all planets and major moons".to_string());
            dataset.update_statistics();

            // Save to file
            FileStorage::save_json(&dataset, &output)?;

            println!("✓ Sol system creation complete!");
            println!("  Stars: {}", dataset.metadata.statistics.total_stars);
            println!("  Planets: {}", dataset.metadata.statistics.total_planets);
            println!("  Moons: {}", dataset.metadata.statistics.total_moons);
            println!("  Output: {}", output);
        }
    }

    Ok(())
}
