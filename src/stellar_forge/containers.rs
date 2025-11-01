// Container hierarchy and management for StellarForge

use crate::stellar_forge::core::{
    Id, State, Vec3, Identified, Container, ContainerError, Spatial,
};
use crate::stellar_forge::bodies::{StellarBody, BodyKind, SpatialParent};
use crate::stellar_forge::frames::{Frame, FrameHierarchy, FrameKind};
use crate::stellar_forge::motion::MotionModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

// Abstract container trait for all container types
pub trait StellarContainer: Container<Item = StellarBody> + Identified {
    fn container_type(&self) -> ContainerType;
    fn coordinates(&self) -> CoordinateReference;
    fn can_accept(&self, body: &StellarBody) -> bool;
    fn total_mass_kg(&self) -> Option<f64>;
    fn bounds(&self) -> Option<Bounds>;
    fn validate_hierarchy(&self) -> Result<(), String>;
}

// Types of containers
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ContainerType {
    Galaxy,
    Sector,
    StarSystem,
    MultiStarSystem,
    PlanetarySystem,
    MoonSystem,
    AsteroidBelt,
    Station,
    Fleet,
    PoliticalRegion,
    Custom,
}

// Coordinate reference for containers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CoordinateReference {
    Galactic { origin: Vec3 },                // Galactic coordinates
    Barycentric { system_id: Id },           // System barycenter
    Stellar { star_id: Id },                 // Star-centered
    Planetary { planet_id: Id },             // Planet-centered
    Relative { parent_id: Id, offset: Vec3 }, // Relative to parent
}

// Spatial bounds for containers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
    pub shape: BoundShape,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BoundShape {
    Box,
    Sphere { radius: f64 },
    Cylinder { radius: f64, height: f64 },
    Torus { major_radius: f64, minor_radius: f64 },
    Custom(String),
}

