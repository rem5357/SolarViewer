//! Coordinate system conversion between Astrosynthesis and StellarForge
//!
//! Astrosynthesis uses a custom coordinate system where:
//! - X: Right (positive east)
//! - Y: Up (positive north)
//! - Z: Out of screen (toward viewer)
//!
//! StellarForge uses IAU Galactic Coordinates where:
//! - X: Toward Galactic Center
//! - Y: Galactic rotation direction
//! - Z: Galactic north pole

use nalgebra::Vector3;

/// Convert Astrosynthesis coordinates to IAU Galactic coordinates
pub struct CoordinateConverter {
    scale_factor: f64,
}

impl CoordinateConverter {
    /// Create a new converter with default scale (1.0)
    pub fn new() -> Self {
        Self { scale_factor: 1.0 }
    }

    /// Create a converter with custom scale factor
    pub fn with_scale(scale_factor: f64) -> Self {
        Self { scale_factor }
    }

    /// Convert Astrosynthesis (x, y, z) to IAU Galactic coordinates
    ///
    /// Transformation:
    /// - Astro X → Galactic Y (right becomes rotation direction)
    /// - Astro Y → Galactic Z (up becomes galactic north)
    /// - Astro Z → Galactic X (out becomes toward center)
    pub fn convert(&self, astro_x: f64, astro_y: f64, astro_z: f64) -> (f64, f64, f64) {
        let galactic_x = astro_z * self.scale_factor;
        let galactic_y = astro_x * self.scale_factor;
        let galactic_z = astro_y * self.scale_factor;

        (galactic_x, galactic_y, galactic_z)
    }

    /// Convert using Vector3
    pub fn convert_vector(&self, astro_pos: Vector3<f64>) -> Vector3<f64> {
        let (x, y, z) = self.convert(astro_pos.x, astro_pos.y, astro_pos.z);
        Vector3::new(x, y, z)
    }

    /// Convert back from Galactic to Astrosynthesis coordinates
    pub fn convert_inverse(&self, gal_x: f64, gal_y: f64, gal_z: f64) -> (f64, f64, f64) {
        let astro_x = gal_y / self.scale_factor;
        let astro_y = gal_z / self.scale_factor;
        let astro_z = gal_x / self.scale_factor;

        (astro_x, astro_y, astro_z)
    }
}

impl Default for CoordinateConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_conversion() {
        let converter = CoordinateConverter::new();

        // Test conversion
        let (gx, gy, gz) = converter.convert(1.0, 2.0, 3.0);
        assert_eq!(gx, 3.0); // Z → X
        assert_eq!(gy, 1.0); // X → Y
        assert_eq!(gz, 2.0); // Y → Z

        // Test round-trip
        let (ax, ay, az) = converter.convert_inverse(gx, gy, gz);
        assert!((ax - 1.0).abs() < 1e-10);
        assert!((ay - 2.0).abs() < 1e-10);
        assert!((az - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_factor() {
        let converter = CoordinateConverter::with_scale(2.0);
        let (gx, gy, gz) = converter.convert(1.0, 1.0, 1.0);

        assert_eq!(gx, 2.0);
        assert_eq!(gy, 2.0);
        assert_eq!(gz, 2.0);
    }
}
