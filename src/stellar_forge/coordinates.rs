// Proper astronomical coordinate systems for StellarForge
// Following IAU standards and conventions

use crate::stellar_forge::core::{Vec3, Units};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Standard astronomical coordinate systems
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum AstronomicalCoordinateSystem {
    /// International Celestial Reference System (ICRS)
    /// The current standard, essentially J2000.0 equatorial
    ICRS,

    /// Galactic coordinates (IAU 1958 system)
    /// Origin at Sun, X toward galactic center, Y in rotation direction, Z toward NGP
    Galactic,

    /// Equatorial coordinates (J2000.0)
    /// Right Ascension and Declination
    EquatorialJ2000,

    /// Ecliptic coordinates
    /// Based on Earth's orbital plane
    Ecliptic,

    /// Supergalactic coordinates
    /// For extragalactic work
    Supergalactic,

    /// Local Standard of Rest (LSR)
    /// Corrected for Sun's peculiar motion
    LSR,

    /// Heliocentric coordinates
    /// Sun-centered
    Heliocentric,

    /// Barycentric coordinates (Solar System Barycenter)
    SolarSystemBarycentric,
}

/// Galactic coordinates (standard IAU system)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GalacticCoordinates {
    /// Galactic longitude in radians (0 to 2π)
    /// l=0° toward galactic center, l=90° in rotation direction
    pub longitude_rad: f64,

    /// Galactic latitude in radians (-π/2 to π/2)
    /// b=0° in galactic plane, b=90° toward NGP
    pub latitude_rad: f64,

    /// Distance from Sun in meters (or parsecs for display)
    pub distance_m: f64,
}

impl GalacticCoordinates {
    pub fn new(l_deg: f64, b_deg: f64, distance_pc: f64) -> Self {
        Self {
            longitude_rad: l_deg.to_radians(),
            latitude_rad: b_deg.to_radians(),
            distance_m: distance_pc * Units::PARSEC,
        }
    }

    pub fn to_cartesian(&self) -> Vec3 {
        // Standard transformation from galactic spherical to cartesian
        // X points toward galactic center (l=0°, b=0°)
        // Y points in direction of rotation (l=90°, b=0°)
        // Z points toward North Galactic Pole (b=90°)
        let cos_b = self.latitude_rad.cos();
        let sin_b = self.latitude_rad.sin();
        let cos_l = self.longitude_rad.cos();
        let sin_l = self.longitude_rad.sin();

        Vec3::new(
            self.distance_m * cos_b * cos_l,  // X toward GC
            self.distance_m * cos_b * sin_l,  // Y toward l=90°
            self.distance_m * sin_b,          // Z toward NGP
        )
    }

    pub fn from_cartesian(cart: Vec3) -> Self {
        let distance_m = cart.norm();
        let longitude_rad = cart.y.atan2(cart.x);
        let latitude_rad = if distance_m > 0.0 {
            (cart.z / distance_m).asin()
        } else {
            0.0
        };

        Self {
            longitude_rad,
            latitude_rad,
            distance_m,
        }
    }

    pub fn longitude_deg(&self) -> f64 {
        self.longitude_rad.to_degrees()
    }

    pub fn latitude_deg(&self) -> f64 {
        self.latitude_rad.to_degrees()
    }

    pub fn distance_pc(&self) -> f64 {
        self.distance_m / Units::PARSEC
    }

    pub fn distance_ly(&self) -> f64 {
        self.distance_m / Units::LIGHT_YEAR
    }
}

/// Equatorial coordinates (J2000.0)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EquatorialCoordinates {
    /// Right Ascension in radians (0 to 2π)
    pub right_ascension_rad: f64,

    /// Declination in radians (-π/2 to π/2)
    pub declination_rad: f64,

    /// Distance from Earth/Sun in meters
    pub distance_m: f64,
}

impl EquatorialCoordinates {
    pub fn new(ra_hours: f64, dec_deg: f64, distance_pc: f64) -> Self {
        Self {
            right_ascension_rad: ra_hours * PI / 12.0,  // hours to radians
            declination_rad: dec_deg.to_radians(),
            distance_m: distance_pc * Units::PARSEC,
        }
    }

