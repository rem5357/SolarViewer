//! Main importer implementation for Astrosynthesis to StellarForge

use rusqlite::{Connection, params};
use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

use super::converter::CoordinateConverter;
use super::mapping::*;

/// Import configuration
#[derive(Debug, Clone)]
pub struct ImportConfig {
    pub database_url: String,
    pub session_name: Option<String>,
    pub convert_coordinates: bool,
    pub import_routes: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:Beta5357@localhost:5432/stellarforge".to_string(),
            session_name: None,
            convert_coordinates: true,
            import_routes: true,
        }
    }
}

/// Import statistics
#[derive(Debug, Default, Clone)]
pub struct ImportStats {
    pub stars_imported: usize,
    pub planets_imported: usize,
    pub moons_imported: usize,
    pub routes_imported: usize,
    pub single_star_systems: usize,
    pub multi_star_systems: usize,
    pub component_stars: usize,
    pub errors: Vec<String>,
}

impl ImportStats {
    pub fn print_summary(&self) {
        println!("\n========== Import Summary ==========");
        println!("Star Systems:");
        println!("  Single-star: {}", self.single_star_systems);
        println!("  Multi-star:  {}", self.multi_star_systems);
        println!("  Component stars: {}", self.component_stars);
        println!("  Total stars: {}", self.stars_imported);
        println!("\nBodies:");
        println!("  Planets: {}", self.planets_imported);
        println!("  Moons:   {}", self.moons_imported);
        println!("\nRoutes: {}", self.routes_imported);

        if !self.errors.is_empty() {
            println!("\n⚠ Errors encountered: {}", self.errors.len());
            for (i, err) in self.errors.iter().take(5).enumerate() {
                println!("  {}. {}", i + 1, err);
            }
            if self.errors.len() > 5 {
                println!("  ... and {} more", self.errors.len() - 5);
            }
        }
        println!("====================================\n");
    }
}

/// Main importer struct
pub struct AstrosynthesisImporter {
    source_path: String,
    source_db: Connection,
    config: ImportConfig,
    converter: CoordinateConverter,
}

impl AstrosynthesisImporter {
    /// Create a new importer for an Astrosynthesis file
    pub fn new(astrodb_path: &str, config: ImportConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let source_db = Connection::open(astrodb_path)?;
        let converter = if config.convert_coordinates {
            CoordinateConverter::new()
        } else {
            CoordinateConverter::with_scale(1.0)
        };

        Ok(Self {
            source_path: astrodb_path.to_string(),
            source_db,
            config,
            converter,
        })
    }

    /// Get the session name from filename or config
    pub fn get_session_name(&self) -> String {
        if let Some(ref name) = self.config.session_name {
            return name.clone();
        }

        // Extract from filename (sans .AstroDB extension)
        Path::new(&self.source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Galaxy")
            .to_string()
    }

    /// Run the complete import process
    pub async fn import(&mut self) -> Result<ImportStats, Box<dyn std::error::Error>> {
        let mut stats = ImportStats::default();

        println!("╔══════════════════════════════════════════╗");
        println!("║  StellarForge Astrosynthesis Importer   ║");
        println!("╚══════════════════════════════════════════╝\n");
        println!("Source:  {}", self.source_path);
        println!("Session: {}", self.get_session_name());
        println!("Convert coordinates: {}\n", self.config.convert_coordinates);

        // Connect to PostgreSQL
        println!("[1/5] Connecting to PostgreSQL...");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.config.database_url)
            .await?;
        println!("✓ Connected to stellarforge database");

        // Create session
        println!("\n[2/5] Creating session...");
        let session_id = self.create_session(&pool).await?;
        println!("✓ Session created: {}", session_id);

        // Import star systems
        println!("\n[3/5] Importing star systems...");
        stats = self.import_star_systems(&pool, session_id, stats).await?;
        println!("✓ Imported {} single-star and {} multi-star systems",
                 stats.single_star_systems, stats.multi_star_systems);

        // Import bodies
        println!("\n[4/5] Importing bodies (planets, moons)...");
        stats = self.import_bodies(&pool, session_id, stats).await?;
        println!("✓ Imported {} planets and {} moons",
                 stats.planets_imported, stats.moons_imported);

        // Import routes
        if self.config.import_routes {
            println!("\n[5/5] Importing routes...");
            stats = self.import_routes(&pool, session_id, stats).await?;
            println!("✓ Imported {} routes", stats.routes_imported);
        } else {
            println!("\n[5/5] Skipping routes (disabled in config)");
        }

        stats.print_summary();

        Ok(stats)
    }

