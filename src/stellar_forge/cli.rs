// Command-line interface for StellarForge database operations

use anyhow::Result;
use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::stellar_forge::{
    database::{
        ConnectionPool, SessionRepository, SystemRepository,
        BodyRepository, PoliticalRepository, RouteRepository,
        queries::{SpatialQueries, AnalyticalQueries},
        migrations,
    },
    coordinates::GalacticCoordinates,
    core::Units,
};

#[derive(Parser)]
#[clap(author, version, about = "StellarForge - Stellar Cartography Database Manager")]
struct Cli {
    /// Database connection URL
    #[clap(short, long, env = "DATABASE_URL")]
    database_url: Option<String>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the database with PostGIS and all tables
    Init {
        /// Reset database if it exists (DANGEROUS!)
        #[clap(long)]
        reset: bool,
    },

    /// Session management commands
    Session {
        #[clap(subcommand)]
        action: SessionCommands,
    },

    /// Import data from Astrosynthesis
    Import {
        /// Session name for the import
        #[clap(short, long)]
        session_name: String,

        /// Path to Astrosynthesis AstroDB file
        #[clap(short, long)]
        file: String,

        /// Convert from Astrosynthesis coordinates
        #[clap(long)]
        convert_coordinates: bool,
    },

    /// Star system operations
    System {
        #[clap(subcommand)]
        action: SystemCommands,
    },

    /// Political entity operations
    Political {
        #[clap(subcommand)]
        action: PoliticalCommands,
    },

    /// Route operations
    Route {
        #[clap(subcommand)]
        action: RouteCommands,
    },

    /// Spatial analysis queries
    Analyze {
        #[clap(subcommand)]
        query: AnalysisCommands,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Create a new session
    Create {
        #[clap(short, long)]
        name: String,

        #[clap(short, long)]
        description: Option<String>,

        #[clap(long, default_value = "full_galaxy")]
        session_type: String,
    },

    /// List all sessions
    List,

    /// Show session details
    Show {
        #[clap(short, long)]
        id: Uuid,
    },

    /// Create a subsection from an existing session
    Subsection {
        #[clap(short, long)]
        parent_id: Uuid,

        #[clap(short, long)]
        name: String,

        /// Center X coordinate in light years
        #[clap(long)]
        center_x: f64,

        /// Center Y coordinate in light years
        #[clap(long)]
        center_y: f64,

        /// Center Z coordinate in light years
        #[clap(long)]
        center_z: f64,

        /// Radius in light years
        #[clap(short, long)]
        radius: f64,
    },
}

#[derive(Subcommand)]
enum SystemCommands {
    /// Add a star system
    Add {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(short, long)]
        name: String,

        /// X coordinate in light years
        #[clap(long)]
        x: f64,

        /// Y coordinate in light years
        #[clap(long)]
        y: f64,

        /// Z coordinate in light years
        #[clap(long)]
        z: f64,

        #[clap(long, default_value = "single")]
        system_type: String,
    },

    /// Find systems within radius
    Near {
        #[clap(short, long)]
        session_id: Uuid,

        /// Center X coordinate in light years
        #[clap(long)]
        x: f64,

        /// Center Y coordinate in light years
        #[clap(long)]
        y: f64,

        /// Center Z coordinate in light years
        #[clap(long)]
        z: f64,

        /// Search radius in light years
        #[clap(short, long)]
        radius: f64,
    },

    /// Find nearest neighbors to a system
    Neighbors {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(long)]
        system_id: Uuid,

        #[clap(short, long, default_value = "10")]
        limit: i32,
    },
}

#[derive(Subcommand)]
enum PoliticalCommands {
    /// Create a political entity
    Create {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(short, long)]
        name: String,

        #[clap(short, long, default_value = "republic")]
        government_type: String,
    },

    /// Add a system to political control
    Control {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(short, long)]
        entity_id: Uuid,

        #[clap(long)]
        system_id: Uuid,

        #[clap(long, default_value = "sovereign")]
        control_type: String,

        #[clap(long, default_value = "1.0")]
        strength: f64,
    },

    /// Generate influence zone for an entity
    Influence {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(short, long)]
        entity_id: Uuid,

        #[clap(short, long, default_value = "10.0")]
        base_radius: f64,
    },

    /// Find disputed territories
    Disputes {
        #[clap(short, long)]
        session_id: Uuid,
    },

    /// Find contested systems
    Contested {
        #[clap(short, long)]
        session_id: Uuid,
    },
}

#[derive(Subcommand)]
enum RouteCommands {
    /// Create a route through systems
    Create {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(short, long)]
        name: String,