    pub fn to_cartesian(&self) -> Vec3 {
        let cos_dec = self.declination_rad.cos();
        let sin_dec = self.declination_rad.sin();
        let cos_ra = self.right_ascension_rad.cos();
        let sin_ra = self.right_ascension_rad.sin();

        Vec3::new(
            self.distance_m * cos_dec * cos_ra,
            self.distance_m * cos_dec * sin_ra,
            self.distance_m * sin_dec,
        )
    }

    pub fn ra_hours(&self) -> f64 {
        self.right_ascension_rad * 12.0 / PI
    }

    pub fn dec_degrees(&self) -> f64 {
        self.declination_rad.to_degrees()
    }
}

/// Coordinate transformation matrices and methods
pub struct CoordinateTransform;

impl CoordinateTransform {
    /// Transform from Equatorial J2000 to Galactic coordinates
    /// Using IAU standard transformation matrix
    pub fn equatorial_to_galactic(eq: EquatorialCoordinates) -> GalacticCoordinates {
        // Standard transformation matrix elements (Reid & Brunthaler 2004)
        // These define the orientation of the galactic coordinate system
        const T11: f64 = -0.054875539390;
        const T12: f64 = -0.873437104725;
        const T13: f64 = -0.483834991775;
        const T21: f64 = +0.494109453633;
        const T22: f64 = -0.444829594298;
        const T23: f64 = +0.746982248696;
        const T31: f64 = -0.867666135681;
        const T32: f64 = -0.198076389622;
        const T33: f64 = +0.455983794523;

        let eq_cart = eq.to_cartesian();

        // Apply rotation matrix
        let gal_x = T11 * eq_cart.x + T12 * eq_cart.y + T13 * eq_cart.z;
        let gal_y = T21 * eq_cart.x + T22 * eq_cart.y + T23 * eq_cart.z;
        let gal_z = T31 * eq_cart.x + T32 * eq_cart.y + T33 * eq_cart.z;

        GalacticCoordinates::from_cartesian(Vec3::new(gal_x, gal_y, gal_z))
    }

    /// Transform from Galactic to Equatorial J2000 coordinates
    pub fn galactic_to_equatorial(gal: GalacticCoordinates) -> EquatorialCoordinates {
        // Inverse transformation matrix (transpose of the above)
        const T11: f64 = -0.054875539390;
        const T21: f64 = -0.873437104725;
        const T31: f64 = -0.483834991775;
        const T12: f64 = +0.494109453633;
        const T22: f64 = -0.444829594298;
        const T32: f64 = +0.746982248696;
        const T13: f64 = -0.867666135681;
        const T23: f64 = -0.198076389622;
        const T33: f64 = +0.455983794523;

        let gal_cart = gal.to_cartesian();

        // Apply inverse rotation matrix
        let eq_x = T11 * gal_cart.x + T12 * gal_cart.y + T13 * gal_cart.z;
        let eq_y = T21 * gal_cart.x + T22 * gal_cart.y + T23 * gal_cart.z;
        let eq_z = T31 * gal_cart.x + T32 * gal_cart.y + T33 * gal_cart.z;

        let cart = Vec3::new(eq_x, eq_y, eq_z);
        let distance_m = cart.norm();
        let ra_rad = cart.y.atan2(cart.x);
        let dec_rad = if distance_m > 0.0 {
            (cart.z / distance_m).asin()
        } else {
            0.0
        };

        EquatorialCoordinates {
            right_ascension_rad: if ra_rad < 0.0 { ra_rad + 2.0 * PI } else { ra_rad },
            declination_rad: dec_rad,
            distance_m,
        }
    }

