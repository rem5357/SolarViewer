// Services for managing stellar data in StellarForge

use crate::stellar_forge::core::{Id, State, Vec3, CoordinateError};
use crate::stellar_forge::bodies::{StellarBody, BodyKind, SpatialParent};
use crate::stellar_forge::containers::{Galaxy, StarSystem, PoliticalRegion, Fleet};
use crate::stellar_forge::frames::{Frame, FrameHierarchy, FrameKind};
use crate::stellar_forge::motion::{MotionModel, OrbitalElements};
use crate::stellar_forge::physical::Physical;
use crate::stellar_forge::associations::{Association, AssociationManager, Tag};
use std::collections::HashMap;
use time::OffsetDateTime;

// Frame management service
pub trait FrameService {
    fn get_frame(&self, id: Id) -> Option<&Frame>;
    fn add_frame(&mut self, frame: Frame) -> Result<Id, ServiceError>;
    fn remove_frame(&mut self, id: Id) -> Result<(), ServiceError>;
    fn transform_state(&self, state: State, from: Id, to: Id, epoch: OffsetDateTime)
        -> Result<State, CoordinateError>;
    fn to_galactic(&self, frame_id: Id, state: State, epoch: OffsetDateTime)
        -> Result<State, CoordinateError>;
    fn get_hierarchy(&self) -> &FrameHierarchy;
}

// Node/Body management service
pub trait NodeService {
    fn create_body(&mut self, draft: BodyDraft) -> Result<StellarBody, ServiceError>;
    fn update_body(&mut self, body: &StellarBody) -> Result<(), ServiceError>;
    fn delete_body(&mut self, id: Id) -> Result<(), ServiceError>;
    fn get_body(&self, id: Id) -> Option<&StellarBody>;
    fn get_body_mut(&mut self, id: Id) -> Option<&mut StellarBody>;
    fn reparent(&mut self, body_id: Id, new_parent: SpatialParent) -> Result<(), ServiceError>;
    fn add_child(&mut self, parent_id: Id, child: StellarBody) -> Result<(), ServiceError>;
    fn remove_child(&mut self, parent_id: Id, child_id: Id) -> Result<StellarBody, ServiceError>;
    fn find_bodies_by_type(&self, kind: BodyKind) -> Vec<&StellarBody>;
    fn find_bodies_by_tag(&self, tag: &Tag) -> Vec<&StellarBody>;
}

// Query service for spatial and relational queries
pub trait QueryService {
    fn find_within(&self, center: Vec3, radius_m: f64, frame_id: Id, epoch: OffsetDateTime)
        -> Vec<Id>;
    fn find_nearest(&self, point: Vec3, count: usize, frame_id: Id, epoch: OffsetDateTime)
        -> Vec<(Id, f64)>;
    fn find_in_bounds(&self, min: Vec3, max: Vec3, frame_id: Id) -> Vec<Id>;
    fn trace_route(&self, from: Id, to: Id) -> Option<Vec<Id>>;
    fn find_habitable(&self, system_id: Id) -> Vec<Id>;
    fn find_by_association(&self, association_type: &str, group: &str) -> Vec<Id>;
}

// System generation service
pub trait GenerationService {
    fn create_single_star_system(&mut self, spec: SystemSpec) -> Result<StarSystem, ServiceError>;
    fn create_binary_system(&mut self, spec: BinarySystemSpec) -> Result<StarSystem, ServiceError>;
    fn add_planet_to_system(&mut self, system_id: Id, planet: PlanetSpec)
        -> Result<Id, ServiceError>;
    fn add_moon_to_planet(&mut self, planet_id: Id, moon: MoonSpec)
        -> Result<Id, ServiceError>;
    fn create_asteroid_belt(&mut self, system_id: Id, belt: BeltSpec)
        -> Result<Id, ServiceError>;
    fn create_station(&mut self, parent: SpatialParent, station: StationSpec)
        -> Result<Id, ServiceError>;
}

