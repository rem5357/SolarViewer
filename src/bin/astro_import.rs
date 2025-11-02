//! Standalone Astrosynthesis to StellarForge importer
//!
//! This is a simplified, standalone version that imports Astrosynthesis data
//! directly into PostgreSQL without depending on the full stellar_forge modules.

use rusqlite::{Connection, params};
use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use clap::Parser;
use std::path::Path;
use uuid::Uuid;
use nalgebra::Vector3;

#[derive(Parser)]
#[command(name = "astro-import")]
#[command(about = "Import Astrosynthesis .AstroDB files into StellarForge database")]
struct Args {
    /// Path to the Astrosynthesis .AstroDB file
    #[arg(short, long)]
    file: String,

    /// Session name (defaults to filename without extension)
    #[arg(short, long)]
    session: Option<String>,

    /// PostgreSQL connection URL
    #[arg(short, long, default_value = "postgresql://postgres:Beta5357@localhost:5432/stellarforge")]
    database: String,

    /// Convert coordinates from Astrosynthesis to IAU Galactic
    #[arg(long, default_value = "true")]
    convert_coords: bool,
}

#[derive(Debug, Default)]
struct Stats {
    single_stars: usize,
    multi_stars: usize,
    component_stars: usize,
    planets: usize,
    moons: usize,
    routes: usize,
    errors: Vec<String>,
}

impl Stats {
    fn print(&self) {
        println!("\n╔══════════════════════════════════════╗");
        println!("║        Import Summary                ║");
        println!("╚══════════════════════════════════════╝");
        println!("Star Systems:");
        println!("  Single-star:     {}", self.single_stars);
        println!("  Multi-star:      {}", self.multi_stars);
        println!("  Component stars: {}", self.component_stars);
        println!("  Total stars:     {}", self.single_stars + self.component_stars);
        println!("\nBodies:");
        println!("  Planets: {}", self.planets);
        println!("  Moons:   {}", self.moons);
        println!("\nRoutes: {}", self.routes);

        if !self.errors.is_empty() {
            println!("\n⚠ Errors: {}", self.errors.len());
            for (i, err) in self.errors.iter().take(3).enumerate() {
                println!("  {}. {}", i + 1, err);
            }
            if self.errors.len() > 3 {
                println!("  ... and {} more", self.errors.len() - 3);
            }
        }
        println!("╚══════════════════════════════════════╝\n");
    }
}