// The Galaxy - the ultimate container
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Galaxy {
    pub id: Id,
    pub name: String,
    pub sectors: Vec<Sector>,
    pub star_systems: Vec<StarSystem>,
    pub rogue_objects: Vec<StellarBody>,  // Objects not in any system
    pub frame_hierarchy: FrameHierarchy,
    pub metadata: GalaxyMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GalaxyMetadata {
    pub size_ly: Vec3,
    pub total_stars: u64,
    pub creation_date: OffsetDateTime,
    pub last_modified: OffsetDateTime,
    pub description: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Galaxy {
    pub fn new(name: impl Into<String>) -> Self {
        let mut frame_hierarchy = FrameHierarchy::new();
        let galactic_frame = Frame::new_galactic_iau("Galactic IAU");
        frame_hierarchy.add_frame(galactic_frame).ok();

        Self {
            id: Id::new_v4(),
            name: name.into(),
            sectors: Vec::new(),
            star_systems: Vec::new(),
            rogue_objects: Vec::new(),
            frame_hierarchy,
            metadata: GalaxyMetadata {
                size_ly: Vec3::new(100000.0, 100000.0, 1000.0),  // Milky Way-like
                total_stars: 0,
                creation_date: OffsetDateTime::now_utc(),
                last_modified: OffsetDateTime::now_utc(),
                description: None,
                properties: HashMap::new(),
            },
        }
    }

    pub fn add_sector(&mut self, sector: Sector) -> Result<(), ContainerError> {
        if self.sectors.iter().any(|s| s.id == sector.id) {
            return Err(ContainerError::ChildAlreadyExists(sector.id));
        }
        self.sectors.push(sector);
        self.metadata.last_modified = OffsetDateTime::now_utc();
        Ok(())
    }

    pub fn add_star_system(&mut self, system: StarSystem) -> Result<(), ContainerError> {
        if self.star_systems.iter().any(|s| s.id == system.id) {
            return Err(ContainerError::ChildAlreadyExists(system.id));
        }

        // Add system frame to hierarchy
        let system_frame = Frame::new_barycentric(
            format!("{} Barycenter", system.name),
            self.frame_hierarchy.frames.values().next().unwrap().id,  // Galactic frame
            system.galactic_position(),
            OffsetDateTime::now_utc(),
        );
        self.frame_hierarchy.add_frame(system_frame).ok();

        self.star_systems.push(system);
        self.metadata.total_stars += 1;
        self.metadata.last_modified = OffsetDateTime::now_utc();
        Ok(())
    }

    pub fn add_rogue_object(&mut self, mut object: StellarBody) -> Result<(), ContainerError> {
        object.spatial_parent = SpatialParent::Frame(
            self.frame_hierarchy.frames.values().next().unwrap().id
        );
        self.rogue_objects.push(object);
        self.metadata.last_modified = OffsetDateTime::now_utc();
        Ok(())
    }

    pub fn find_system(&self, id: Id) -> Option<&StarSystem> {
        self.star_systems.iter().find(|s| s.id == id)
    }

    pub fn find_system_mut(&mut self, id: Id) -> Option<&mut StarSystem> {
        self.star_systems.iter_mut().find(|s| s.id == id)
    }

    pub fn find_body(&self, id: Id) -> Option<&StellarBody> {
        // Search in systems
        for system in &self.star_systems {
            if let Some(body) = system.find_body_recursive(id) {
                return Some(body);
            }
        }

        // Search in rogue objects
        self.rogue_objects.iter().find(|b| b.id == id)
    }

    pub fn systems_within(&self, center: Vec3, radius_m: f64) -> Vec<&StarSystem> {
        self.star_systems
            .iter()
            .filter(|s| (s.galactic_position() - center).norm() <= radius_m)
            .collect()
    }

    pub fn systems_within_ly(&self, center: Vec3, radius_ly: f64) -> Vec<&StarSystem> {
        let radius_m = radius_ly * crate::stellar_forge::core::Units::LIGHT_YEAR;
        self.systems_within(center, radius_m)
    }

    pub fn nearest_systems(&self, position: Vec3, count: usize) -> Vec<(&StarSystem, f64)> {
        let mut systems_with_distance: Vec<_> = self.star_systems
            .iter()
            .map(|s| (s, (s.galactic_position() - position).norm()))
            .collect();

        systems_with_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        systems_with_distance.truncate(count);
        systems_with_distance
    }
}

// Sector - a region of the galaxy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sector {
    pub id: Id,
    pub name: String,
    pub bounds: Bounds,
    pub system_ids: Vec<Id>,  // References to systems in this sector
}

// Star system container
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StarSystem {
    pub id: Id,
    pub name: String,
    pub galactic_coordinates: crate::stellar_forge::coordinates::GalacticCoordinates,
    pub system_type: SystemType,
    pub stars: Vec<StellarBody>,
    pub planets: Vec<StellarBody>,
    pub belts: Vec<StellarBody>,
    pub stations: Vec<StellarBody>,
    pub other_bodies: Vec<StellarBody>,
    pub barycenter: Vec3,
    pub frame_id: Id,
    /// Legacy position for compatibility
    pub legacy_position: Option<Vec3>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SystemType {
    Single,      // Single star
    Binary,      // Two stars
    Multiple(u8), // Multiple stars (count)
    Cluster,     // Star cluster
    Nebula,      // Nebula system
}

impl StarSystem {
    pub fn new_single_star(
        name: impl Into<String>,
        galactic_coords: crate::stellar_forge::coordinates::GalacticCoordinates,
        star: StellarBody,
    ) -> Self {
        let mut system = Self {
            id: Id::new_v4(),
            name: name.into(),
            galactic_coordinates: galactic_coords,
            system_type: SystemType::Single,
            stars: vec![star],
            planets: Vec::new(),
            belts: Vec::new(),
            stations: Vec::new(),
            other_bodies: Vec::new(),
            barycenter: Vec3::zeros(),
            frame_id: Id::new_v4(),
            legacy_position: None,
        };
        system.update_barycenter();
        system
    }

    pub fn new_binary(
        name: impl Into<String>,
        galactic_coords: crate::stellar_forge::coordinates::GalacticCoordinates,
        star1: StellarBody,
        star2: StellarBody,
    ) -> Self {
        let mut system = Self {
            id: Id::new_v4(),
            name: name.into(),
            galactic_coordinates: galactic_coords,
            system_type: SystemType::Binary,
            stars: vec![star1, star2],
            planets: Vec::new(),
            belts: Vec::new(),
            stations: Vec::new(),
            other_bodies: Vec::new(),
            barycenter: Vec3::zeros(),
            frame_id: Id::new_v4(),
            legacy_position: None,
        };
        system.update_barycenter();
        system
    }

    /// Create from Astrosynthesis coordinates (for import)
    pub fn from_astrosynthesis(
        name: impl Into<String>,
        astro_x: f64,
        astro_y: f64,
        astro_z: f64,
        star: StellarBody,
    ) -> Self {
        use crate::stellar_forge::coordinates::CoordinateTransform;

        let galactic_coords = CoordinateTransform::astrosynthesis_to_galactic(astro_x, astro_y, astro_z);
        let legacy_pos = Vec3::new(
            astro_x * crate::stellar_forge::core::Units::LIGHT_YEAR,
            astro_y * crate::stellar_forge::core::Units::LIGHT_YEAR,
            astro_z * crate::stellar_forge::core::Units::LIGHT_YEAR,
        );

        let mut system = Self::new_single_star(name, galactic_coords, star);
        system.legacy_position = Some(legacy_pos);
        system
    }

    /// Get position in cartesian galactic coordinates
    pub fn galactic_position(&self) -> Vec3 {
        self.galactic_coordinates.to_cartesian()
    }

    pub fn add_planet(&mut self, mut planet: StellarBody) -> Result<(), ContainerError> {
        if !matches!(planet.kind, BodyKind::Planet | BodyKind::RoguePlanet) {
            return Err(ContainerError::InvalidChildType);
        }

        planet.spatial_parent = SpatialParent::Body(self.id);
        self.planets.push(planet);
        self.update_barycenter();
        Ok(())
    }

    pub fn add_asteroid_belt(&mut self, mut belt: StellarBody) -> Result<(), ContainerError> {
        if belt.kind != BodyKind::AsteroidBelt {
            return Err(ContainerError::InvalidChildType);
        }

        belt.spatial_parent = SpatialParent::Body(self.id);
        self.belts.push(belt);
        Ok(())
    }

    pub fn add_station(&mut self, mut station: StellarBody) -> Result<(), ContainerError> {
        if station.kind != BodyKind::Station {
            return Err(ContainerError::InvalidChildType);
        }

        station.spatial_parent = SpatialParent::Body(self.id);
        self.stations.push(station);
        Ok(())
    }

    pub fn update_barycenter(&mut self) {
        let mut total_mass = 0.0;
        let mut weighted_pos = Vec3::zeros();

        for star in &self.stars {
            if let Some(mass) = star.mass_kg() {
                weighted_pos += star.position() * mass;
                total_mass += mass;
            }
        }

        if total_mass > 0.0 {
            self.barycenter = weighted_pos / total_mass;
        }
    }

    pub fn find_body_recursive(&self, id: Id) -> Option<&StellarBody> {
        // Check stars
        for star in &self.stars {
            if star.id == id {
                return Some(star);
            }
            if let Some(found) = star.find_descendant(id) {
                return Some(found);
            }
        }

        // Check planets
        for planet in &self.planets {
            if planet.id == id {
                return Some(planet);
            }
            if let Some(found) = planet.find_descendant(id) {
                return Some(found);
            }
        }

        // Check other bodies
        for body in self.belts.iter().chain(&self.stations).chain(&self.other_bodies) {
            if body.id == id {
                return Some(body);
            }
            if let Some(found) = body.find_descendant(id) {
                return Some(found);
            }
        }

        None
    }

    pub fn total_mass(&self) -> f64 {
        let mut mass = 0.0;

        for star in &self.stars {
            if let Some(m) = star.total_mass_kg() {
                mass += m;
            }
        }

        for planet in &self.planets {
            if let Some(m) = planet.total_mass_kg() {
                mass += m;
            }
        }

        mass
    }

    pub fn habitable_zone(&self) -> Option<(f64, f64)> {
        // Simplified habitable zone calculation
        if let Some(primary) = self.stars.first() {
            if let Some(physical) = &primary.physical {
                if let crate::stellar_forge::physical::Physical::Star(star) = physical {
                    let luminosity_solar = star.luminosity_w / 3.828e26;
                    let inner = 0.95 * luminosity_solar.sqrt();  // AU
                    let outer = 1.37 * luminosity_solar.sqrt();  // AU
                    return Some((
                        inner * crate::stellar_forge::core::Units::AU,
                        outer * crate::stellar_forge::core::Units::AU,
                    ));
                }
            }
        }
        None
    }

    pub fn is_binary(&self) -> bool {
        matches!(self.system_type, SystemType::Binary)
    }

    pub fn is_multiple(&self) -> bool {
        matches!(self.system_type, SystemType::Multiple(_))
    }
}

// Political or organizational container
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoliticalRegion {
    pub id: Id,
    pub name: String,
    pub government_type: String,
    pub capital_system_id: Option<Id>,
    pub member_system_ids: Vec<Id>,
    pub claimed_regions: Vec<Bounds>,
    pub founded_date: Option<OffsetDateTime>,
    pub properties: HashMap<String, serde_json::Value>,
}

impl PoliticalRegion {
    pub fn new(name: impl Into<String>, government: impl Into<String>) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            government_type: government.into(),
            capital_system_id: None,
            member_system_ids: Vec::new(),
            claimed_regions: Vec::new(),
            founded_date: Some(OffsetDateTime::now_utc()),
            properties: HashMap::new(),
        }
    }

    pub fn add_system(&mut self, system_id: Id) {
        if !self.member_system_ids.contains(&system_id) {
            self.member_system_ids.push(system_id);
        }
    }

    pub fn set_capital(&mut self, system_id: Id) {
        self.capital_system_id = Some(system_id);
        self.add_system(system_id);
    }

    pub fn claim_region(&mut self, bounds: Bounds) {
        self.claimed_regions.push(bounds);
    }
}