        /// System IDs in order
        #[clap(long, value_delimiter = ',')]
        systems: Vec<Uuid>,

        #[clap(long, default_value = "trade")]
        route_type: String,
    },

    /// List all routes
    List {
        #[clap(short, long)]
        session_id: Uuid,
    },

    /// Find systems along a route
    Along {
        #[clap(short, long)]
        route_id: Uuid,

        #[clap(long, default_value = "5.0")]
        max_distance: f64,
    },
}

#[derive(Subcommand)]
enum AnalysisCommands {
    /// Find strategic chokepoints
    Chokepoints {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(long, default_value = "3")]
        min_routes: i32,
    },

    /// Find frontier systems
    Frontier {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(long, default_value = "15.0")]
        neighbor_distance: f64,

        #[clap(long, default_value = "3")]
        max_neighbors: i32,
    },

    /// Calculate influence at a point
    Influence {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(long)]
        x: f64,

        #[clap(long)]
        y: f64,

        #[clap(long)]
        z: f64,
    },

    /// Get density distribution
    Density {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(long, default_value = "50.0")]
        grid_size: f64,
    },

    /// Political power rankings
    Rankings {
        #[clap(short, long)]
        session_id: Uuid,
    },

    /// Find safe route avoiding hostile space
    SafeRoute {
        #[clap(short, long)]
        session_id: Uuid,

        #[clap(long)]
        from: Uuid,

        #[clap(long)]
        to: Uuid,

        #[clap(long)]
        avoid: Uuid,

        #[clap(long, default_value = "10")]
        max_jumps: i32,
    },
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    // Default database URL if not provided
    let database_url = cli.database_url.unwrap_or_else(|| {
        "postgresql://postgres:Beta5357@localhost/stellarforge".to_string()
    });

    // Create connection pool
    let pool = ConnectionPool::new(&database_url).await?;

    match cli.command {
        Commands::Init { reset } => {
            println!("Initializing StellarForge database...");

            if reset {
                println!("WARNING: Resetting database - all data will be lost!");
                migrations::reset_database(pool.pool()).await?;
            } else {
                if migrations::check_migrations_needed(pool.pool()).await? {
                    migrations::run_migrations(pool.pool()).await?;
                    println!("Database initialized successfully");
                } else {
                    println!("Database already initialized");
                }
            }

            // Check PostGIS
            if pool.check_postgis().await? {
                let version = pool.postgis_version().await?;
                println!("PostGIS version: {}", version);
            } else {
                println!("WARNING: PostGIS not installed!");
            }
        }

        Commands::Session { action } => {
            handle_session_command(&pool, action).await?;
        }

        Commands::Import { session_name, file, convert_coordinates } => {
            use crate::stellar_forge::import::{AstrosynthesisImporter, ImportConfig};

            let config = ImportConfig {
                database_url: db_url.clone(),
                session_name: Some(session_name),
                convert_coordinates,
                import_routes: true,
            };

            let mut importer = AstrosynthesisImporter::new(&file, config)?;
            let stats = importer.import().await?;

            println!("\n✅ Import complete!");
        }

        Commands::System { action } => {
            handle_system_command(&pool, action).await?;
        }

        Commands::Political { action } => {
            handle_political_command(&pool, action).await?;
        }

        Commands::Route { action } => {
            handle_route_command(&pool, action).await?;
        }

        Commands::Analyze { query } => {
            handle_analysis_command(&pool, query).await?;
        }
    }

    Ok(())
}

async fn handle_session_command(pool: &ConnectionPool, action: SessionCommands) -> Result<()> {
    let repo = SessionRepository::new(pool);

    match action {
        SessionCommands::Create { name, description, session_type } => {
            let id = repo.create_session(&name, description.as_deref(), &session_type).await?;
            println!("Created session: {}", id);
            println!("Name: {}", name);
            if let Some(desc) = description {
                println!("Description: {}", desc);
            }
        }

        SessionCommands::List => {
            let sessions = repo.list_sessions().await?;
            println!("Found {} sessions:", sessions.len());
            for session in sessions {
                println!("  {} - {} ({})", session.id, session.name, session.session_type);
                if let Some(systems) = session.total_systems {
                    println!("    Systems: {}", systems);
                }
            }
        }

        SessionCommands::Show { id } => {
            let session = repo.get_session(id).await?;
            println!("Session: {}", session.name);
            println!("ID: {}", session.id);
            println!("Type: {}", session.session_type);
            if let Some(desc) = session.description {
                println!("Description: {}", desc);
            }
            println!("Created: {}", session.created_at);

            // Statistics
            if let Some(systems) = session.total_systems {
                println!("\nStatistics:");
                println!("  Total systems: {}", systems);
            }
            if let Some(stars) = session.total_stars {
                println!("  Total stars: {}", stars);
            }
            if let Some(planets) = session.total_planets {
                println!("  Total planets: {}", planets);
            }
            if let Some(populated) = session.total_populated_worlds {
                println!("  Populated worlds: {}", populated);
            }
        }

        SessionCommands::Subsection { parent_id, name, center_x, center_y, center_z, radius } => {
            let id = repo.create_subsection(
                parent_id, &name, center_x, center_y, center_z, radius
            ).await?;
            println!("Created subsection: {}", id);
            println!("Name: {}", name);
            println!("Center: ({:.2}, {:.2}, {:.2}) ly", center_x, center_y, center_z);
            println!("Radius: {:.2} ly", radius);
        }
    }

    Ok(())
}

