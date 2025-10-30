use rusqlite::{Connection, Result as SqliteResult};
use serde::Serialize;

/// Represents a star system extracted from Astrosynthesis
#[derive(Debug, Clone, Serialize)]
pub struct Star {
    pub id: i32,
    pub name: String,
    pub star_type: String,  // body_type column
    pub size: f64,          // radius column
    pub color: String,      // spectral type (color is derived from this)
    pub x: f64,
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
    pub fn read_all_stars(&self) -> SqliteResult<Vec<Star>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, body_type, radius, spectral, x, y, z
             FROM bodies
             WHERE system_id = id AND parent_id = 0
             ORDER BY name"
        )?;

        let stars = stmt.query_map([], |row| {
            Ok(Star {
                id: row.get(0)?,
                name: row.get(1)?,
                star_type: row.get::<_, String>(2).unwrap_or_default(),
                size: row.get(3)?,
                color: row.get::<_, String>(4).unwrap_or_default(),
                x: row.get(5)?,
                y: row.get(6)?,
                z: row.get(7)?,
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
