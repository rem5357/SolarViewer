/// Star map renderer - creates PNG images of star fields
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};
use std::path::Path;
use anyhow::Result;

use super::projection::Point2D;

#[derive(Debug, Clone)]
pub struct StarData {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub spectral_type: String,
    pub luminosity: f64,
}

#[derive(Debug, Clone)]
pub struct StarConnection {
    pub from_idx: usize,
    pub to_idx: usize,
    pub distance_ly: f64,
}

pub struct StarMapRenderer {
    width: u32,
    height: u32,
    background: Rgb<u8>,
    star_color: Rgb<u8>,
    line_color: Rgb<u8>,
    text_color: Rgb<u8>,
    base_star_radius: u32,
}

impl Default for StarMapRenderer {
    fn default() -> Self {
        Self {
            width: 5000,
            height: 5000,
            background: Rgb([0, 0, 0]),       // Black
            star_color: Rgb([255, 255, 255]), // White
            line_color: Rgb([100, 150, 255]), // Light blue
            text_color: Rgb([200, 220, 255]), // Light blue for text
            base_star_radius: 40,
        }
    }
}

impl StarMapRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn render_to_file<P: AsRef<Path>>(
        &self,
        stars: &[StarData],
        star_positions_2d: &[Point2D],
        connections: &[StarConnection],
        center_star_idx: Option<usize>,
        output_path: P,
    ) -> Result<()> {
        // Create image buffer with background color
        let mut img: RgbImage = ImageBuffer::from_pixel(self.width, self.height, self.background);

        // Draw connections (lines between nearby stars)
        for conn in connections {
            if conn.from_idx < star_positions_2d.len() && conn.to_idx < star_positions_2d.len() {
                let p1 = &star_positions_2d[conn.from_idx];
                let p2 = &star_positions_2d[conn.to_idx];

                // Vary line width based on distance
                let line_color = if conn.distance_ly < 5.0 {
                    Rgb([150, 200, 255]) // Brighter for close stars
                } else {
                    self.line_color
                };

                // Draw line using Bresenham's algorithm
                draw_line_segment_mut(
                    &mut img,
                    (p1.x as f32, p1.y as f32),
                    (p2.x as f32, p2.y as f32),
                    line_color,
                );
            }
        }

        // Draw stars
        for (idx, star) in stars.iter().enumerate() {
            if idx >= star_positions_2d.len() {
                break;
            }

            let pos = &star_positions_2d[idx];
            let px = pos.x as i32;
            let py = pos.y as i32;

            // Calculate star size based on luminosity
            let size_factor = if star.luminosity > 0.0 {
                (star.luminosity.cbrt()).min(3.0) // Cube root, max 3x
            } else {
                1.0
            };
            let radius = ((self.base_star_radius as f64) * size_factor) as i32;

            // Draw star circle
            let star_color = if Some(idx) == center_star_idx {
                Rgb([255, 200, 0]) // Gold for center star
            } else {
                self.star_color
            };

            draw_filled_circle_mut(&mut img, (px, py), radius, star_color);
        }

        // Draw title
        let title = format!("Star Field - {} stars", stars.len());
        // Note: Text rendering requires font loading which may fail, so we skip it for now
        // In a full implementation, we'd load a system font here

        // Save image
        img.save(&output_path)?;
        Ok(())
    }

    /// Helper method to calculate which connections to draw
    pub fn find_connections(
        stars: &[StarData],
        max_distance_ly: f64,
    ) -> Vec<StarConnection> {
        let mut connections = Vec::new();

        for i in 0..stars.len() {
            for j in (i + 1)..stars.len() {
                let dx = stars[j].x - stars[i].x;
                let dy = stars[j].y - stars[i].y;
                let dz = stars[j].z - stars[i].z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                if dist <= max_distance_ly {
                    connections.push(StarConnection {
                        from_idx: i,
                        to_idx: j,
                        distance_ly: dist,
                    });
                }
            }
        }

        // Sort by distance
        connections.sort_by(|a, b| a.distance_ly.partial_cmp(&b.distance_ly).unwrap());

        connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = StarMapRenderer::new(5000, 5000);
        assert_eq!(renderer.width, 5000);
        assert_eq!(renderer.height, 5000);
    }

    #[test]
    fn test_find_connections() {
        let stars = vec![
            StarData {
                name: "A".to_string(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
                spectral_type: "G3V".to_string(),
                luminosity: 1.0,
            },
            StarData {
                name: "B".to_string(),
                x: 5.0,
                y: 0.0,
                z: 0.0,
                spectral_type: "M4V".to_string(),
                luminosity: 0.1,
            },
            StarData {
                name: "C".to_string(),
                x: 100.0,
                y: 0.0,
                z: 0.0,
                spectral_type: "B5V".to_string(),
                luminosity: 10.0,
            },
        ];

        let connections = StarMapRenderer::find_connections(&stars, 10.0);
        assert_eq!(connections.len(), 1); // Only A-B should be within 10 ly
        assert_eq!(connections[0].from_idx, 0);
        assert_eq!(connections[0].to_idx, 1);
    }
}