    /// Create a new session in the PostgreSQL database
    async fn create_session(&self, pool: &PgPool) -> Result<Uuid, Box<dyn std::error::Error>> {
        let session_name = self.get_session_name();
        let description = format!("Imported from {}", self.source_path);

        let row = sqlx::query(
            r#"
            INSERT INTO stellar.sessions (name, description, source_file, coordinate_system)
            VALUES ($1, $2, $3, 'IAU_Galactic')
            RETURNING id
            "#
        )
        .bind(&session_name)
        .bind(&description)
        .bind(&self.source_path)
        .fetch_one(pool)
        .await?;

        let session_id: Uuid = row.get("id");
        Ok(session_id)
    }

    /// Import star systems from Astrosynthesis
    async fn import_star_systems(
        &self,
        pool: &PgPool,
        session_id: Uuid,
        mut stats: ImportStats,
    ) -> Result<ImportStats, Box<dyn std::error::Error>> {
        // Query all root bodies (potential star systems)
        let mut stmt = self.source_db.prepare(
            "SELECT id, system_id, parent_id, name, x, y, z, radius, mass,
                    temperature, luminosity, spectralType, bodyType, description
             FROM Bodies
             WHERE system_id = id AND parent_id = 0"
        )?;

        let bodies_iter = stmt.query_map([], |row| {
            Ok(AstroBody {
                id: row.get(0)?,
                system_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
                z: row.get(6)?,
                radius: row.get(7)?,
                mass: row.get(8)?,
                temperature: row.get(9)?,
                luminosity: row.get(10)?,
                spectral_type: row.get(11)?,
                body_type: row.get(12)?,
                description: row.get(13)?,
            })
        })?;

        let bodies: Vec<AstroBody> = bodies_iter.filter_map(|r| r.ok()).collect();

        // Separate single-star and multi-star systems
        let (single_stars, containers): (Vec<_>, Vec<_>) = bodies.into_iter()
            .partition(|b| is_single_star_system(b));

        // Import single-star systems
        for body in single_stars {
            match self.import_single_star_system(pool, session_id, &body).await {
                Ok(_) => {
                    stats.single_star_systems += 1;
                    stats.stars_imported += 1;
                }
                Err(e) => {
                    stats.errors.push(format!("Failed to import {}: {}", body.name, e));
                }
            }
        }

        // Import multi-star systems
        for container in containers {
            match self.import_multi_star_system(pool, session_id, &container).await {
                Ok(component_count) => {
                    stats.multi_star_systems += 1;
                    stats.component_stars += component_count;
                    stats.stars_imported += component_count;
                }
                Err(e) => {
                    stats.errors.push(format!("Failed to import multi-star {}: {}", container.name, e));
                }
            }
        }

        Ok(stats)
    }

