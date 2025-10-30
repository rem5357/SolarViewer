pub mod renderer;
pub mod projection;

pub use renderer::StarMapRenderer;
pub use projection::ProjectionEngine;

use anyhow::Result;
use crate::extraction::StarReader;
use renderer::StarData;
use projection::Point3D;

/// Render a star map centered on a specific star
pub fn render_star_map(
    db_path: &str,
    center_star_name: &str,
    search_radius_ly: f64,
    output_path: &str,
    width: u32,
    height: u32,
    connection_distance_ly: f64,
) -> Result<()> {
    // Read all stars from database
    let reader = StarReader::new(db_path)?;
    let all_stars = reader.read_all_stars()?;

    // Find the center star
    let center_star = all_stars
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(center_star_name))
        .ok_or_else(|| anyhow::anyhow!("Star '{}' not found in database", center_star_name))?;

    // Find all stars within search radius
    let mut nearby_stars = Vec::new();
    for star in &all_stars {
        let dx = star.x - center_star.x;
        let dy = star.y - center_star.y;
        let dz = star.z - center_star.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist <= search_radius_ly {
            nearby_stars.push(star.clone());
        }
    }

    println!("Found {} stars within {} ly of {}", nearby_stars.len(), search_radius_ly, center_star_name);

    // Convert to StarData for rendering
    let render_stars: Vec<StarData> = nearby_stars
        .iter()
        .map(|s| StarData {
            name: s.name.clone(),
            x: s.x,
            y: s.y,
            z: s.z,
            spectral_type: s.spectral_type.clone(),
            luminosity: s.luminosity_solar,
        })
        .collect();

    // Project to 2D
    let points_3d: Vec<Point3D> = render_stars
        .iter()
        .map(|s| Point3D { x: s.x, y: s.y, z: s.z })
        .collect();

    let projection_engine = ProjectionEngine::new(width, height, 300);
    let mut points_2d = projection_engine.project_orthographic(&points_3d);

    // Resolve overlaps
    projection_engine.resolve_overlaps(&mut points_2d, 150.0);

    // Find connections
    let connections = StarMapRenderer::find_connections(&render_stars, connection_distance_ly);

    // Render to PNG
    let renderer = StarMapRenderer::new(width, height);
    let center_star_idx = render_stars
        .iter()
        .position(|s| s.name.eq_ignore_ascii_case(center_star_name));

    renderer.render_to_file(&render_stars, &points_2d, &connections, center_star_idx, output_path)?;

    println!("Map rendered to: {}", output_path);
    println!("  Stars plotted: {}", render_stars.len());
    println!("  Connections (<{} ly): {}", connection_distance_ly, connections.len());

    Ok(())
}
