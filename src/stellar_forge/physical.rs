// Physical properties and components for stellar bodies

use crate::stellar_forge::core::Units;
use serde::{Deserialize, Serialize};

// Main physical properties enum
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Physical {
    Star(StarPhysical),
    Planet(PlanetPhysical),
    Moon(PlanetPhysical),  // Moons use same properties as planets
    Station(StationPhysical),
    Belt(BeltPhysical),
    Asteroid(AsteroidPhysical),
    Generic(GenericPhysical),
}

impl Physical {
    pub fn new_star(spectral_type: impl Into<String>) -> Self {
        let spectral = spectral_type.into();
        let props = StarPhysical::from_spectral_type(&spectral);
        Physical::Star(props)
    }

    pub fn new_planet() -> Self {
        Physical::Planet(PlanetPhysical::default())
    }

    pub fn new_moon() -> Self {
        Physical::Moon(PlanetPhysical::default())
    }

    pub fn new_station() -> Self {
        Physical::Station(StationPhysical::default())
    }

    pub fn mass_kg(&self) -> Option<f64> {
        match self {
            Physical::Star(s) => Some(s.mass_kg),
            Physical::Planet(p) | Physical::Moon(p) => Some(p.mass_kg),
            Physical::Station(s) => s.dry_mass_kg,
            Physical::Asteroid(a) => Some(a.mass_kg),
            Physical::Generic(g) => g.mass_kg,
            _ => None,
        }
    }

    pub fn radius_m(&self) -> Option<f64> {
        match self {
            Physical::Star(s) => Some(s.radius_m),
            Physical::Planet(p) | Physical::Moon(p) => Some(p.radius_m),
            Physical::Belt(b) => Some((b.inner_radius_m + b.outer_radius_m) / 2.0),
            Physical::Asteroid(a) => Some(a.radius_m),
            Physical::Generic(g) => g.radius_m,
            _ => None,
        }
    }
}

// Star physical properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StarPhysical {
    pub mass_kg: f64,
    pub radius_m: f64,
    pub luminosity_w: f64,
    pub temperature_k: f64,
    pub spectral_type: String,
    pub spectral_class: SpectralClass,
    pub luminosity_class: LuminosityClass,
    pub age_years: Option<f64>,
    pub metallicity: Option<f64>,  // [Fe/H]
    pub magnetic_field_t: Option<f64>,
    pub rotation_velocity_mps: Option<f64>,
}

impl StarPhysical {
    // Create star properties from spectral type string (e.g., "G2V")
    pub fn from_spectral_type(spectral: &str) -> Self {
        let (class, subclass, luminosity) = parse_spectral_type(spectral);

        // Get base properties for the spectral class
        let (mass, radius, temp, lum) = match class {
            SpectralClass::O => {
                let temp = 30000.0 + subclass as f64 * 2000.0;
                let mass = 16.0 * Units::SOLAR_MASS;
                let radius = 6.6 * Units::SOLAR_RADIUS;
                let lum = 30000.0 * 3.828e26;  // Solar luminosity
                (mass, radius, temp, lum)
            }
            SpectralClass::B => {
                let temp = 10000.0 + subclass as f64 * 2000.0;
                let mass = 2.9 * Units::SOLAR_MASS;
                let radius = 1.8 * Units::SOLAR_RADIUS;
                let lum = 25.0 * 3.828e26;
                (mass, radius, temp, lum)
            }
            SpectralClass::A => {
                let temp = 7500.0 + subclass as f64 * 250.0;
                let mass = 1.4 * Units::SOLAR_MASS;
                let radius = 1.4 * Units::SOLAR_RADIUS;
                let lum = 5.0 * 3.828e26;
                (mass, radius, temp, lum)
            }
            SpectralClass::F => {
                let temp = 6000.0 + subclass as f64 * 150.0;
                let mass = 1.04 * Units::SOLAR_MASS;
                let radius = 1.15 * Units::SOLAR_RADIUS;
                let lum = 1.5 * 3.828e26;
                (mass, radius, temp, lum)
            }
            SpectralClass::G => {
                let temp = 5200.0 + subclass as f64 * 80.0;
                let mass = (0.8 + subclass as f64 * 0.02) * Units::SOLAR_MASS;
                let radius = (0.96 + subclass as f64 * 0.004) * Units::SOLAR_RADIUS;
                let lum = (0.6 + subclass as f64 * 0.04) * 3.828e26;
                (mass, radius, temp, lum)
            }
            SpectralClass::K => {
                let temp = 3700.0 + subclass as f64 * 150.0;
                let mass = 0.45 * Units::SOLAR_MASS;
                let radius = 0.7 * Units::SOLAR_RADIUS;
                let lum = 0.08 * 3.828e26;
                (mass, radius, temp, lum)
            }
            SpectralClass::M => {
                let temp = 2400.0 + subclass as f64 * 130.0;
                let mass = 0.08 * Units::SOLAR_MASS;
                let radius = 0.2 * Units::SOLAR_RADIUS;
                let lum = 0.001 * 3.828e26;
                (mass, radius, temp, lum)
            }
            _ => {
                // Default to Sun-like
                (Units::SOLAR_MASS, Units::SOLAR_RADIUS, 5778.0, 3.828e26)
            }
        };

        // Adjust for luminosity class
        let (mass_mult, radius_mult, lum_mult) = match luminosity {
            LuminosityClass::Ia | LuminosityClass::Ib => (25.0, 200.0, 10000.0),
            LuminosityClass::II => (10.0, 50.0, 1000.0),
            LuminosityClass::III => (3.0, 10.0, 100.0),
            LuminosityClass::IV => (1.5, 2.0, 5.0),
            LuminosityClass::V => (1.0, 1.0, 1.0),
            LuminosityClass::VI => (0.8, 0.8, 0.5),
            LuminosityClass::VII => (0.6, 0.01, 0.0001),
        };

        Self {
            mass_kg: mass * mass_mult,
            radius_m: radius * radius_mult,
            luminosity_w: lum * lum_mult,
            temperature_k: temp,
            spectral_type: spectral.to_string(),
            spectral_class: class,
            luminosity_class: luminosity,
            age_years: None,
            metallicity: None,
            magnetic_field_t: None,
            rotation_velocity_mps: None,
        }
    }
}

