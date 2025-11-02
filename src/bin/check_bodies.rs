use rusqlite::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("TestAlpha.AstroDB")?;

    // Get all distinct body_type values
    println!("=== Distinct body_type values ===");
    let mut stmt = conn.prepare("SELECT DISTINCT body_type FROM Bodies ORDER BY body_type")?;
    let types: Vec<String> = stmt.query_map([], |row| {
        let val: Option<String> = row.get(0)?;
        Ok(val.unwrap_or("NULL".to_string()))
    })?.filter_map(|r| r.ok()).collect();

    for t in &types {
        println!("  - '{}'", t);
    }

    // Count bodies by type
    println!("\n=== Count by body_type ===");
    for t in &types {
        let count: i64 = if t == "NULL" {
            conn.query_row("SELECT COUNT(*) FROM Bodies WHERE body_type IS NULL", [], |row| row.get(0))?
        } else {
            conn.query_row("SELECT COUNT(*) FROM Bodies WHERE body_type = ?", [t], |row| row.get(0))?
        };
        println!("  {}: {}", t, count);
    }

    // Check for non-star bodies with parent_id != 0
    println!("\n=== Non-star bodies (parent_id != 0, spectral empty) ===");
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM Bodies WHERE parent_id != 0 AND (spectral IS NULL OR spectral = '')",
        [],
        |row| row.get(0)
    )?;
    println!("  Total: {}", count);

    // Sample 5 non-star bodies
    println!("\n=== Sample non-star bodies ===");
    let mut stmt = conn.prepare(
        "SELECT id, name, body_type, parent_id, spectral FROM Bodies
         WHERE parent_id != 0 AND (spectral IS NULL OR spectral = '')
         LIMIT 10"
    )?;

    let samples = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    })?;

    for sample in samples {
        if let Ok((id, name, body_type, parent_id, spectral)) = sample {
            println!("  ID: {}, Name: '{}', Type: '{:?}', Parent: {}, Spectral: '{:?}'",
                     id, name, body_type, parent_id, spectral);
        }
    }

    Ok(())
}
