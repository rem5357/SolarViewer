// Core traits and fundamental types for StellarForge

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;

// Type aliases for clarity
pub type Id = Uuid;
pub type Vec3 = nalgebra::Vector3<f64>;
pub type Quaternion = nalgebra::UnitQuaternion<f64>;

// Universal units (SI internally, display conversions as needed)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Units;

impl Units {
    pub const METER: f64 = 1.0;
    pub const KILOMETER: f64 = 1000.0;
    pub const AU: f64 = 1.496e11;  // Astronomical Unit in meters
    pub const LIGHT_YEAR: f64 = 9.461e15;  // Light year in meters
    pub const PARSEC: f64 = 3.086e16;  // Parsec in meters
    pub const SOLAR_MASS: f64 = 1.989e30;  // Solar mass in kg
    pub const EARTH_MASS: f64 = 5.972e24;  // Earth mass in kg
    pub const SOLAR_RADIUS: f64 = 6.96e8;  // Solar radius in meters
    pub const EARTH_RADIUS: f64 = 6.371e6;  // Earth radius in meters
}

// Core state representation
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct State {
    pub position_m: Vec3,     // Position in meters
    pub velocity_mps: Vec3,   // Velocity in meters per second
}

// Transform between coordinate frames
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub translation_m: Vec3,
    pub rotation: Quaternion,
    pub angular_velocity_rps: Option<Vec3>,  // radians per second
}

// Core identification trait
pub trait Identified {
    fn id(&self) -> Id;
    fn name(&self) -> &str;
    fn set_name(&mut self, name: impl Into<String>);
}

// Trait for objects existing in space
pub trait Spatial {
    fn position(&self) -> Vec3;
    fn velocity(&self) -> Vec3;
    fn state(&self) -> State;
    fn state_at(&self, epoch: OffsetDateTime) -> State;
}

// Trait for objects that can contain other objects
pub trait Container: Identified {
    type Item;

    fn children(&self) -> &[Self::Item];
    fn children_mut(&mut self) -> &mut Vec<Self::Item>;
    fn add_child(&mut self, child: Self::Item) -> Result<(), ContainerError>;
    fn remove_child(&mut self, id: Id) -> Result<Self::Item, ContainerError>;
    fn find_child(&self, id: Id) -> Option<&Self::Item>;
    fn find_child_mut(&mut self, id: Id) -> Option<&mut Self::Item>;
    fn child_count(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.child_count() == 0
    }

    // Recursive search through container hierarchy
    fn find_recursive(&self, id: Id) -> Option<&Self::Item>
    where
        Self::Item: Container<Item = Self::Item> + Identified,
    {
        if let Some(child) = self.find_child(id) {
            return Some(child);
        }

        for child in self.children() {
            if let Some(found) = child.find_recursive(id) {
                return Some(found);
            }
        }
        None
    }
}

// Trait for objects with mass and physical properties
pub trait Massive {
    fn mass_kg(&self) -> Option<f64>;
    fn radius_m(&self) -> Option<f64>;
    fn density_kgm3(&self) -> Option<f64> {
        match (self.mass_kg(), self.radius_m()) {
            (Some(m), Some(r)) if r > 0.0 => {
                let volume = (4.0 / 3.0) * std::f64::consts::PI * r * r * r;
                Some(m / volume)
            }
            _ => None,
        }
    }
}

// Trait for objects with orbital mechanics
pub trait Orbital {
    fn orbital_period_s(&self) -> Option<f64>;
    fn semi_major_axis_m(&self) -> Option<f64>;
    fn eccentricity(&self) -> Option<f64>;
    fn inclination_rad(&self) -> Option<f64>;
    fn is_retrograde(&self) -> bool;
}

// Trait for objects with rotation
pub trait Rotating {
    fn rotation_period_s(&self) -> Option<f64>;
    fn axial_tilt_rad(&self) -> Option<f64>;
    fn rotation_axis(&self) -> Option<Vec3>;
}

// Trait for objects that can be tagged/categorized
pub trait Taggable {
    fn tags(&self) -> &[String];
    fn add_tag(&mut self, tag: impl Into<String>);
    fn remove_tag(&mut self, tag: &str) -> bool;
    fn has_tag(&self, tag: &str) -> bool;
    fn clear_tags(&mut self);
}

// Trait for objects that can be associated with groups
pub trait Associable {
    fn associations(&self) -> &[crate::stellar_forge::associations::Association];
    fn add_association(&mut self, assoc: crate::stellar_forge::associations::Association);
    fn remove_association(&mut self, id: &str) -> bool;
    fn is_member_of(&self, group: &str) -> bool;
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("Child with id {0} not found")]
    ChildNotFound(Id),

    #[error("Child with id {0} already exists")]
    ChildAlreadyExists(Id),

    #[error("Container cannot accept this type of child")]
    InvalidChildType,

    #[error("Maximum container capacity reached")]
    CapacityExceeded,

    #[error("Circular reference detected")]
    CircularReference,
}

#[derive(Debug, thiserror::Error)]
pub enum CoordinateError {
    #[error("Frame {0} not found")]
    FrameNotFound(Id),

    #[error("Cannot transform between disconnected frames")]
    DisconnectedFrames,

    #[error("Coordinate transformation failed: {0}")]
    TransformFailed(String),
}

// Time utilities
pub struct TimeUtils;

impl TimeUtils {
    pub fn j2000() -> OffsetDateTime {
        time::macros::datetime!(2000-01-01 12:00:00 UTC)
    }

    pub fn julian_centuries_since_j2000(epoch: OffsetDateTime) -> f64 {
        let j2000 = Self::j2000();
        let duration = epoch - j2000;
        duration.as_seconds_f64() / (36525.0 * 86400.0)
    }
}