// Spectral classification
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum SpectralClass {
    O, B, A, F, G, K, M,  // Main sequence
    L, T, Y,  // Brown dwarfs
    C, S,     // Carbon stars
    W,        // Wolf-Rayet
    Unknown,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum LuminosityClass {
    Ia,  // Bright supergiant
    Ib,  // Supergiant
    II,  // Bright giant
    III, // Giant
    IV,  // Subgiant
    V,   // Main sequence (dwarf)
    VI,  // Subdwarf
    VII, // White dwarf
}

// Parse spectral type string into components
fn parse_spectral_type(spectral: &str) -> (SpectralClass, u8, LuminosityClass) {
    if spectral.is_empty() {
        return (SpectralClass::Unknown, 0, LuminosityClass::V);
    }

    let mut chars = spectral.chars();

    // Parse spectral class
    let class = match chars.next() {
        Some('O') | Some('o') => SpectralClass::O,
        Some('B') | Some('b') => SpectralClass::B,
        Some('A') | Some('a') => SpectralClass::A,
        Some('F') | Some('f') => SpectralClass::F,
        Some('G') | Some('g') => SpectralClass::G,
        Some('K') | Some('k') => SpectralClass::K,
        Some('M') | Some('m') => SpectralClass::M,
        Some('L') | Some('l') => SpectralClass::L,
        Some('T') | Some('t') => SpectralClass::T,
        Some('Y') | Some('y') => SpectralClass::Y,
        Some('C') | Some('c') => SpectralClass::C,
        Some('S') | Some('s') => SpectralClass::S,
        Some('W') | Some('w') => SpectralClass::W,
        _ => SpectralClass::Unknown,
    };

    // Parse subclass (0-9)
    let subclass = chars
        .next()
        .and_then(|c| c.to_digit(10))
        .map(|d| d as u8)
        .unwrap_or(5);

    // Parse luminosity class
    let rest: String = chars.collect();
    let luminosity = match rest.as_str() {
        "Ia" | "IA" => LuminosityClass::Ia,
        "Ib" | "IB" => LuminosityClass::Ib,
        "II" => LuminosityClass::II,
        "III" => LuminosityClass::III,
        "IV" => LuminosityClass::IV,
        "V" | "" => LuminosityClass::V,
        "VI" => LuminosityClass::VI,
        "VII" => LuminosityClass::VII,
        _ => LuminosityClass::V,
    };

    (class, subclass, luminosity)
}

// Planetary body physical properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanetPhysical {
    pub mass_kg: f64,
    pub radius_m: f64,
    pub density_kgm3: Option<f64>,
    pub gravity_mps2: f64,
    pub escape_velocity_mps: f64,
    pub geometric_albedo: Option<f64>,
    pub bond_albedo: Option<f64>,
    pub surface_temperature_k: Option<f64>,
    pub atmosphere: Option<Atmosphere>,
    pub composition: PlanetaryComposition,
    pub surface_water_percent: Option<f64>,
    pub habitability_score: Option<f64>,
    pub population: Option<f64>,
    pub magnetic_field_t: Option<f64>,
    pub ring_system: Option<RingSystem>,
}

