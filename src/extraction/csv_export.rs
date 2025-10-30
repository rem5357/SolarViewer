use std::fs::File;
use std::io::Write;
use anyhow::Result;

use super::Star;

/// Export stars to a CSV file with comprehensive stellar data
/// Includes multi-star system information when applicable
pub fn export_stars_to_csv(stars: &[Star], output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Write header row
    writeln!(file, "Name,Spectral Type,Radius (Solar),Mass (Solar),Luminosity (Solar),Temperature (K),Star X,Star Y,Star Z,System Name,System X,System Y,System Z")?;

    // Write data rows
    for star in stars {
        let system_name = match &star.system_name {
            Some(name) => name.as_str(),
            None => "",
        };

        writeln!(
            file,
            "\"{}\",\"{}\",{},{},{},{},{},{},\"{}\",{},{},{}",
            // Escape quotes in names by doubling them (CSV standard)
            star.name.replace("\"", "\"\""),
            star.spectral_type,
            star.radius_solar,
            star.mass_solar,
            star.luminosity_solar,
            star.temperature_k,
            star.x,
            star.y,
            star.z,
            system_name.replace("\"", "\"\""),
            star.system_x,
            star.system_y
        )?;
    }

    Ok(())
}