    /// Convert from Astrosynthesis coordinates to standard Galactic
    /// Astrosynthesis uses a different orientation that needs correction
    pub fn astrosynthesis_to_galactic(astro_x: f64, astro_y: f64, astro_z: f64) -> GalacticCoordinates {
        // Astrosynthesis appears to use:
        // - X pointing "right" on the map
        // - Y pointing "up" on the map
        // - Z pointing "out" of the screen
        // This needs to be rotated to match IAU galactic coordinates

        // Approximate transformation (may need refinement based on actual data)
        // This assumes Astrosynthesis has galactic center roughly along +X
        let gal_x = astro_x * Units::LIGHT_YEAR;  // Assuming input in light-years
        let gal_y = -astro_z * Units::LIGHT_YEAR; // Swap and negate
        let gal_z = astro_y * Units::LIGHT_YEAR;  // Move Y to Z

        GalacticCoordinates::from_cartesian(Vec3::new(gal_x, gal_y, gal_z))
    }

    /// Convert from standard Galactic to Astrosynthesis coordinates
    pub fn galactic_to_astrosynthesis(gal: GalacticCoordinates) -> (f64, f64, f64) {
        let cart = gal.to_cartesian();

        // Inverse of the above transformation
        let astro_x = cart.x / Units::LIGHT_YEAR;
        let astro_y = cart.z / Units::LIGHT_YEAR;
        let astro_z = -cart.y / Units::LIGHT_YEAR;

        (astro_x, astro_y, astro_z)
    }

    /// Apply Local Standard of Rest (LSR) correction
    /// Corrects for the Sun's peculiar motion relative to nearby stars
    pub fn apply_lsr_correction(vel: Vec3) -> Vec3 {
        // Sun's motion relative to LSR (Schönrich et al. 2010)
        // U = 11.1 km/s (toward galactic center)
        // V = 12.24 km/s (in direction of rotation)
        // W = 7.25 km/s (toward NGP)
        const U_SOLAR: f64 = 11100.0;  // m/s
        const V_SOLAR: f64 = 12240.0;  // m/s
        const W_SOLAR: f64 = 7250.0;   // m/s

        Vec3::new(
            vel.x - U_SOLAR,
            vel.y - V_SOLAR,
            vel.z - W_SOLAR,
        )
    }

    /// Get Sun's position in galactic coordinates
    /// By definition, Sun is at origin of galactic coordinate system
    pub fn sun_position_galactic() -> Vec3 {
        Vec3::zeros()
    }

    /// Get galactic center position from Sun
    pub fn galactic_center_position() -> GalacticCoordinates {
        // Galactic center is at l=0°, b=0°, distance ~8 kpc
        GalacticCoordinates::new(0.0, 0.0, 8000.0)
    }

    /// Get North Galactic Pole in equatorial coordinates
    pub fn north_galactic_pole_equatorial() -> EquatorialCoordinates {
        // NGP at RA = 12h 51m 26.28s, Dec = +27° 07' 41.7" (J2000)
        EquatorialCoordinates::new(
            12.0 + 51.0/60.0 + 26.28/3600.0,  // hours
            27.0 + 7.0/60.0 + 41.7/3600.0,    // degrees
            f64::INFINITY,  // At infinity
        )
    }
}

/// Position of important reference points
pub struct ReferencePositions;

impl ReferencePositions {
    /// Sol (our Sun) - origin of galactic coordinates
    pub fn sol() -> GalacticCoordinates {
        GalacticCoordinates::new(0.0, 0.0, 0.0)
    }

    /// Alpha Centauri system
    pub fn alpha_centauri() -> GalacticCoordinates {
        // l = 315.8°, b = -0.68°, d = 1.34 pc
        GalacticCoordinates::new(315.8, -0.68, 1.34)
    }

    /// Sagittarius A* (galactic center black hole)
    pub fn sgr_a_star() -> GalacticCoordinates {
        // At galactic center: l = 0°, b = 0°, d = 8178 pc
        GalacticCoordinates::new(0.0, 0.0, 8178.0)
    }

    /// Polaris (North Star)
    pub fn polaris() -> GalacticCoordinates {
        // l = 123.3°, b = -17.1°, d = 132.6 pc
        GalacticCoordinates::new(123.3, -17.1, 132.6)
    }

    /// Vega
    pub fn vega() -> GalacticCoordinates {
        // l = 67.4°, b = 19.2°, d = 7.68 pc
        GalacticCoordinates::new(67.4, 19.2, 7.68)
    }

