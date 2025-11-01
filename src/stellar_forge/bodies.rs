// Stellar body types and implementations for StellarForge

use crate::stellar_forge::core::{
    Id, State, Vec3, Identified, Spatial, Container, ContainerError,
    Massive, Orbital, Rotating, Taggable, Associable,
};
use crate::stellar_forge::frames::{Frame, FrameKind};
use crate::stellar_forge::motion::MotionModel;
use crate::stellar_forge::physical::Physical;
use crate::stellar_forge::associations::Association;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

// Enumeration of all body types
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum BodyKind {
    Star,
    Planet,
    Moon,
    Station,
    AsteroidBelt,
    Asteroid,
    Comet,
    Planetoid,
    Wreck,
    Artifact,
    Nebula,
    LagrangePoint,
    Vehicle,
    RoguePlanet,
    BinarySystem,  // Container for binary star systems
    Custom(u32),   // User-defined types with ID
}

impl BodyKind {
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            BodyKind::Star
                | BodyKind::Planet
                | BodyKind::Moon
                | BodyKind::Station
                | BodyKind::AsteroidBelt
                | BodyKind::BinarySystem
                | BodyKind::Vehicle
        )
    }

    pub fn is_stellar(&self) -> bool {
        matches!(self, BodyKind::Star | BodyKind::BinarySystem)
    }

    pub fn is_planetary(&self) -> bool {
        matches!(
            self,
            BodyKind::Planet | BodyKind::Moon | BodyKind::Planetoid | BodyKind::RoguePlanet
        )
    }
}

// Reference to either a frame or another body
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpatialParent {
    Frame(Id),
    Body(Id),
}

// Base stellar body structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StellarBody {
    // Core identification
    pub id: Id,
    pub name: String,
    pub kind: BodyKind,

    // Spatial hierarchy
    pub spatial_parent: SpatialParent,
    pub frame_id: Id,
    pub children: Vec<StellarBody>,

    // State and motion
    pub epoch: OffsetDateTime,
    pub state: State,
    pub motion: Option<MotionModel>,

    // Physical properties
    pub physical: Option<Physical>,

    // Rotation
    pub rotation_period_hours: Option<f64>,
    pub axial_tilt_rad: Option<f64>,
    pub retrograde_rotation: bool,

    // Categorization
    pub tags: Vec<String>,
    pub associations: Vec<Association>,

    // Visual/rendering hints
    pub visible: bool,
    pub color: Option<[f32; 3]>,
    pub render_distance: Option<f64>,

    // Custom data
    pub metadata: HashMap<String, serde_json::Value>,
}