async fn handle_system_command(pool: &ConnectionPool, action: SystemCommands) -> Result<()> {
    let repo = SystemRepository::new(pool);

    match action {
        SystemCommands::Add { session_id, name, x, y, z, system_type } => {
            let id = repo.create_system(session_id, &name, x, y, z, &system_type).await?;

            // Calculate galactic coordinates for display
            let cart = nalgebra::Vector3::new(
                x * Units::LIGHT_YEAR,
                y * Units::LIGHT_YEAR,
                z * Units::LIGHT_YEAR,
            );
            let galactic = GalacticCoordinates::from_cartesian(cart);

            println!("Created system: {}", id);
            println!("Name: {}", name);
            println!("Position: ({:.2}, {:.2}, {:.2}) ly", x, y, z);
            println!("Galactic: l={:.2}°, b={:.2}°, d={:.2} ly",
                galactic.longitude_deg(),
                galactic.latitude_deg(),
                galactic.distance_ly()
            );
        }

        SystemCommands::Near { session_id, x, y, z, radius } => {
            let systems = repo.find_systems_within(session_id, x, y, z, radius).await?;
            println!("Found {} systems within {:.2} ly of ({:.2}, {:.2}, {:.2}):",
                systems.len(), radius, x, y, z
            );

            for system in systems {
                println!("  {} - {}", system.id, system.name);
                if let Some(d) = system.distance_from_sol_ly {
                    println!("    Distance from Sol: {:.2} ly", d);
                }
            }
        }

        SystemCommands::Neighbors { session_id, system_id, limit } => {
            let neighbors = repo.find_nearest_systems(session_id, system_id, limit).await?;
            println!("Nearest {} neighbors:", neighbors.len());

            for (system, distance) in neighbors {
                println!("  {} - {} ({:.2} ly)", system.id, system.name, distance);
            }
        }
    }

    Ok(())
}

async fn handle_political_command(pool: &ConnectionPool, action: PoliticalCommands) -> Result<()> {
    let repo = PoliticalRepository::new(pool);

    match action {
        PoliticalCommands::Create { session_id, name, government_type } => {
            let id = repo.create_entity(session_id, &name, &government_type).await?;
            println!("Created political entity: {}", id);
            println!("Name: {}", name);
            println!("Government: {}", government_type);
        }

        PoliticalCommands::Control { session_id, entity_id, system_id, control_type, strength } => {
            repo.add_system_control(
                session_id, entity_id, system_id, &control_type, strength
            ).await?;
            println!("Added system control");
            println!("Entity: {}", entity_id);
            println!("System: {}", system_id);
            println!("Control: {} (strength: {:.2})", control_type, strength);
        }

        PoliticalCommands::Influence { session_id, entity_id, base_radius } => {
            repo.generate_influence_zone(session_id, entity_id, base_radius).await?;
            println!("Generated influence zone");
            println!("Entity: {}", entity_id);
            println!("Base radius: {:.2} ly", base_radius);
        }

        PoliticalCommands::Disputes { session_id } => {
            let disputes = repo.find_disputed_zones(session_id).await?;
            println!("Found {} disputed zones:", disputes.len());

            for (entity_a, entity_b) in disputes {
                println!("  {} <-> {}", entity_a, entity_b);
            }
        }

        PoliticalCommands::Contested { session_id } => {
            let queries = SpatialQueries::new(pool.pool());
            let contested = queries.find_contested_systems(session_id).await?;

            println!("Found {} contested systems:", contested.len());
            for (id, name, claims) in contested {
                println!("  {} - {} ({} claims)", id, name, claims);
            }
        }
    }

    Ok(())
}

