use std::fs::File;
use std::io::Write;
use anyhow::Result;

use super::Star;

/// Export stars to a CSV file with comprehensive stellar data
/// Columns: Name, Spectral Type, Radius (Solar), Mass (Solar), Luminosity (Solar), Temperature (K), X, Y, Z
pub fn export_stars_to_csv(stars: &[Star], output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Write header row
    writeln!(file, "Name,Spectral Type,Radius (Solar),Mass (Solar),Luminosity (Solar),Temperature (K),X,Y,Z")?;

    // Write data rows
    for star in stars {
        writeln!(
            file,
            "\"{}\",\"{}\",{},{},{},{},{},{},{}",
            // Escape quotes in names by doubling them (CSV standard)
            star.name.replace("\"", "\"\""),
            star.spectral_type,
            star.radius_solar,
            star.mass_solar,
            star.luminosity_solar,
            star.temperature_k,
            star.x,
            star.y,
            star.z
        )?;
    }

    Ok(())
}