// Fleet or group container for mobile objects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fleet {
    pub id: Id,
    pub name: String,
    pub flagship_id: Option<Id>,
    pub vessels: Vec<StellarBody>,
    pub formation: Formation,
    pub current_position: State,
    pub destination: Option<Vec3>,
    pub motion_model: Option<MotionModel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Formation {
    Line,
    Wedge,
    Sphere,
    Wall,
    Custom(String),
}

impl Fleet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            flagship_id: None,
            vessels: Vec::new(),
            formation: Formation::Line,
            current_position: State {
                position_m: Vec3::zeros(),
                velocity_mps: Vec3::zeros(),
            },
            destination: None,
            motion_model: None,
        }
    }

    pub fn add_vessel(&mut self, vessel: StellarBody) -> Result<(), ContainerError> {
        if vessel.kind != BodyKind::Vehicle {
            return Err(ContainerError::InvalidChildType);
        }
        self.vessels.push(vessel);
        Ok(())
    }

    pub fn set_flagship(&mut self, vessel_id: Id) -> Result<(), ContainerError> {
        if self.vessels.iter().any(|v| v.id == vessel_id) {
            self.flagship_id = Some(vessel_id);
            Ok(())
        } else {
            Err(ContainerError::ChildNotFound(vessel_id))
        }
    }

    pub fn set_formation(&mut self, formation: Formation) {
        self.formation = formation;
        // Recalculate vessel positions based on formation
        self.arrange_formation();
    }

    fn arrange_formation(&mut self) {
        // Arrange vessels according to formation pattern
        // This would calculate relative positions for each vessel
    }
}