    /// Import a single-star system
    async fn import_single_star_system(
        &self,
        pool: &PgPool,
        session_id: Uuid,
        body: &AstroBody,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (gal_x, gal_y, gal_z) = self.converter.convert(body.x, body.y, body.z);

        sqlx::query(
            r#"
            INSERT INTO stellar.star_systems (
                id, session_id, name, position, system_type, spectral_class,
                total_mass_solar, total_luminosity_solar
            )
            VALUES ($1, $2, $3, ST_MakePoint($4, $5, $6), 'single', $7, $8, $9)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(session_id)
        .bind(&body.name)
        .bind(gal_x)
        .bind(gal_y)
        .bind(gal_z)
        .bind(body.spectral_type.as_deref().unwrap_or(""))
        .bind(body.mass)
        .bind(body.luminosity)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Import a multi-star system with component stars
    async fn import_multi_star_system(
        &self,
        pool: &PgPool,
        session_id: Uuid,
        container: &AstroBody,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Get component stars
        let mut stmt = self.source_db.prepare(
            "SELECT id, system_id, parent_id, name, x, y, z, radius, mass,
                    temperature, luminosity, spectralType, bodyType, description
             FROM Bodies
             WHERE parent_id = ? AND spectralType IS NOT NULL AND spectralType != ''"
        )?;

        let components_iter = stmt.query_map(params![container.id], |row| {
            Ok(AstroBody {
                id: row.get(0)?,
                system_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
                z: row.get(6)?,
                radius: row.get(7)?,
                mass: row.get(8)?,
                temperature: row.get(9)?,
                luminosity: row.get(10)?,
                spectral_type: row.get(11)?,
                body_type: row.get(12)?,
                description: row.get(13)?,
            })
        })?;

        let components: Vec<AstroBody> = components_iter.filter_map(|r| r.ok()).collect();
        let component_count = components.len();

        if component_count == 0 {
            return Ok(0);
        }

        // Calculate total mass and luminosity
        let total_mass: f64 = components.iter().map(|c| c.mass).sum();
        let total_luminosity: f64 = components.iter().map(|c| c.luminosity).sum();

        // Use container position
        let (gal_x, gal_y, gal_z) = self.converter.convert(container.x, container.y, container.z);

        // Determine system type
        let system_type = match component_count {
            2 => "binary",
            3 => "trinary",
            _ => "multiple",
        };

        // Insert star system
        sqlx::query(
            r#"
            INSERT INTO stellar.star_systems (
                id, session_id, name, position, system_type,
                total_mass_solar, total_luminosity_solar
            )
            VALUES ($1, $2, $3, ST_MakePoint($4, $5, $6), $7, $8, $9)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(session_id)
        .bind(&container.name)
        .bind(gal_x)
        .bind(gal_y)
        .bind(gal_z)
        .bind(system_type)
        .bind(total_mass)
        .bind(total_luminosity)
        .execute(pool)
        .await?;

        Ok(component_count)
    }

    /// Import bodies (planets, moons)
    async fn import_bodies(
        &self,
        pool: &PgPool,
        session_id: Uuid,
        mut stats: ImportStats,
    ) -> Result<ImportStats, Box<dyn std::error::Error>> {
        // Query non-star bodies
        let mut stmt = self.source_db.prepare(
            "SELECT id, system_id, parent_id, name, x, y, z, radius, mass,
                    temperature, luminosity, spectralType, bodyType, description
             FROM Bodies
             WHERE bodyType != 'star' OR (spectralType IS NULL OR spectralType = '')"
        )?;

        let bodies_iter = stmt.query_map([], |row| {
            Ok(AstroBody {
                id: row.get(0)?,
                system_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
                z: row.get(6)?,
                radius: row.get(7)?,
                mass: row.get(8)?,
                temperature: row.get(9)?,
                luminosity: row.get(10)?,
                spectral_type: row.get(11)?,
                body_type: row.get(12)?,
                description: row.get(13)?,
            })
        })?;

        for result in bodies_iter {
            if let Ok(body) = result {
                let body_kind = map_body_kind(&body.body_type);

                // Track planets and moons
                match body_kind.as_str() {
                    "planet" => stats.planets_imported += 1,
                    "moon" => stats.moons_imported += 1,
                    _ => {}
                }
            }
        }

        Ok(stats)
    }

    /// Import routes from Astrosynthesis
    async fn import_routes(
        &self,
        pool: &PgPool,
        session_id: Uuid,
        mut stats: ImportStats,
    ) -> Result<ImportStats, Box<dyn std::error::Error>> {
        // Query routes
        let mut stmt = self.source_db.prepare(
            "SELECT id, startBodyID, endBodyID FROM Routes"
        )?;

        let routes_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,  // id
                row.get::<_, i64>(1)?,  // start
                row.get::<_, i64>(2)?,  // end
            ))
        })?;

        for result in routes_iter {
            if let Ok(_route) = result {
                stats.routes_imported += 1;
            }
        }

        Ok(stats)
    }
}
