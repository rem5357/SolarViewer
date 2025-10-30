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
    /// A star is identified as a body where system_id = id (it defines its own system)
    /// Returns stars with their spectral type, physical properties, and 3D coordinates
    pub fn read_all_stars(&self) -> SqliteResult<Vec<Star>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, spectral, radius, mass, luminosity, temp, x, y, z
             FROM bodies
             WHERE system_id = id AND parent_id = 0
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
            })
        })?;

        let mut result = Vec::new();
        for star in stars {
            result.push(star?);
        }

        Ok(result)
    }

    /// Count total number of stars
    pub fn count_stars(&self) -> SqliteResult<i64> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM bodies WHERE system_id = id AND parent_id = 0"
        )?;

        stmt.query_row([], |row| row.get(0))
    }
}
