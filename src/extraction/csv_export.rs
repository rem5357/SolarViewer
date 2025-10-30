use std::fs::File;
use std::io::Write;
use anyhow::Result;

use super::Star;

/// Export stars to a CSV file with headers: name, star_type, size, color, x, y, z
pub fn export_stars_to_csv(stars: &[Star], output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Write header row
    writeln!(file, "Name,Star Type,Size,Color,X,Y,Z")?;

    // Write data rows
    for star in stars {
        writeln!(
            file,
            "\"{}\",\"{}\",{},{},{},{},{}",
            // Escape quotes in names by doubling them (CSV standard)
            star.name.replace("\"", "\"\""),
            star.star_type.replace("\"", "\"\""),
            star.size,
            star.color.replace("\"", "\"\""),
            star.x,
            star.y,
            star.z
        )?;
    }

    Ok(())
}