// Service errors
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Entity not found: {0}")]
    NotFound(Id),

    #[error("Invalid parent: {0}")]
    InvalidParent(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Coordinate error: {0}")]
    CoordinateError(#[from] CoordinateError),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Draft for creating bodies
#[derive(Clone, Debug)]
pub struct BodyDraft {
    pub name: String,
    pub kind: BodyKind,
    pub spatial_parent: SpatialParent,
    pub position: Vec3,
    pub velocity: Vec3,
    pub motion: Option<MotionModel>,
    pub physical: Option<Physical>,
    pub tags: Vec<Tag>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl BodyDraft {
    pub fn new(name: impl Into<String>, kind: BodyKind, parent: SpatialParent) -> Self {
        Self {
            name: name.into(),
            kind,
            spatial_parent: parent,
            position: Vec3::zeros(),
            velocity: Vec3::zeros(),
            motion: None,
            physical: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

// Specifications for system generation
#[derive(Clone, Debug)]
pub struct SystemSpec {
    pub name: String,
    pub position: Vec3,  // Galactic position in light-years
    pub star_spec: StarSpec,
    pub planet_count: Option<usize>,
    pub has_asteroid_belt: bool,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug)]
pub struct StarSpec {
    pub spectral_type: String,
    pub mass_solar: f64,
    pub radius_solar: f64,
    pub temperature_k: f64,
    pub luminosity_solar: f64,
    pub age_gyr: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct BinarySystemSpec {
    pub name: String,
    pub position: Vec3,
    pub primary_star: StarSpec,
    pub secondary_star: StarSpec,
    pub separation_au: f64,
    pub orbital_period_years: f64,
}

#[derive(Clone, Debug)]
pub struct PlanetSpec {
    pub name: String,
    pub orbital_radius_au: f64,
    pub eccentricity: f64,
    pub inclination_deg: f64,
    pub mass_earth: f64,
    pub radius_earth: f64,
    pub planet_type: PlanetType,
    pub has_atmosphere: bool,
    pub has_water: bool,
    pub moon_count: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum PlanetType {
    Terrestrial,
    GasGiant,
    IceGiant,
    Desert,
    Ocean,
    Lava,
    Ice,
}

#[derive(Clone, Debug)]
pub struct MoonSpec {
    pub name: String,
    pub orbital_radius_km: f64,
    pub mass_lunar: f64,
    pub radius_lunar: f64,
    pub is_captured: bool,
}

#[derive(Clone, Debug)]
pub struct BeltSpec {
    pub name: String,
    pub inner_radius_au: f64,
    pub outer_radius_au: f64,
    pub total_mass_earth: f64,
    pub composition: String,
}

#[derive(Clone, Debug)]
pub struct StationSpec {
    pub name: String,
    pub station_type: StationType,
    pub population: Option<u64>,
    pub docking_capacity: u32,
}

#[derive(Clone, Debug)]
pub enum StationType {
    Research,
    Military,
    Trading,
    Mining,
    Shipyard,
}

// Main service implementation combining all services
pub struct StellarForgeService {
    pub galaxy: Galaxy,
    pub frame_hierarchy: FrameHierarchy,
    pub association_manager: AssociationManager,
    bodies_index: HashMap<Id, StellarBody>,
    systems_index: HashMap<Id, StarSystem>,
    political_regions: HashMap<Id, PoliticalRegion>,
    fleets: HashMap<Id, Fleet>,
}

impl StellarForgeService {
    pub fn new(galaxy_name: impl Into<String>) -> Self {
        let galaxy = Galaxy::new(galaxy_name);
        Self {
            galaxy,
            frame_hierarchy: FrameHierarchy::new(),
            association_manager: AssociationManager::new(),
            bodies_index: HashMap::new(),
            systems_index: HashMap::new(),
            political_regions: HashMap::new(),
            fleets: HashMap::new(),
        }
    }

    // Initialize with galactic frame
    pub fn initialize(&mut self) -> Result<(), ServiceError> {
        let galactic_frame = Frame::new_galactic("Galactic");
        self.frame_hierarchy.add_frame(galactic_frame)
            .map_err(|e| ServiceError::CoordinateError(e))?;
        Ok(())
    }

    // Add a system to the galaxy
    pub fn add_system(&mut self, system: StarSystem) -> Result<Id, ServiceError> {
        let system_id = system.id;

        // Create barycenter frame for the system
        let galactic_frame_id = self.frame_hierarchy.frames
            .values()
            .find(|f| f.kind == FrameKind::GalacticIAU)
            .map(|f| f.id)
            .ok_or_else(|| ServiceError::InvalidOperation("No galactic frame".into()))?;

        let barycenter_frame = Frame::new_barycentric(
            format!("{} Barycenter", system.name),
            galactic_frame_id,
            system.galactic_position(),
            OffsetDateTime::now_utc(),
        );

        self.frame_hierarchy.add_frame(barycenter_frame)
            .map_err(|e| ServiceError::CoordinateError(e))?;

        // Index all bodies in the system
        for star in &system.stars {
            self.index_body_recursive(star);
        }
        for planet in &system.planets {
            self.index_body_recursive(planet);
        }

        self.systems_index.insert(system_id, system.clone());
        self.galaxy.add_star_system(system)
            .map_err(|e| ServiceError::InvalidOperation(format!("{:?}", e)))?;

        Ok(system_id)
    }

    fn index_body_recursive(&mut self, body: &StellarBody) {
        self.bodies_index.insert(body.id, body.clone());
        for child in &body.children {
            self.index_body_recursive(child);
        }
    }

    // Find a body anywhere in the galaxy
    pub fn find_body(&self, id: Id) -> Option<&StellarBody> {
        self.bodies_index.get(&id).or_else(|| self.galaxy.find_body(id))
    }

    // Add a political region
    pub fn add_political_region(&mut self, region: PoliticalRegion) -> Result<Id, ServiceError> {
        let region_id = region.id;

        // Create associations for member systems
        for system_id in &region.member_system_ids {
            let assoc = Association::new(
                crate::stellar_forge::associations::AssociationType::Political,
                "member",
                format!("Region:{}", region.name),
            );
            self.association_manager.add_association(*system_id, assoc);
        }

        self.political_regions.insert(region_id, region);
        Ok(region_id)
    }

    // Create and add a fleet
    pub fn create_fleet(&mut self, name: impl Into<String>) -> Result<Id, ServiceError> {
        let fleet = Fleet::new(name);
        let fleet_id = fleet.id;
        self.fleets.insert(fleet_id, fleet);
        Ok(fleet_id)
    }

    // Move a fleet to a new position
    pub fn move_fleet(&mut self, fleet_id: Id, new_position: Vec3) -> Result<(), ServiceError> {
        let fleet = self.fleets.get_mut(&fleet_id)
            .ok_or(ServiceError::NotFound(fleet_id))?;

        fleet.current_position.position_m = new_position;
        Ok(())
    }

    // Query systems within range
    pub fn query_systems_in_range(&self, center: Vec3, radius_ly: f64) -> Vec<&StarSystem> {
        self.galaxy.systems_within(center, radius_ly)
    }

    // Find nearest systems
    pub fn query_nearest_systems(&self, position: Vec3, count: usize) -> Vec<(&StarSystem, f64)> {
        self.galaxy.nearest_systems(position, count)
    }

    // Get all bodies with a specific tag
    pub fn find_bodies_with_tag(&self, tag: &Tag) -> Vec<Id> {
        self.association_manager.get_entities_with_tag(tag)
    }

    // Calculate total mass in a volume
    pub fn calculate_mass_in_volume(&self, center: Vec3, radius_ly: f64) -> f64 {
        let systems = self.query_systems_in_range(center, radius_ly);
        systems.iter().map(|s| s.total_mass()).sum()
    }
}

impl FrameService for StellarForgeService {
    fn get_frame(&self, id: Id) -> Option<&Frame> {
        self.frame_hierarchy.get_frame(id)
    }

    fn add_frame(&mut self, frame: Frame) -> Result<Id, ServiceError> {
        let frame_id = frame.id;
        self.frame_hierarchy.add_frame(frame)
            .map_err(|e| ServiceError::CoordinateError(e))?;
        Ok(frame_id)
    }

    fn remove_frame(&mut self, _id: Id) -> Result<(), ServiceError> {
        // Would need to implement frame removal with validation
        Err(ServiceError::InvalidOperation("Frame removal not implemented".into()))
    }

    fn transform_state(&self, state: State, from: Id, to: Id, epoch: OffsetDateTime)
        -> Result<State, CoordinateError> {
        self.frame_hierarchy.transform_state(state, from, to, epoch)
    }

    fn to_galactic(&self, frame_id: Id, state: State, epoch: OffsetDateTime)
        -> Result<State, CoordinateError> {
        let galactic_frame = self.frame_hierarchy.frames
            .values()
            .find(|f| f.kind == FrameKind::GalacticIAU)
            .ok_or(CoordinateError::FrameNotFound(frame_id))?;

        self.frame_hierarchy.transform_state(state, frame_id, galactic_frame.id, epoch)
    }

    fn get_hierarchy(&self) -> &FrameHierarchy {
        &self.frame_hierarchy
    }
}

// Propagation service for computing future states
pub struct PropagationService {
    cache: HashMap<(Id, i64), State>,  // (body_id, epoch_seconds) -> State
}

impl PropagationService {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn propagate_body(&mut self, body: &StellarBody, to_epoch: OffsetDateTime) -> State {
        let cache_key = (body.id, to_epoch.unix_timestamp());

        if let Some(cached) = self.cache.get(&cache_key) {
            return *cached;
        }

        let state = if let Some(motion) = &body.motion {
            motion.propagate(body.state, body.epoch, to_epoch)
        } else {
            body.state
        };

        self.cache.insert(cache_key, state);
        state
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}