use rusqlite::{Connection, Result as SqliteResult};
use serde::Serialize;

/// Represents a star system extracted from Astrosynthesis
#[derive(Debug, Clone, Serialize)]
pub struct Star {
    pub id: i32,
    pub name: String,
    pub spectral_type: String,  // Official astronomical classification (e.g., "G3V", "M4V")
    pub radius_solar: f64,       // Star radius in solar radii
    pub mass_solar: f64,         // Star mass in solar masses
    pub luminosity_solar: f64,   // Star luminosity in solar luminosities
    pub temperature_k: f64,      // Surface temperature in Kelvin
    pub x: f64,                  // 3D coordinate in light-years
    pub y: f64,
    pub z: f64,
    pub system_name: Option<String>,  // Name of multi-star system container (if this is a component)
    pub system_x: f64,           // System container position (same for all components)
    pub system_y: f64,
    pub system_z: f64,
}

/// Reader for extracting stars from Astrosynthesis .AstroDB files
pub struct StarReader {
    conn: Connection,
}

impl StarReader {
    /// Create a new StarReader from an .AstroDB file path
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        Ok(StarReader { conn })
    }

    /// Extract all stars from the database
    /// Handles both single-star systems and multi-star containers
    /// For multi-star systems, includes the container name and position
    pub fn read_all_stars(&self) -> SqliteResult<Vec<Star>> {
        let mut result = Vec::new();

        // Get single-star systems (system_id = id with spectral type)
        let mut stmt = self.conn.prepare(
            "SELECT id, name, spectral, radius, mass, luminosity, temp, x, y, z
             FROM bodies
             WHERE system_id = id AND parent_id = 0 AND spectral != '' AND spectral IS NOT NULL
             ORDER BY name"
        )?;

        let stars = stmt.query_map([], |row| {
            Ok(Star {
                id: row.get(0)?,
                name: row.get(1)?,
                spectral_type: row.get::<_, String>(2).unwrap_or_default(),
                radius_solar: row.get(3)?,
                mass_solar: row.get(4)?,
                luminosity_solar: row.get(5)?,
                temperature_k: row.get(6)?,
                x: row.get(7)?,
                y: row.get(8)?,
                z: row.get(9)?,
                system_name: None,
                system_x: row.get(7)?,
                system_y: row.get(8)?,
                system_z: row.get(9)?,
            })
        })?;

        for star in stars {
            result.push(star?);
        }

        // Get component stars from multi-star containers
        // A multi-star container has system_id = id, parent_id = 0, no spectral type,
        // and has child stars (parent_id = container_id, spectral type set)
        let mut multi_stmt = self.conn.prepare(
            "SELECT b.id, b.name, b.spectral, b.radius, b.mass, b.luminosity, b.temp,
                    b.x, b.y, b.z, c.name, c.x, c.y, c.z
             FROM bodies b
             JOIN bodies c ON b.parent_id = c.id
             WHERE c.system_id = c.id AND c.parent_id = 0
             AND (c.spectral = '' OR c.spectral IS NULL)
             AND b.spectral != '' AND b.spectral IS NOT NULL
             AND b.parent_id = c.id
             ORDER BY c.name, b.name"
        )?;

        let multi_stars = multi_stmt.query_map([], |row| {
            Ok(Star {
                id: row.get(0)?,
                name: row.get(1)?,
                spectral_type: row.get::<_, String>(2).unwrap_or_default(),
                radius_solar: row.get(3)?,
                mass_solar: row.get(4)?,
                luminosity_solar: row.get(5)?,
                temperature_k: row.get(6)?,
                x: row.get(7)?,
                y: row.get(8)?,
                z: row.get(9)?,
                system_name: Some(row.get::<_, String>(10)?),
                system_x: row.get(11)?,
                system_y: row.get(12)?,
                system_z: row.get(13)?,
            })
        })?;

        for star in multi_stars {
            result.push(star?);
        }

        // Sort by name
        result.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(result)
    }

    /// Count total number of stars (includes both single-star systems and multi-star components)
    pub fn count_stars(&self) -> SqliteResult<i64> {
        // Count single-star systems
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM bodies
             WHERE system_id = id AND parent_id = 0 AND spectral != '' AND spectral IS NOT NULL"
        )?;
        let single_stars: i64 = stmt.query_row([], |row| row.get(0))?;

        // Count component stars in multi-star systems
        let mut multi_stmt = self.conn.prepare(
            "SELECT COUNT(DISTINCT b.id) FROM bodies b
             JOIN bodies c ON b.parent_id = c.id
             WHERE c.system_id = c.id AND c.parent_id = 0
             AND (c.spectral = '' OR c.spectral IS NULL)
             AND b.spectral != '' AND b.spectral IS NOT NULL
             AND b.parent_id = c.id"
        )?;
        let multi_stars: i64 = multi_stmt.query_row([], |row| row.get(0))?;

        Ok(single_stars + multi_stars)
    }
}
