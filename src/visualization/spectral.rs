/// Spectral type classification and color mapping for stars
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralType {
    O,  // Blue
    B,  // Blue-white
    A,  // White
    F,  // Yellow-white
    G,  // Yellow (like Sol)
    K,  // Orange
    M,  // Red
    Unknown,
}

impl FromStr for SpectralType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_uppercase();
        if s.is_empty() {
            return Ok(SpectralType::Unknown);
        }

        match s.chars().next().unwrap() {
            'O' => Ok(SpectralType::O),
            'B' => Ok(SpectralType::B),
            'A' => Ok(SpectralType::A),
            'F' => Ok(SpectralType::F),
            'G' => Ok(SpectralType::G),
            'K' => Ok(SpectralType::K),
            'M' => Ok(SpectralType::M),
            _ => Ok(SpectralType::Unknown),
        }
    }
}

/// RGB color with alpha channel
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn with_alpha(&self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    pub fn to_tiny_skia(&self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(self.r, self.g, self.b, self.a)
    }
}

/// Get core and glow colors for a spectral type
pub fn get_spectral_colors(spectral_type: SpectralType) -> (Color, Color) {
    // Returns (core_color, glow_color)
    match spectral_type {
        SpectralType::O => (
            Color::rgb(155, 176, 255), // Bright blue core
            Color::rgba(100, 130, 255, 0), // Blue glow (transparent at edge)
        ),
        SpectralType::B => (
            Color::rgb(170, 191, 255), // Blue-white
            Color::rgba(120, 150, 255, 0),
        ),
        SpectralType::A => (
            Color::rgb(202, 215, 255), // White
            Color::rgba(180, 200, 255, 0),
        ),
        SpectralType::F => (
            Color::rgb(248, 247, 255), // Yellow-white
            Color::rgba(240, 230, 200, 0),
        ),
        SpectralType::G => (
            Color::rgb(255, 244, 234), // Yellow (Sol-like)
            Color::rgba(255, 220, 150, 0),
        ),
        SpectralType::K => (
            Color::rgb(255, 210, 161), // Orange
            Color::rgba(255, 180, 100, 0),
        ),
        SpectralType::M => (
            Color::rgb(255, 204, 111), // Red-orange
            Color::rgba(255, 150, 80, 0),
        ),
        SpectralType::Unknown => (
            Color::rgb(200, 200, 200), // Gray for unknown
            Color::rgba(150, 150, 150, 0),
        ),
    }
}
