use rusqlite::{Connection, Result as SqliteResult};

/// Analyze multi-star systems in the database
pub fn analyze_multistar_systems(db_path: &str) -> SqliteResult<()> {
    let conn = Connection::open(db_path)?;

    println!("\n=== MULTI-STAR SYSTEM ANALYSIS ===\n");

    // Find containers (system_id = id, no spectral type, but has child components)
    let mut stmt = conn.prepare(
        "SELECT id, name, x, y, z,
                (SELECT COUNT(*) FROM bodies b2
                 WHERE b2.system_id = bodies.id AND b2.parent_id = 0 AND b2.id != bodies.id) as component_count
         FROM bodies
         WHERE system_id = id AND parent_id = 0 AND (spectral = '' OR spectral IS NULL)
         AND (SELECT COUNT(*) FROM bodies b2
              WHERE b2.system_id = bodies.id AND b2.parent_id = 0 AND b2.id != bodies.id) > 0
         ORDER BY name"
    )?;

    let multistar_systems = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,           // id
            row.get::<_, String>(1)?,        // name
            row.get::<_, f64>(2)?,           // x
            row.get::<_, f64>(3)?,           // y
            row.get::<_, f64>(4)?,           // z
            row.get::<_, i32>(5)?,           // component_count
        ))
    })?;

    let mut count = 0;
    for system in multistar_systems {
        let (id, name, x, y, z, comp_count) = system?;
        println!("Multi-Star Container: {} (ID: {})", name, id);
        println!("  Position: ({}, {}, {})", x, y, z);
        println!("  Component Stars: {}", comp_count);

        // Find the component stars
        let mut comp_stmt = conn.prepare(
            "SELECT id, name, spectral, radius, mass, luminosity, temp
             FROM bodies
             WHERE system_id = ? AND parent_id = 0 AND id != ?
             ORDER BY name"
        )?;

        let components = comp_stmt.query_map([id, id], |row| {
            Ok((
                row.get::<_, String>(1)?,                               // name
                row.get::<_, String>(2).unwrap_or_default(),           // spectral
                row.get::<_, f64>(3)?,                                  // radius
                row.get::<_, f64>(4)?,                                  // mass
                row.get::<_, f64>(5)?,                                  // luminosity
                row.get::<_, f64>(6)?,                                  // temp
            ))
        })?;

        for comp in components {
            let (comp_name, spectral, radius, mass, lum, temp) = comp?;
            println!("    ├─ {}: {} (R:{}, M:{}, L:{})",
                     comp_name, spectral, radius, mass, lum);
        }
        println!();

        count += 1;
        if count >= 10 {
            println!("... (showing first 10 multi-star systems)");
            break;
        }
    }

    // Summary statistics
    let mut total_stmt = conn.prepare(
        "SELECT COUNT(*) FROM bodies
         WHERE system_id = id AND parent_id = 0 AND (spectral = '' OR spectral IS NULL)
         AND (SELECT COUNT(*) FROM bodies b2
              WHERE b2.system_id = bodies.id AND b2.parent_id = 0 AND b2.id != bodies.id) > 0"
    )?;
    let total_multistar: i64 = total_stmt.query_row([], |row| row.get(0))?;

    let mut single_stmt = conn.prepare(
        "SELECT COUNT(*) FROM bodies
         WHERE system_id = id AND parent_id = 0 AND spectral != '' AND spectral IS NOT NULL"
    )?;
    let total_singlestar: i64 = single_stmt.query_row([], |row| row.get(0))?;

    let mut empty_stmt = conn.prepare(
        "SELECT COUNT(*) FROM bodies
         WHERE system_id = id AND parent_id = 0 AND (spectral = '' OR spectral IS NULL)
         AND (SELECT COUNT(*) FROM bodies b2
              WHERE b2.system_id = bodies.id AND b2.parent_id = 0 AND b2.id != bodies.id) = 0"
    )?;
    let total_empty: i64 = empty_stmt.query_row([], |row| row.get(0))?;

    println!("\n=== SUMMARY ===");
    println!("Single-Star Systems: {}", total_singlestar);
    println!("Multi-Star Containers: {}", total_multistar);
    println!("Empty/Unclassified: {}", total_empty);
    println!("Total: {}", total_singlestar + total_multistar + total_empty);

    Ok(())
}
