// Storage and persistence layer for StellarForge

use crate::stellar_forge::core::{Id, State, Vec3};
use crate::stellar_forge::bodies::{StellarBody, BodyKind};
use crate::stellar_forge::containers::{Galaxy, StarSystem, PoliticalRegion, Fleet};
use crate::stellar_forge::frames::Frame;
use crate::stellar_forge::associations::{Association, Tag};
use serde::{Deserialize, Serialize};
use std::path::Path;
use time::OffsetDateTime;

// Main storage format for complete datasets
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StellarForgeDataset {
    pub version: String,
    pub created_utc: OffsetDateTime,
    pub last_modified_utc: OffsetDateTime,
    pub galaxy: Galaxy,
    pub metadata: DatasetMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,  // e.g., "Imported from Astrosynthesis"
    pub tags: Vec<String>,
    pub statistics: DatasetStatistics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetStatistics {
    pub total_systems: usize,
    pub total_stars: usize,
    pub total_planets: usize,
    pub total_moons: usize,
    pub total_stations: usize,
    pub habitable_worlds: usize,
    pub populated_worlds: usize,
}

impl StellarForgeDataset {
    pub fn new(galaxy: Galaxy) -> Self {
        let stats = Self::calculate_statistics(&galaxy);

        Self {
            version: "1.0.0".to_string(),
            created_utc: OffsetDateTime::now_utc(),
            last_modified_utc: OffsetDateTime::now_utc(),
            galaxy,
            metadata: DatasetMetadata {
                name: "Untitled Dataset".to_string(),
                description: None,
                author: None,
                source: None,
                tags: Vec::new(),
                statistics: stats,
            },
        }
    }

    fn calculate_statistics(galaxy: &Galaxy) -> DatasetStatistics {
        let mut stats = DatasetStatistics {
            total_systems: galaxy.star_systems.len(),
            total_stars: 0,
            total_planets: 0,
            total_moons: 0,
            total_stations: 0,
            habitable_worlds: 0,
            populated_worlds: 0,
        };

        for system in &galaxy.star_systems {
            stats.total_stars += system.stars.len();
            stats.total_planets += system.planets.len();
            stats.total_stations += system.stations.len();

            for planet in &system.planets {
                // Count moons
                stats.total_moons += planet.children.iter()
                    .filter(|c| c.kind == BodyKind::Moon)
                    .count();

                // Check habitability
                if let Some(physical) = &planet.physical {
                    if let crate::stellar_forge::physical::Physical::Planet(p) = physical {
                        if let Some(hab) = p.habitability_score {
                            if hab > 0.5 {
                                stats.habitable_worlds += 1;
                            }
                        }
                        if let Some(pop) = p.population {
                            if pop > 0.0 {
                                stats.populated_worlds += 1;
                            }
                        }
                    }
                }
            }
        }

        stats
    }

    pub fn update_statistics(&mut self) {
        self.metadata.statistics = Self::calculate_statistics(&self.galaxy);
        self.last_modified_utc = OffsetDateTime::now_utc();
    }
}

// File I/O operations
pub struct FileStorage;

impl FileStorage {
    // Save to JSON file
    pub fn save_json(dataset: &StellarForgeDataset, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(dataset)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    // Load from JSON file
    pub fn load_json(path: impl AsRef<Path>) -> std::io::Result<StellarForgeDataset> {
        let json = std::fs::read_to_string(path)?;
        let dataset = serde_json::from_str(&json)?;
        Ok(dataset)
    }

    // Save to binary format (using bincode)
    #[cfg(feature = "binary")]
    pub fn save_binary(dataset: &StellarForgeDataset, path: impl AsRef<Path>) -> std::io::Result<()> {
        let bytes = bincode::serialize(dataset)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    // Load from binary format
    #[cfg(feature = "binary")]
    pub fn load_binary(path: impl AsRef<Path>) -> std::io::Result<StellarForgeDataset> {
        let bytes = std::fs::read(path)?;
        let dataset = bincode::deserialize(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(dataset)
    }

    // Export subset of data
    pub fn export_system(system: &StarSystem, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(system)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    // Import system into galaxy
    pub fn import_system(path: impl AsRef<Path>) -> std::io::Result<StarSystem> {
        let json = std::fs::read_to_string(path)?;
        let system = serde_json::from_str(&json)?;
        Ok(system)
    }
}

// Database repository traits
pub trait Repository<T> {
    type Error;

    fn insert(&mut self, entity: &T) -> Result<Id, Self::Error>;
    fn update(&mut self, entity: &T) -> Result<(), Self::Error>;
    fn delete(&mut self, id: Id) -> Result<(), Self::Error>;
    fn get(&self, id: Id) -> Result<Option<T>, Self::Error>;
    fn get_all(&self) -> Result<Vec<T>, Self::Error>;
}

// PostgreSQL repository implementation
#[cfg(feature = "postgres")]
pub mod postgres {
    use super::*;
    use sqlx::{Pool, Postgres, Row};
    use async_trait::async_trait;

    pub struct PostgresRepository {
        pool: Pool<Postgres>,
    }

    impl PostgresRepository {
        pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
            let pool = Pool::<Postgres>::connect(database_url).await?;
            Ok(Self { pool })
        }

        pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                CREATE EXTENSION IF NOT EXISTS postgis;

                CREATE TABLE IF NOT EXISTS frames (
                    id UUID PRIMARY KEY,
                    kind TEXT NOT NULL,
                    name TEXT NOT NULL,
                    parent_id UUID REFERENCES frames(id),
                    epoch TIMESTAMPTZ NOT NULL,
                    transform JSONB,
                    metadata JSONB
                );

                CREATE TABLE IF NOT EXISTS bodies (
                    id UUID PRIMARY KEY,
                    name TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    spatial_parent JSONB NOT NULL,
                    frame_id UUID NOT NULL REFERENCES frames(id),
                    epoch TIMESTAMPTZ NOT NULL,
                    state JSONB NOT NULL,
                    motion JSONB,
                    physical JSONB,
                    rotation_period_hours DOUBLE PRECISION,
                    axial_tilt_rad DOUBLE PRECISION,
                    retrograde_rotation BOOLEAN NOT NULL DEFAULT FALSE,
                    tags TEXT[] NOT NULL DEFAULT '{}',
                    associations JSONB NOT NULL DEFAULT '[]'::jsonb,
                    visible BOOLEAN NOT NULL DEFAULT TRUE,
                    color REAL[],
                    render_distance DOUBLE PRECISION,
                    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
                    position GEOMETRY(PointZ, 4326)
                );

                CREATE TABLE IF NOT EXISTS star_systems (
                    id UUID PRIMARY KEY,
                    name TEXT NOT NULL,
                    galactic_position GEOMETRY(PointZ, 4326) NOT NULL,
                    system_type TEXT NOT NULL,
                    barycenter GEOMETRY(PointZ, 4326),
                    frame_id UUID REFERENCES frames(id),
                    data JSONB NOT NULL
                );

                CREATE TABLE IF NOT EXISTS political_regions (
                    id UUID PRIMARY KEY,
                    name TEXT NOT NULL,
                    government_type TEXT NOT NULL,
                    capital_system_id UUID REFERENCES star_systems(id),
                    member_system_ids UUID[],
                    claimed_regions JSONB,
                    founded_date TIMESTAMPTZ,
                    properties JSONB
                );

                CREATE TABLE IF NOT EXISTS associations (
                    id TEXT PRIMARY KEY,
                    entity_id UUID NOT NULL,
                    association_type TEXT NOT NULL,
                    role TEXT NOT NULL,
                    group_name TEXT NOT NULL,
                    since_epoch TIMESTAMPTZ,
                    until_epoch TIMESTAMPTZ,
                    properties JSONB
                );

                -- Spatial indexes
                CREATE INDEX IF NOT EXISTS idx_bodies_position
                    ON bodies USING GIST (position);
                CREATE INDEX IF NOT EXISTS idx_systems_position
                    ON star_systems USING GIST (galactic_position);

                -- Other indexes
                CREATE INDEX IF NOT EXISTS idx_bodies_frame
                    ON bodies(frame_id);
                CREATE INDEX IF NOT EXISTS idx_bodies_tags
                    ON bodies USING GIN(tags);
                CREATE INDEX IF NOT EXISTS idx_bodies_kind
                    ON bodies(kind);
                CREATE INDEX IF NOT EXISTS idx_associations_entity
                    ON associations(entity_id);
                CREATE INDEX IF NOT EXISTS idx_associations_group
                    ON associations(group_name);
                "#
            )
            .execute(&self.pool)
            .await?;

            Ok(())
        }
    }

    #[async_trait]
    impl Repository<StellarBody> for PostgresRepository {
        type Error = sqlx::Error;

        async fn insert(&mut self, body: &StellarBody) -> Result<Id, Self::Error> {
            let position = format!(
                "POINT Z({} {} {})",
                body.state.position_m.x,
                body.state.position_m.y,
                body.state.position_m.z
            );

            sqlx::query(
                r#"
                INSERT INTO bodies (
                    id, name, kind, spatial_parent, frame_id, epoch,
                    state, motion, physical, rotation_period_hours,
                    axial_tilt_rad, retrograde_rotation, tags, associations,
                    visible, color, render_distance, metadata, position
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18,
                    ST_GeomFromText($19, 4326)
                )
                "#
            )
            .bind(body.id)
            .bind(&body.name)
            .bind(format!("{:?}", body.kind))
            .bind(serde_json::to_value(&body.spatial_parent).unwrap())
            .bind(body.frame_id)
            .bind(body.epoch)
            .bind(serde_json::to_value(&body.state).unwrap())
            .bind(serde_json::to_value(&body.motion).unwrap())
            .bind(serde_json::to_value(&body.physical).unwrap())
            .bind(body.rotation_period_hours)
            .bind(body.axial_tilt_rad)
            .bind(body.retrograde_rotation)
            .bind(&body.tags)
            .bind(serde_json::to_value(&body.associations).unwrap())
            .bind(body.visible)
            .bind(body.color.as_ref().map(|c| c.to_vec()))
            .bind(body.render_distance)
            .bind(serde_json::to_value(&body.metadata).unwrap())
            .bind(position)
            .execute(&self.pool)
            .await?;

            Ok(body.id)
        }

        async fn update(&mut self, body: &StellarBody) -> Result<(), Self::Error> {
            sqlx::query(
                r#"
                UPDATE bodies SET
                    name = $2,
                    state = $3,
                    motion = $4,
                    physical = $5,
                    tags = $6,
                    associations = $7,
                    metadata = $8
                WHERE id = $1
                "#
            )
            .bind(body.id)
            .bind(&body.name)
            .bind(serde_json::to_value(&body.state).unwrap())
            .bind(serde_json::to_value(&body.motion).unwrap())
            .bind(serde_json::to_value(&body.physical).unwrap())
            .bind(&body.tags)
            .bind(serde_json::to_value(&body.associations).unwrap())
            .bind(serde_json::to_value(&body.metadata).unwrap())
            .execute(&self.pool)
            .await?;

            Ok(())
        }

        async fn delete(&mut self, id: Id) -> Result<(), Self::Error> {
            sqlx::query("DELETE FROM bodies WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;
            Ok(())
        }

        async fn get(&self, id: Id) -> Result<Option<StellarBody>, Self::Error> {
            // This would require proper deserialization from the database
            // For now, returning a placeholder
            Ok(None)
        }

        async fn get_all(&self) -> Result<Vec<StellarBody>, Self::Error> {
            // This would query and deserialize all bodies
            Ok(Vec::new())
        }
    }

    // Spatial queries using PostGIS
    impl PostgresRepository {
        pub async fn find_bodies_within(
            &self,
            center: Vec3,
            radius_m: f64,
        ) -> Result<Vec<Id>, sqlx::Error> {
            let rows = sqlx::query(
                r#"
                SELECT id FROM bodies
                WHERE ST_DWithin(
                    position,
                    ST_GeomFromText($1, 4326),
                    $2
                )
                "#
            )
            .bind(format!("POINT Z({} {} {})", center.x, center.y, center.z))
            .bind(radius_m)
            .fetch_all(&self.pool)
            .await?;

            Ok(rows.iter().map(|r| r.get::<Id, _>("id")).collect())
        }

        pub async fn find_nearest_bodies(
            &self,
            point: Vec3,
            limit: i32,
        ) -> Result<Vec<(Id, f64)>, sqlx::Error> {
            let rows = sqlx::query(
                r#"
                SELECT id, ST_Distance(position, ST_GeomFromText($1, 4326)) as distance
                FROM bodies
                ORDER BY position <-> ST_GeomFromText($1, 4326)
                LIMIT $2
                "#
            )
            .bind(format!("POINT Z({} {} {})", point.x, point.y, point.z))
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

            Ok(rows
                .iter()
                .map(|r| (r.get::<Id, _>("id"), r.get::<f64, _>("distance")))
                .collect())
        }
    }
}

// Import/Export utilities
pub struct ImportExport;

impl ImportExport {
    // Export to CSV format (simplified)
    pub fn export_to_csv(galaxy: &Galaxy, path: impl AsRef<Path>) -> std::io::Result<()> {
        use std::io::Write;

        let mut file = std::fs::File::create(path)?;

        // Header
        writeln!(file, "System,Star,Spectral,X_LY,Y_LY,Z_LY,Planets,Habitable")?;

        for system in &galaxy.star_systems {
            let pos = system.galactic_position() / crate::stellar_forge::core::Units::LIGHT_YEAR;

            for star in &system.stars {
                let spectral = if let Some(physical) = &star.physical {
                    if let crate::stellar_forge::physical::Physical::Star(s) = physical {
                        s.spectral_type.clone()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                let habitable = system.habitable_zone().is_some();

                writeln!(
                    file,
                    "{},{},{},{:.2},{:.2},{:.2},{},{}",
                    system.name,
                    star.name,
                    spectral,
                    pos.x,
                    pos.y,
                    pos.z,
                    system.planets.len(),
                    habitable
                )?;
            }
        }

        Ok(())
    }

    // Import from Astrosynthesis SQL (stub)
    pub fn import_from_astrosynthesis(
        _db_path: impl AsRef<Path>,
    ) -> Result<Galaxy, Box<dyn std::error::Error>> {
        // This would use the existing Astrosynthesis import code
        // and convert to StellarForge structures
        todo!("Implement Astrosynthesis import conversion")
    }
}

// Migration from old formats
pub struct Migration;

impl Migration {
    // Convert from Astrosynthesis bodies to StellarForge
    pub fn convert_astro_body(
        _astro_body: serde_json::Value,  // Astrosynthesis body JSON
    ) -> Result<StellarBody, Box<dyn std::error::Error>> {
        // Conversion logic would go here
        todo!("Implement body conversion")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stellar_forge::builders::{SystemBuilder, GalaxyBuilder};

    #[test]
    fn test_dataset_creation() {
        let galaxy = GalaxyBuilder::new("Test Galaxy")
            .with_system(
                SystemBuilder::new("Test System")
                    .at_position(0.0, 0.0, 0.0)
                    .with_star("G2V")
            )
            .build();

        let dataset = StellarForgeDataset::new(galaxy);

        assert_eq!(dataset.version, "1.0.0");
        assert_eq!(dataset.metadata.statistics.total_systems, 1);
        assert_eq!(dataset.metadata.statistics.total_stars, 1);
    }

    #[test]
    fn test_json_serialization() {
        let galaxy = Galaxy::new("Test");
        let dataset = StellarForgeDataset::new(galaxy);

        let json = serde_json::to_string(&dataset).unwrap();
        let parsed: StellarForgeDataset = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.galaxy.name, "Test");
    }
}