fn convert_coords(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    // Astro (X,Y,Z) -> Galactic (Z,X,Y)
    (z, x, y)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("╔══════════════════════════════════════════╗");
    println!("║  Astrosynthesis → StellarForge Import   ║");
    println!("╚══════════════════════════════════════════╝\n");

    // Determine session name
    let session_name = args.session.unwrap_or_else(|| {
        Path::new(&args.file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Galaxy")
            .to_string()
    });

    println!("Source:   {}", args.file);
    println!("Session:  {}", session_name);
    println!("Database: {}", args.database);
    println!("Convert:  {}\n", args.convert_coords);

    // Open SQLite source
    println!("[1/5] Opening Astrosynthesis file...");
    let sqlite = Connection::open(&args.file)?;
    println!("✓ Connected to {}", args.file);

    // Connect to PostgreSQL
    println!("\n[2/5] Connecting to PostgreSQL...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&args.database)
        .await?;
    println!("✓ Connected to stellarforge database");

    // Create session
    println!("\n[3/5] Creating session...");
    let metadata = serde_json::json!({
        "source_file": args.file,
        "coordinate_system": if args.convert_coords { "IAU_Galactic" } else { "Astrosynthesis" },
        "import_timestamp": chrono::Utc::now().to_rfc3339()
    });

    let session_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO stellar.sessions (name, description, session_type, metadata)
        VALUES ($1, $2, 'import', $3)
        RETURNING id
        "#
    )
    .bind(&session_name)
    .bind(format!("Imported from {}", args.file))
    .bind(&metadata)
    .fetch_one(&pool)
    .await?;
    println!("✓ Session created: {}", session_id);

    // Import star systems
    println!("\n[4/5] Importing star systems...");
    let mut stats = Stats::default();
    stats = import_stars(&sqlite, &pool, session_id, args.convert_coords, stats).await?;
    println!("✓ Imported {} single and {} multi-star systems",
             stats.single_stars, stats.multi_stars);

    // Import bodies
    println!("\n[5/5] Importing bodies...");
    stats = import_bodies(&sqlite, &pool, session_id, args.convert_coords, stats).await?;
    println!("✓ Imported {} planets and {} moons", stats.planets, stats.moons);

    stats.print();
    println!("✅ Import complete!");
    println!("\n💡 Connect to database in DBeaver:");
    println!("   Host: localhost:5432");
    println!("   Database: stellarforge");
    println!("   Username: postgres");
    println!("   Password: Beta5357\n");

    Ok(())
}

async fn import_stars(
    sqlite: &Connection,
    pool: &PgPool,
    session_id: Uuid,
    convert: bool,
    mut stats: Stats,
) -> Result<Stats, Box<dyn std::error::Error>> {
    // Query root bodies (potential star systems)
    let mut stmt = sqlite.prepare(
        "SELECT id, system_id, parent_id, name, x, y, z, mass, luminosity, spectral, radius, temp
         FROM Bodies
         WHERE system_id = id AND parent_id = 0"
    )?;

    let systems = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,       // id
            row.get::<_, String>(3)?,    // name
            row.get::<_, f64>(4)?,       // x
            row.get::<_, f64>(5)?,       // y
            row.get::<_, f64>(6)?,       // z
            row.get::<_, f64>(7)?,       // mass
            row.get::<_, f64>(8)?,       // luminosity
            row.get::<_, Option<String>>(9)?, // spectral
            row.get::<_, f64>(10)?,      // radius
            row.get::<_, f64>(11)?,      // temp
        ))
    })?;

    for result in systems {
        if let Ok((astro_id, name, x, y, z, mass, lum, spectral, radius, temp)) = result {
            let (gx, gy, gz) = if convert {
                convert_coords(x, y, z)
            } else {
                (x, y, z)
            };

            // Check if multi-star container
            let is_container = spectral.is_none() || spectral.as_deref() == Some("");

            if is_container {
                // Multi-star system
                let mut comp_stmt = sqlite.prepare(
                    "SELECT name, spectral, mass, luminosity, radius, temp
                     FROM Bodies
                     WHERE parent_id = ? AND spectral IS NOT NULL AND spectral != ''"
                )?;

                let components: Vec<_> = comp_stmt.query_map(params![astro_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,   // name
                        row.get::<_, String>(1)?,   // spectral
                        row.get::<_, f64>(2)?,      // mass
                        row.get::<_, f64>(3)?,      // luminosity
                        row.get::<_, f64>(4)?,      // radius
                        row.get::<_, f64>(5)?,      // temp
                    ))
                })?.filter_map(|r| r.ok()).collect();

                if !components.is_empty() {
                    let comp_count = components.len();
                    let sys_type = if comp_count == 2 { "binary" } else { "multiple" };

                    // Create system container (no mass/luminosity - that's on the stars!)
                    let system_uuid: Uuid = sqlx::query_scalar(
                        r#"
                        INSERT INTO stellar.star_systems (
                            session_id, name, position, system_type
                        )
                        VALUES ($1, $2, ST_MakePoint($3, $4, $5), $6::stellar.system_type)
                        RETURNING id
                        "#
                    )
                    .bind(session_id)
                    .bind(&name)
                    .bind(gx)
                    .bind(gy)
                    .bind(gz)
                    .bind(sys_type)
                    .fetch_one(pool)
                    .await?;

                    // Now create the actual star records in stellar.stars table
                    for (comp_name, comp_spec, comp_mass, comp_lum, comp_radius, comp_temp) in components {
                        // Ensure temperature is valid (> 0)
                        let temp = if comp_temp <= 0.0 { 5778.0 } else { comp_temp }; // Default to Sun's temp

                        sqlx::query(
                            r#"
                            INSERT INTO stellar.stars (
                                session_id, system_id, name,
                                spectral_class, mass_solar, radius_solar,
                                luminosity_solar, temperature_k,
                                position_absolute
                            )
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, ST_MakePoint($9, $10, $11))
                            "#
                        )
                        .bind(session_id)
                        .bind(system_uuid)
                        .bind(&comp_name)
                        .bind(&comp_spec)
                        .bind(comp_mass)
                        .bind(comp_radius)
                        .bind(comp_lum)
                        .bind(temp)
                        .bind(gx)
                        .bind(gy)
                        .bind(gz)
                        .execute(pool)
                        .await?;

                        stats.component_stars += 1;
                    }

                    stats.multi_stars += 1;
                }
            } else {
                // Single star system - create container
                let system_uuid: Uuid = sqlx::query_scalar(
                    r#"
                    INSERT INTO stellar.star_systems (
                        session_id, name, position, system_type
                    )
                    VALUES ($1, $2, ST_MakePoint($3, $4, $5), 'single')
                    RETURNING id
                    "#
                )
                .bind(session_id)
                .bind(&name)
                .bind(gx)
                .bind(gy)
                .bind(gz)
                .fetch_one(pool)
                .await?;

                // Create the star record in stellar.stars table
                let spec = spectral.unwrap_or_else(|| "G2V".to_string());
                let temp_valid = if temp <= 0.0 { 5778.0 } else { temp }; // Default to Sun's temp

                sqlx::query(
                    r#"
                    INSERT INTO stellar.stars (
                        session_id, system_id, name,
                        spectral_class, mass_solar, radius_solar,
                        luminosity_solar, temperature_k,
                        position_absolute
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, ST_MakePoint($9, $10, $11))
                    "#
                )
                .bind(session_id)
                .bind(system_uuid)
                .bind(&name)
                .bind(&spec)
                .bind(mass)
                .bind(radius)
                .bind(lum)
                .bind(temp_valid)
                .bind(gx)
                .bind(gy)
                .bind(gz)
                .execute(pool)
                .await?;

                stats.single_stars += 1;
            }
        }
    }

    Ok(stats)
}