impl Default for PlanetPhysical {
    fn default() -> Self {
        let mass = Units::EARTH_MASS;
        let radius = Units::EARTH_RADIUS;
        let gravity = 9.81;
        let escape_vel = (2.0 * gravity * radius).sqrt();

        Self {
            mass_kg: mass,
            radius_m: radius,
            density_kgm3: Some(5515.0),  // Earth density
            gravity_mps2: gravity,
            escape_velocity_mps: escape_vel,
            geometric_albedo: Some(0.367),  // Earth
            bond_albedo: Some(0.306),
            surface_temperature_k: Some(288.0),
            atmosphere: Some(Atmosphere::earth_like()),
            composition: PlanetaryComposition::Terrestrial,
            surface_water_percent: Some(71.0),
            habitability_score: Some(1.0),
            population: None,
            magnetic_field_t: Some(5e-5),  // Earth's field
            ring_system: None,
        }
    }
}

// Planetary composition types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlanetaryComposition {
    Terrestrial,    // Rocky, Earth-like
    GasGiant,       // Jupiter-like
    IceGiant,       // Neptune-like
    Metallic,       // High metal content
    Carbon,         // Carbon-rich
    Ocean,          // Water world
    Ice,            // Frozen world
    Lava,           // Molten surface
    Desert,         // Arid world
    Custom(String),
}

// Atmospheric composition and properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atmosphere {
    pub surface_pressure_pa: f64,
    pub scale_height_m: f64,
    pub composition: Vec<AtmosphericComponent>,
    pub greenhouse_factor: f64,
    pub transparency: f64,  // 0.0 = opaque, 1.0 = transparent
    pub breathable: bool,
}

impl Atmosphere {
    pub fn earth_like() -> Self {
        Self {
            surface_pressure_pa: 101325.0,
            scale_height_m: 8500.0,
            composition: vec![
                AtmosphericComponent {
                    gas: "N2".to_string(),
                    percentage: 78.08,
                },
                AtmosphericComponent {
                    gas: "O2".to_string(),
                    percentage: 20.95,
                },
                AtmosphericComponent {
                    gas: "Ar".to_string(),
                    percentage: 0.93,
                },
                AtmosphericComponent {
                    gas: "CO2".to_string(),
                    percentage: 0.04,
                },
            ],
            greenhouse_factor: 1.0,
            transparency: 0.8,
            breathable: true,
        }
    }

    pub fn mars_like() -> Self {
        Self {
            surface_pressure_pa: 610.0,
            scale_height_m: 11000.0,
            composition: vec![
                AtmosphericComponent {
                    gas: "CO2".to_string(),
                    percentage: 95.32,
                },
                AtmosphericComponent {
                    gas: "N2".to_string(),
                    percentage: 2.7,
                },
                AtmosphericComponent {
                    gas: "Ar".to_string(),
                    percentage: 1.6,
                },
            ],
            greenhouse_factor: 0.1,
            transparency: 0.9,
            breathable: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AtmosphericComponent {
    pub gas: String,
    pub percentage: f64,
}

// Ring system for planets
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RingSystem {
    pub inner_radius_m: f64,
    pub outer_radius_m: f64,
    pub thickness_m: f64,
    pub opacity: f64,
    pub composition: String,
}

// Station physical properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationPhysical {
    pub dry_mass_kg: Option<f64>,
    pub cargo_capacity_kg: Option<f64>,
    pub docking_ports: u32,
    pub population_capacity: Option<u64>,
    pub current_population: Option<u64>,
    pub power_generation_w: Option<f64>,
    pub hull_material: Option<String>,
    pub station_type: StationType,
}

impl Default for StationPhysical {
    fn default() -> Self {
        Self {
            dry_mass_kg: Some(1e9),  // 1 million tons
            cargo_capacity_kg: Some(1e8),
            docking_ports: 4,
            population_capacity: Some(10000),
            current_population: None,
            power_generation_w: Some(1e9),  // 1 GW
            hull_material: Some("Composite".to_string()),
            station_type: StationType::Trading,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StationType {
    Research,
    Military,
    Trading,
    Mining,
    Agricultural,
    Industrial,
    Residential,
    Medical,
    Shipyard,
    Custom(String),
}

// Asteroid belt properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeltPhysical {
    pub inner_radius_m: f64,
    pub outer_radius_m: f64,
    pub thickness_m: f64,
    pub total_mass_kg: Option<f64>,
    pub particle_density: Option<f64>,  // particles per cubic km
    pub average_particle_size_m: Option<f64>,
    pub composition: String,
}

// Individual asteroid properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsteroidPhysical {
    pub mass_kg: f64,
    pub radius_m: f64,
    pub composition: AsteroidComposition,
    pub rotation_period_hours: Option<f64>,
    pub porosity: Option<f64>,
    pub resource_value: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AsteroidComposition {
    Carbonaceous,  // C-type
    Silicate,      // S-type
    Metallic,      // M-type
    Icy,           // D-type
    Mixed,
    Custom(String),
}

// Generic physical properties for custom bodies
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericPhysical {
    pub mass_kg: Option<f64>,
    pub radius_m: Option<f64>,
    pub density_kgm3: Option<f64>,
    pub temperature_k: Option<f64>,
    pub custom_properties: std::collections::HashMap<String, serde_json::Value>,
}