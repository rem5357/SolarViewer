pub mod renderer;
pub mod projection;
pub mod spectral;
pub mod enhanced_renderer;

pub use renderer::StarMapRenderer;
pub use projection::ProjectionEngine;
pub use spectral::SpectralType;
pub use enhanced_renderer::EnhancedStarMapRenderer;

use anyhow::Result;
use crate::extraction::StarReader;
use projection::Point3D;
use spectral::get_spectral_colors;

/// Render a star map centered on a specific star
pub fn render_star_map(
    db_path: &str,
    center_star_name: &str,
    mut search_radius_ly: f64,
    output_path: &str,
    width: u32,
    height: u32,
    mut connection_distance_ly: f64,
) -> Result<()> {
    // Use sensible defaults for enhanced visualization
    if search_radius_ly <= 0.0 {
        search_radius_ly = 25.0;
    }
    if connection_distance_ly <= 0.0 {
        connection_distance_ly = 7.0;
    }

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

    // Convert to StarData for rendering with spectral types
    let render_stars: Vec<enhanced_renderer::StarDataEnhanced> = nearby_stars
        .iter()
        .map(|s| {
            let spectral = s.spectral_type.parse::<SpectralType>().unwrap_or(SpectralType::Unknown);
            enhanced_renderer::StarDataEnhanced {
                name: s.name.clone(),
                x: s.x,
                y: s.y,
                z: s.z,
                spectral_type: spectral,
                luminosity: s.luminosity_solar,
            }
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

    // Find connections using enhanced renderer
    let connections = EnhancedStarMapRenderer::find_connections(&render_stars, connection_distance_ly);

    // Render to PNG using enhanced renderer
    let renderer = EnhancedStarMapRenderer::new(width, height);
    let center_star_idx = render_stars
        .iter()
        .position(|s| s.name.eq_ignore_ascii_case(center_star_name));

    renderer.render_to_file(&render_stars, &points_2d, &connections, center_star_idx, output_path)?;

    println!("Map rendered to: {}", output_path);
    println!("  Stars plotted: {}", render_stars.len());
    println!("  Connections (<{} ly): {}", connection_distance_ly, connections.len());

    Ok(())
}