    /// Betelgeuse
    pub fn betelgeuse() -> GalacticCoordinates {
        // l = 199.8°, b = -8.96°, d = 168.1 pc
        GalacticCoordinates::new(199.8, -8.96, 168.1)
    }
}

/// Helper to format coordinates for display
pub struct CoordinateFormatter;

impl CoordinateFormatter {
    /// Format galactic coordinates as "l=123.45°, b=-67.89°, d=1234.5 pc"
    pub fn format_galactic(gal: &GalacticCoordinates) -> String {
        format!(
            "l={:.2}°, b={:+.2}°, d={:.1} pc",
            gal.longitude_deg(),
            gal.latitude_deg(),
            gal.distance_pc()
        )
    }

    /// Format equatorial coordinates as "RA 12h 34m 56.78s, Dec +12° 34' 56.7\", d=123.4 pc"
    pub fn format_equatorial(eq: &EquatorialCoordinates) -> String {
        let ra_hours = eq.ra_hours();
        let ra_h = ra_hours.floor() as i32;
        let ra_m = ((ra_hours - ra_h as f64) * 60.0).floor() as i32;
        let ra_s = ((ra_hours - ra_h as f64) * 60.0 - ra_m as f64) * 60.0;

        let dec_deg = eq.dec_degrees();
        let dec_sign = if dec_deg >= 0.0 { "+" } else { "" };
        let dec_d = dec_deg.abs().floor() as i32;
        let dec_m = ((dec_deg.abs() - dec_d as f64) * 60.0).floor() as i32;
        let dec_s = ((dec_deg.abs() - dec_d as f64) * 60.0 - dec_m as f64) * 60.0;

        format!(
            "RA {:02}h {:02}m {:.2}s, Dec {}{:02}° {:02}' {:.1}\", d={:.1} pc",
            ra_h, ra_m, ra_s,
            dec_sign, dec_d, dec_m, dec_s,
            eq.distance_m / Units::PARSEC
        )
    }

    /// Format cartesian position as "X=1234.5, Y=-567.8, Z=90.1 ly"
    pub fn format_cartesian(pos: Vec3, unit: &str) -> String {
        let divisor = match unit {
            "pc" => Units::PARSEC,
            "ly" => Units::LIGHT_YEAR,
            "AU" => Units::AU,
            "km" => Units::KILOMETER,
            _ => Units::METER,
        };

        format!(
            "X={:.1}, Y={:.1}, Z={:.1} {}",
            pos.x / divisor,
            pos.y / divisor,
            pos.z / divisor,
            unit
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galactic_coordinates() {
        // Test galactic center
        let gc = GalacticCoordinates::new(0.0, 0.0, 8000.0);
        let cart = gc.to_cartesian();
        assert!((cart.x - 8000.0 * Units::PARSEC).abs() < 1.0);
        assert!(cart.y.abs() < 1.0);
        assert!(cart.z.abs() < 1.0);

        // Test round trip
        let gc2 = GalacticCoordinates::from_cartesian(cart);
        assert!((gc2.longitude_deg() - 0.0).abs() < 1e-10);
        assert!((gc2.latitude_deg() - 0.0).abs() < 1e-10);
        assert!((gc2.distance_pc() - 8000.0).abs() < 1e-6);
    }

    #[test]
    fn test_coordinate_transform() {
        // Test that NGP transforms correctly
        let ngp_eq = CoordinateTransform::north_galactic_pole_equatorial();
        let ngp_gal = CoordinateTransform::equatorial_to_galactic(ngp_eq);

        // NGP should be at b = 90°
        assert!((ngp_gal.latitude_deg() - 90.0).abs() < 1.0);
    }

    #[test]
    fn test_reference_positions() {
        let sol = ReferencePositions::sol();
        assert_eq!(sol.distance_pc(), 0.0);

        let alpha_cen = ReferencePositions::alpha_centauri();
        assert!((alpha_cen.distance_pc() - 1.34).abs() < 0.01);
    }
}