async fn handle_route_command(pool: &ConnectionPool, action: RouteCommands) -> Result<()> {
    let repo = RouteRepository::new(pool);

    match action {
        RouteCommands::Create { session_id, name, systems, route_type } => {
            let id = repo.create_route_from_systems(
                session_id, &name, &systems, &route_type
            ).await?;
            println!("Created route: {}", id);
            println!("Name: {}", name);
            println!("Type: {}", route_type);
            println!("Waypoints: {} systems", systems.len());
        }

        RouteCommands::List { session_id } => {
            let routes = repo.get_routes(session_id).await?;
            println!("Found {} routes:", routes.len());

            for route in routes {
                println!("  {} - {}", route.id, route.name);
                if let Some(active) = route.is_active {
                    if active {
                        print!(" [ACTIVE]");
                    }
                }
                if let Some(trade) = route.trade_value_credits {
                    println!("    Trade value: {} credits", trade);
                }
            }
        }

        RouteCommands::Along { route_id, max_distance } => {
            let queries = SpatialQueries::new(pool.pool());
            let systems = queries.systems_along_route(route_id, max_distance).await?;

            println!("Found {} systems within {:.2} ly of route:",
                systems.len(), max_distance
            );
            for (id, name, distance) in systems {
                println!("  {} - {} ({:.2} ly)", id, name, distance);
            }
        }
    }

    Ok(())
}

async fn handle_analysis_command(pool: &ConnectionPool, query: AnalysisCommands) -> Result<()> {
    match query {
        AnalysisCommands::Chokepoints { session_id, min_routes } => {
            let queries = SpatialQueries::new(pool.pool());
            let chokepoints = queries.find_chokepoints(session_id, min_routes).await?;

            println!("Strategic chokepoints (>= {} routes):", min_routes);
            for (id, name, routes, trade) in chokepoints {
                println!("  {} - {}", id, name);
                println!("    Routes: {}", routes);
                println!("    Trade value: {:.0} credits", trade);
            }
        }

        AnalysisCommands::Frontier { session_id, neighbor_distance, max_neighbors } => {
            let queries = SpatialQueries::new(pool.pool());
            let frontier = queries.find_frontier_systems(
                session_id, neighbor_distance, max_neighbors
            ).await?;

            println!("Frontier systems (<= {} neighbors within {:.2} ly):",
                max_neighbors, neighbor_distance
            );
            for (id, name, neighbors) in frontier {
                println!("  {} - {} ({} neighbors)", id, name, neighbors);
            }
        }

        AnalysisCommands::Influence { session_id, x, y, z } => {
            let queries = SpatialQueries::new(pool.pool());
            let influences = queries.calculate_influence_at_point(
                session_id, x, y, z
            ).await?;

            println!("Political influence at ({:.2}, {:.2}, {:.2}):", x, y, z);
            for (id, name, strength) in influences {
                println!("  {} - {}: {:.2}%", id, name, strength * 100.0);
            }
        }

        AnalysisCommands::Density { session_id, grid_size } => {
            let queries = AnalyticalQueries::new(pool.pool());
            let distribution = queries.density_distribution(session_id, grid_size).await?;

            println!("Galactic density distribution (grid: {:.0} ly³):", grid_size);
            let top_10 = distribution.into_iter().take(10);
            for (x, y, z, count) in top_10 {
                println!("  ({:.0}, {:.0}, {:.0}): {} systems", x, y, z, count);
            }
        }

        AnalysisCommands::Rankings { session_id } => {
            let queries = AnalyticalQueries::new(pool.pool());
            let rankings = queries.political_power_rankings(session_id).await?;

            println!("Political Power Rankings:");
            for (i, (id, name, systems, pop, volume)) in rankings.iter().enumerate() {
                println!("{}. {} ({})", i + 1, name, id);
                println!("   Systems: {}", systems);
                println!("   Population: {}", pop);
                println!("   Territory: {:.0} ly³", volume);
            }
        }

        AnalysisCommands::SafeRoute { session_id, from, to, avoid, max_jumps } => {
            let queries = SpatialQueries::new(pool.pool());
            let path = queries.find_safe_route(
                session_id, from, to, avoid, max_jumps
            ).await?;

            if path.is_empty() {
                println!("No safe route found within {} jumps", max_jumps);
            } else {
                println!("Safe route found ({} jumps):", path.len() - 1);
                for (i, system_id) in path.iter().enumerate() {
                    println!("  {}. {}", i + 1, system_id);
                }
            }
        }
    }

    Ok(())
}