impl StellarBody {
    pub fn new(name: impl Into<String>, kind: BodyKind, parent: SpatialParent) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            kind,
            spatial_parent: parent,
            frame_id: Id::nil(),  // Will be set by container
            children: Vec::new(),
            epoch: OffsetDateTime::now_utc(),
            state: State {
                position_m: Vec3::zeros(),
                velocity_mps: Vec3::zeros(),
            },
            motion: None,
            physical: None,
            rotation_period_hours: None,
            axial_tilt_rad: None,
            retrograde_rotation: false,
            tags: Vec::new(),
            associations: Vec::new(),
            visible: true,
            color: None,
            render_distance: None,
            metadata: HashMap::new(),
        }
    }

    // Create a star
    pub fn new_star(
        name: impl Into<String>,
        parent: SpatialParent,
        spectral_type: impl Into<String>,
    ) -> Self {
        let mut body = Self::new(name, BodyKind::Star, parent);
        body.physical = Some(Physical::new_star(spectral_type));
        body
    }

    // Create a planet
    pub fn new_planet(name: impl Into<String>, parent: SpatialParent) -> Self {
        Self::new(name, BodyKind::Planet, parent)
    }

    // Create a moon
    pub fn new_moon(name: impl Into<String>, parent: SpatialParent) -> Self {
        Self::new(name, BodyKind::Moon, parent)
    }

    // Create a space station
    pub fn new_station(name: impl Into<String>, parent: SpatialParent) -> Self {
        Self::new(name, BodyKind::Station, parent)
    }

    // Create an asteroid belt
    pub fn new_belt(name: impl Into<String>, parent: SpatialParent) -> Self {
        Self::new(name, BodyKind::AsteroidBelt, parent)
    }

    // Set position in parent frame
    pub fn set_position(&mut self, position: Vec3) {
        self.state.position_m = position;
    }

    // Set velocity in parent frame
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.state.velocity_mps = velocity;
    }

    // Set orbital motion
    pub fn set_orbital_motion(&mut self, motion: MotionModel) {
        self.motion = Some(motion);
    }

    // Check if this body can contain the given type
    pub fn can_contain(&self, child_kind: BodyKind) -> bool {
        match self.kind {
            BodyKind::Star => matches!(
                child_kind,
                BodyKind::Planet
                    | BodyKind::AsteroidBelt
                    | BodyKind::Comet
                    | BodyKind::Station
                    | BodyKind::Artifact
            ),
            BodyKind::Planet => matches!(
                child_kind,
                BodyKind::Moon | BodyKind::Station | BodyKind::Vehicle | BodyKind::Artifact
            ),
            BodyKind::Moon => matches!(child_kind, BodyKind::Station | BodyKind::Vehicle),
            BodyKind::Station => matches!(child_kind, BodyKind::Vehicle),
            BodyKind::AsteroidBelt => matches!(
                child_kind,
                BodyKind::Asteroid | BodyKind::Station | BodyKind::Wreck
            ),
            BodyKind::BinarySystem => matches!(child_kind, BodyKind::Star),
            _ => false,
        }
    }

    // Get the effective mass including children
    pub fn total_mass_kg(&self) -> Option<f64> {
        let mut mass = self.mass_kg();

        for child in &self.children {
            if let Some(child_mass) = child.total_mass_kg() {
                mass = Some(mass.unwrap_or(0.0) + child_mass);
            }
        }

        mass
    }

    // Calculate barycenter of this body and its children
    pub fn barycenter(&self) -> Vec3 {
        let mut total_mass = 0.0;
        let mut weighted_position = Vec3::zeros();

        // Include self
        if let Some(mass) = self.mass_kg() {
            total_mass += mass;
            weighted_position += self.state.position_m * mass;
        }

        // Include children
        for child in &self.children {
            if let Some(mass) = child.total_mass_kg() {
                total_mass += mass;
                let child_pos = self.state.position_m + child.state.position_m;
                weighted_position += child_pos * mass;
            }
        }

        if total_mass > 0.0 {
            weighted_position / total_mass
        } else {
            self.state.position_m
        }
    }

    // Find all descendants recursively
    pub fn descendants(&self) -> Vec<&StellarBody> {
        let mut result = Vec::new();

        for child in &self.children {
            result.push(child);
            result.extend(child.descendants());
        }

        result
    }

    // Find a descendant by ID
    pub fn find_descendant(&self, id: Id) -> Option<&StellarBody> {
        for child in &self.children {
            if child.id == id {
                return Some(child);
            }
            if let Some(found) = child.find_descendant(id) {
                return Some(found);
            }
        }
        None
    }

    // Find a descendant by ID (mutable)
    pub fn find_descendant_mut(&mut self, id: Id) -> Option<&mut StellarBody> {
        for child in &mut self.children {
            if child.id == id {
                return Some(child);
            }
            if let Some(found) = child.find_descendant_mut(id) {
                return Some(found);
            }
        }
        None
    }

    // Get hierarchical path as string (e.g., "Sol/Earth/Moon")
    pub fn path(&self, separator: &str) -> String {
        // This would need access to parent hierarchy
        // For now, just return the name
        self.name.clone()
    }

    // Validate orbital configuration
    pub fn validate_orbit(&self) -> Result<(), String> {
        if let Some(motion) = &self.motion {
            match motion {
                MotionModel::Keplerian(elements) => {
                    if elements.eccentricity >= 1.0 && !matches!(self.kind, BodyKind::Comet) {
                        return Err("Non-comet body has hyperbolic orbit".into());
                    }
                    if elements.semi_major_axis_m <= 0.0 {
                        return Err("Invalid semi-major axis".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

// Implement core traits
impl Identified for StellarBody {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}

impl Spatial for StellarBody {
    fn position(&self) -> Vec3 {
        self.state.position_m
    }

    fn velocity(&self) -> Vec3 {
        self.state.velocity_mps
    }

    fn state(&self) -> State {
        self.state
    }

    fn state_at(&self, epoch: OffsetDateTime) -> State {
        if let Some(motion) = &self.motion {
            motion.propagate(self.state, self.epoch, epoch)
        } else {
            self.state
        }
    }
}

impl Container for StellarBody {
    type Item = StellarBody;

    fn children(&self) -> &[Self::Item] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Self::Item> {
        &mut self.children
    }

    fn add_child(&mut self, mut child: Self::Item) -> Result<(), ContainerError> {
        // Check if this container can hold this type
        if !self.can_contain(child.kind) {
            return Err(ContainerError::InvalidChildType);
        }

        // Check for duplicate ID
        if self.find_child(child.id).is_some() {
            return Err(ContainerError::ChildAlreadyExists(child.id));
        }

        // Set child's spatial parent to this body
        child.spatial_parent = SpatialParent::Body(self.id);

        self.children.push(child);
        Ok(())
    }

    fn remove_child(&mut self, id: Id) -> Result<Self::Item, ContainerError> {
        if let Some(pos) = self.children.iter().position(|c| c.id == id) {
            Ok(self.children.remove(pos))
        } else {
            Err(ContainerError::ChildNotFound(id))
        }
    }

    fn find_child(&self, id: Id) -> Option<&Self::Item> {
        self.children.iter().find(|c| c.id == id)
    }

    fn find_child_mut(&mut self, id: Id) -> Option<&mut Self::Item> {
        self.children.iter_mut().find(|c| c.id == id)
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }
}

impl Massive for StellarBody {
    fn mass_kg(&self) -> Option<f64> {
        self.physical.as_ref().and_then(|p| p.mass_kg())
    }

    fn radius_m(&self) -> Option<f64> {
        self.physical.as_ref().and_then(|p| p.radius_m())
    }
}

impl Orbital for StellarBody {
    fn orbital_period_s(&self) -> Option<f64> {
        self.motion.as_ref().and_then(|m| m.orbital_period_s())
    }

    fn semi_major_axis_m(&self) -> Option<f64> {
        self.motion.as_ref().and_then(|m| match m {
            MotionModel::Keplerian(e) => Some(e.semi_major_axis_m),
            _ => None,
        })
    }

    fn eccentricity(&self) -> Option<f64> {
        self.motion.as_ref().and_then(|m| match m {
            MotionModel::Keplerian(e) => Some(e.eccentricity),
            _ => None,
        })
    }

    fn inclination_rad(&self) -> Option<f64> {
        self.motion.as_ref().and_then(|m| match m {
            MotionModel::Keplerian(e) => Some(e.inclination_rad),
            _ => None,
        })
    }

    fn is_retrograde(&self) -> bool {
        self.inclination_rad()
            .map(|i| i > std::f64::consts::PI / 2.0)
            .unwrap_or(false)
    }
}

impl Rotating for StellarBody {
    fn rotation_period_s(&self) -> Option<f64> {
        self.rotation_period_hours.map(|h| h * 3600.0)
    }

    fn axial_tilt_rad(&self) -> Option<f64> {
        self.axial_tilt_rad
    }

    fn rotation_axis(&self) -> Option<Vec3> {
        self.axial_tilt_rad.map(|tilt| {
            // Simplified: assume tilt is relative to orbital plane normal
            Vec3::new(tilt.sin(), 0.0, tilt.cos())
        })
    }
}

impl Taggable for StellarBody {
    fn tags(&self) -> &[String] {
        &self.tags
    }

    fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.has_tag(&tag) {
            self.tags.push(tag);
        }
    }

    fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    fn clear_tags(&mut self) {
        self.tags.clear();
    }
}

impl Associable for StellarBody {
    fn associations(&self) -> &[Association] {
        &self.associations
    }

    fn add_association(&mut self, assoc: Association) {
        self.associations.push(assoc);
    }

    fn remove_association(&mut self, id: &str) -> bool {
        if let Some(pos) = self.associations.iter().position(|a| a.id == id) {
            self.associations.remove(pos);
            true
        } else {
            false
        }
    }

    fn is_member_of(&self, group: &str) -> bool {
        self.associations.iter().any(|a| a.group == group)
    }
}