/// Enhanced star map renderer with spectral colors and labels
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};
use std::path::Path;
use anyhow::Result;

use super::projection::Point2D;
use super::spectral::{SpectralType, get_spectral_colors};

#[derive(Debug, Clone)]
pub struct StarDataEnhanced {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub spectral_type: SpectralType,
    pub luminosity: f64,
}

#[derive(Debug, Clone)]
pub struct StarConnection {
    pub from_idx: usize,
    pub to_idx: usize,
    pub distance_ly: f64,
}

pub struct EnhancedStarMapRenderer {
    width: u32,
    height: u32,
    background: Rgb<u8>,
}

impl Default for EnhancedStarMapRenderer {
    fn default() -> Self {
        Self {
            width: 5000,
            height: 5000,
            background: Rgb([0, 0, 0]), // Black
        }
    }
}

impl EnhancedStarMapRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Convert our Color type to image::Rgb
    fn color_to_rgb(color: super::spectral::Color) -> Rgb<u8> {
        Rgb([color.r, color.g, color.b])
    }

    pub fn render_to_file<P: AsRef<Path>>(
        &self,
        stars: &[StarDataEnhanced],
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

                // Vary line color and width based on distance
                let (line_color, width) = if conn.distance_ly < 3.0 {
                    (Rgb([200, 220, 255]), 3) // Very bright for close stars
                } else if conn.distance_ly < 5.0 {
                    (Rgb([150, 200, 255]), 2) // Bright for close-ish stars
                } else if conn.distance_ly < 7.0 {
                    (Rgb([100, 150, 200]), 1) // Medium brightness
                } else {
                    (Rgb([60, 100, 140]), 1) // Dim for farther stars
                };

                // Draw line
                draw_line_segment_mut(
                    &mut img,
                    (p1.x as f32, p1.y as f32),
                    (p2.x as f32, p2.y as f32),
                    line_color,
                );

                // Draw line again with smaller offset for thickness effect
                if width > 1 {
                    draw_line_segment_mut(
                        &mut img,
                        (p1.x as f32 + 0.5, p1.y as f32 + 0.5),
                        (p2.x as f32 + 0.5, p2.y as f32 + 0.5),
                        line_color,
                    );
                }
            }
        }

        // Draw stars with spectral colors
        for (idx, star) in stars.iter().enumerate() {
            if idx >= star_positions_2d.len() {
                break;
            }

            let pos = &star_positions_2d[idx];
            let px = pos.x as i32;
            let py = pos.y as i32;

            // Get spectral color
            let (core_color, _glow_color) = get_spectral_colors(star.spectral_type);
            let star_rgb = Self::color_to_rgb(core_color);

            // Calculate star size based on luminosity
            let size_factor = if star.luminosity > 0.0 {
                (star.luminosity.cbrt()).min(3.0) // Cube root, max 3x
            } else {
                1.0
            };
            let base_radius = 40;
            let radius = ((base_radius as f64) * size_factor) as i32;

            // Determine if this is the center star
            let is_center = Some(idx) == center_star_idx;

            // Draw glow (larger, semi-transparent effect via repeated circles with fading)
            let glow_color = if is_center {
                Rgb([255, 220, 100]) // Golden glow for center
            } else {
                Rgb([
                    (star_rgb[0] as u32 + 80).min(255) as u8,
                    (star_rgb[1] as u32 + 80).min(255) as u8,
                    (star_rgb[2] as u32 + 80).min(255) as u8,
                ])
            };

            // Draw outer glow
            let glow_radius = (radius as f64 * 0.7) as i32;
            if glow_radius > 0 {
                draw_filled_circle_mut(&mut img, (px, py), glow_radius, glow_color);
            }

            // Draw star core
            let final_color = if is_center {
                Rgb([255, 200, 0]) // Gold for center star
            } else {
                star_rgb
            };

            draw_filled_circle_mut(&mut img, (px, py), radius, final_color);

            // Draw highlight
            let highlight_radius = (radius as f64 * 0.25) as i32;
            if highlight_radius > 0 {
                let highlight = Rgb([255, 255, 255]);
                draw_filled_circle_mut(&mut img, (px - 5, py - 5), highlight_radius, highlight);
            }
        }

        // Save image
        img.save(&output_path)?;
        Ok(())
    }

    /// Helper method to calculate which connections to draw
    pub fn find_connections(
        stars: &[StarDataEnhanced],
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
        let renderer = EnhancedStarMapRenderer::new(5000, 5000);
        assert_eq!(renderer.width, 5000);
        assert_eq!(renderer.height, 5000);
    }
}