async fn import_bodies(
    sqlite: &Connection,
    pool: &PgPool,
    session_id: Uuid,
    _convert: bool,
    mut stats: Stats,
) -> Result<Stats, Box<dyn std::error::Error>> {
    // First, create mappings of Astrosynthesis IDs to our UUIDs
    let system_map: std::collections::HashMap<i64, Uuid> = {
        let mut map = std::collections::HashMap::new();

        // Get all root bodies from Astro
        let mut astro_stmt = sqlite.prepare(
            "SELECT id FROM Bodies WHERE system_id = id AND parent_id = 0"
        )?;
        let astro_ids: Vec<i64> = astro_stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Get corresponding UUIDs from PostgreSQL (by name match)
        for astro_id in astro_ids {
            let name: String = sqlite.query_row(
                "SELECT name FROM Bodies WHERE id = ?",
                params![astro_id],
                |row| row.get(0)
            )?;

            if let Ok(uuid) = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM stellar.star_systems WHERE session_id = $1 AND name = $2"
            )
            .bind(session_id)
            .bind(&name)
            .fetch_one(pool)
            .await {
                map.insert(astro_id, uuid);
            }
        }
        map
    };

    // Create a map of Astrosynthesis Body IDs to determine which are stars
    // Stars have spectral class, planets/moons don't
    let star_astro_ids: std::collections::HashSet<i64> = {
        let mut stmt = sqlite.prepare(
            "SELECT id FROM Bodies WHERE spectral IS NOT NULL AND spectral != ''"
        )?;
        let ids: Vec<i64> = stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        ids.into_iter().collect()
    };

    // Create a map to track planet IDs as we create them (Astro ID -> PG UUID)
    let mut planet_map: std::collections::HashMap<i64, Uuid> = std::collections::HashMap::new();

    // Query ALL non-star bodies (planets, moons, etc.)
    // Parent_id != 0 AND no spectral class = planet or moon
    let mut stmt = sqlite.prepare(
        "SELECT id, system_id, parent_id, name, mass, radius, temp
         FROM Bodies
         WHERE parent_id != 0 AND (spectral IS NULL OR spectral = '')"
    )?;

    let bodies: Vec<_> = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,       // id
            row.get::<_, i64>(1)?,       // system_id
            row.get::<_, i64>(2)?,       // parent_id
            row.get::<_, String>(3)?,    // name
            row.get::<_, f64>(4)?,       // mass
            row.get::<_, f64>(5)?,       // radius
            row.get::<_, f64>(6)?,       // temp
        ))
    })?.filter_map(|r| r.ok()).collect();

    // First pass: Import planets (bodies whose parent is a star)
    for (astro_id, astro_sys_id, parent_id, name, mass, radius, temp) in &bodies {
        // Check if parent is a star
        if star_astro_ids.contains(parent_id) {
            if let Some(&system_uuid) = system_map.get(astro_sys_id) {
                // Get the parent star's UUID from PostgreSQL
                let parent_name: Option<String> = sqlite.query_row(
                    "SELECT name FROM Bodies WHERE id = ?",
                    params![parent_id],
                    |row| row.get(0)
                ).ok();

                if let Some(parent_star_name) = parent_name {
                    let parent_star_id: Option<Uuid> = sqlx::query_scalar(
                        "SELECT id FROM stellar.stars WHERE session_id = $1 AND name = $2"
                    )
                    .bind(session_id)
                    .bind(&parent_star_name)
                    .fetch_optional(pool)
                    .await?;

                    if let Some(star_id) = parent_star_id {
                        // Handle invalid mass/radius values (must be > 0)
                        let mass_valid = if *mass <= 0.0 { 1.0 } else { *mass }; // Default to 1 Earth mass
                        let radius_valid = if *radius <= 0.0 { 1.0 } else { *radius }; // Default to 1 Earth radius

                        let planet_id: Uuid = sqlx::query_scalar(
                            r#"
                            INSERT INTO stellar.planets (
                                session_id, system_id, parent_star_id, name,
                                planet_type, radius_earth, mass_earth,
                                semi_major_axis_au, eccentricity, orbital_period_days,
                                surface_temperature_k
                            )
                            VALUES ($1, $2, $3, $4, 'terrestrial', $5, $6, 1.0, 0.0, 365.0, $7)
                            RETURNING id
                            "#
                        )
                        .bind(session_id)
                        .bind(system_uuid)
                        .bind(star_id)
                        .bind(name)
                        .bind(radius_valid)
                        .bind(mass_valid)
                        .bind(*temp)
                        .fetch_one(pool)
                        .await?;

                        planet_map.insert(*astro_id, planet_id);
                        stats.planets += 1;
                    }
                }
            }
        }
    }

    // Second pass: Import moons (bodies whose parent is a planet)
    for (_astro_id, astro_sys_id, parent_id, name, mass, radius, temp) in &bodies {
        // Check if parent is a planet we just created
        if let Some(&parent_planet_id) = planet_map.get(parent_id) {
            if let Some(&system_uuid) = system_map.get(astro_sys_id) {
                // Handle invalid values
                let radius_km = if *radius <= 0.0 { 1737.0 } else { radius * 6371.0 }; // Default to Moon's radius (1737 km)
                let mass_valid = if *mass <= 0.0 { 0.0123 } else { *mass }; // Default to Moon's mass (0.0123 Earth masses)

                sqlx::query(
                    r#"
                    INSERT INTO stellar.moons (
                        session_id, system_id, parent_planet_id, name,
                        radius_km, mass_earth,
                        semi_major_axis_km, eccentricity, orbital_period_days,
                        surface_temperature_k
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, 384400.0, 0.0, 27.0, $7)
                    "#
                )
                .bind(session_id)
                .bind(system_uuid)
                .bind(parent_planet_id)
                .bind(name)
                .bind(radius_km)
                .bind(mass_valid)
                .bind(*temp)
                .execute(pool)
                .await?;

                stats.moons += 1;
            }
        }
    }

    Ok(stats)
}
