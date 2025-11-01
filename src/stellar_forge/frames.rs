// Coordinate frames and transformations for StellarForge

use crate::stellar_forge::core::{Id, State, Transform, Vec3, Quaternion, CoordinateError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum FrameKind {
    /// IAU standard galactic coordinates (Sun at origin)
    GalacticIAU,
    /// ICRS - International Celestial Reference System
    ICRS,
    /// System barycenter (center of mass)
    Barycentric,
    /// Star-centered frame
    Stellar,
    /// Planet-centered, non-rotating
    Planetary,
    /// Body-fixed rotating frame
    Rotating,
    /// Orbital plane reference frame
    Orbital,
    /// Local frame for stations/ships
    LocalVehicle,
    /// Legacy Astrosynthesis coordinates (for import)
    AstrosynthesisLegacy,
    /// User-defined frames
    Custom,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum CoordinateSystem {
    /// Standard X, Y, Z in meters
    Cartesian,
    /// r, theta (azimuth), phi (elevation)
    Spherical,
    /// r, theta, z
    Cylindrical,
    /// IAU Galactic: l (longitude), b (latitude), distance
    GalacticSpherical,
    /// Equatorial: RA, Dec, distance
    EquatorialSpherical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub id: Id,
    pub name: String,
    pub kind: FrameKind,
    pub parent: Option<Id>,
    pub epoch: OffsetDateTime,
    pub to_parent: Option<Transform>,
    pub coordinate_system: CoordinateSystem,
    pub metadata: FrameMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub description: Option<String>,
    pub is_inertial: bool,
    pub primary_body: Option<Id>,  // The body this frame is centered on
    pub orientation_reference: Option<String>,  // e.g., "ICRS", "J2000"
}

impl Frame {
    /// Create IAU standard galactic coordinate frame
    pub fn new_galactic_iau(name: impl Into<String>) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            kind: FrameKind::GalacticIAU,
            parent: None,
            epoch: time::macros::datetime!(2000-01-01 12:00:00 UTC),
            to_parent: None,
            coordinate_system: CoordinateSystem::Cartesian,
            metadata: FrameMetadata {
                description: Some("IAU Galactic coordinate frame (Sun at origin, X toward GC)".into()),
                is_inertial: true,
                primary_body: None,
                orientation_reference: Some("IAU".into()),
            },
        }
    }

    /// Create ICRS frame (modern standard)
    pub fn new_icrs(name: impl Into<String>) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            kind: FrameKind::ICRS,
            parent: None,
            epoch: time::macros::datetime!(2000-01-01 12:00:00 UTC),
            to_parent: None,
            coordinate_system: CoordinateSystem::Cartesian,
            metadata: FrameMetadata {
                description: Some("International Celestial Reference System".into()),
                is_inertial: true,
                primary_body: None,
                orientation_reference: Some("ICRS".into()),
            },
        }
    }

    pub fn new_barycentric(
        name: impl Into<String>,
        parent: Id,
        position_in_parent: Vec3,
        epoch: OffsetDateTime,
    ) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            kind: FrameKind::Barycentric,
            parent: Some(parent),
            epoch,
            to_parent: Some(Transform {
                translation_m: position_in_parent,
                rotation: Quaternion::identity(),
                angular_velocity_rps: None,
            }),
            coordinate_system: CoordinateSystem::Cartesian,
            metadata: FrameMetadata {
                description: Some("System barycenter frame".into()),
                is_inertial: true,
                primary_body: None,
                orientation_reference: None,
            },
        }
    }

    pub fn new_planetary(
        name: impl Into<String>,
        parent: Id,
        body_id: Id,
        epoch: OffsetDateTime,
    ) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            kind: FrameKind::Planetary,
            parent: Some(parent),
            epoch,
            to_parent: None,  // Will be computed from orbital elements
            coordinate_system: CoordinateSystem::Cartesian,
            metadata: FrameMetadata {
                description: Some("Planet-centered inertial frame".into()),
                is_inertial: true,
                primary_body: Some(body_id),
                orientation_reference: None,
            },
        }
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

// Frame hierarchy and transformation service
pub struct FrameHierarchy {
    frames: HashMap<Id, Frame>,
    cache: TransformCache,
}

impl FrameHierarchy {
    pub fn new() -> Self {
        Self {
            frames: HashMap::new(),
            cache: TransformCache::new(),
        }
    }

    pub fn add_frame(&mut self, frame: Frame) -> Result<(), CoordinateError> {
        // Validate parent exists if not root
        if let Some(parent_id) = frame.parent {
            if !self.frames.contains_key(&parent_id) {
                return Err(CoordinateError::FrameNotFound(parent_id));
            }
        }

        self.frames.insert(frame.id, frame);
        self.cache.invalidate();
        Ok(())
    }

    pub fn get_frame(&self, id: Id) -> Option<&Frame> {
        self.frames.get(&id)
    }

    pub fn get_frame_mut(&mut self, id: Id) -> Option<&mut Frame> {
        self.cache.invalidate();
        self.frames.get_mut(&id)
    }

    // Transform a state from one frame to another
    pub fn transform_state(
        &self,
        state: State,
        from_frame: Id,
        to_frame: Id,
        epoch: OffsetDateTime,
    ) -> Result<State, CoordinateError> {
        if from_frame == to_frame {
            return Ok(state);
        }

        // Find path from source to target through common ancestor
        let path = self.find_transform_path(from_frame, to_frame)?;

        let mut current_state = state;

        // Apply transforms along the path
        for (frame_id, direction) in path {
            let frame = self.frames.get(&frame_id)
                .ok_or(CoordinateError::FrameNotFound(frame_id))?;

            if let Some(transform) = &frame.to_parent {
                current_state = match direction {
                    TransformDirection::ToParent => {
                        self.apply_transform(current_state, transform, epoch)
                    }
                    TransformDirection::FromParent => {
                        self.apply_inverse_transform(current_state, transform, epoch)
                    }
                };
            }
        }

        Ok(current_state)
    }

    // Find the path of transforms needed to go from one frame to another
    fn find_transform_path(
        &self,
        from: Id,
        to: Id,
    ) -> Result<Vec<(Id, TransformDirection)>, CoordinateError> {
        // Get paths to root for both frames
        let from_path = self.path_to_root(from)?;
        let to_path = self.path_to_root(to)?;

        // Find common ancestor
        let common_ancestor = self.find_common_ancestor(&from_path, &to_path)?;

        let mut path = Vec::new();

        // Path from source to common ancestor (going up)
        for &frame_id in from_path.iter().take_while(|&&id| id != common_ancestor) {
            path.push((frame_id, TransformDirection::ToParent));
        }

        // Path from common ancestor to target (going down)
        let to_common_idx = to_path.iter().position(|&id| id == common_ancestor).unwrap();
        for &frame_id in to_path[..to_common_idx].iter().rev() {
            path.push((frame_id, TransformDirection::FromParent));
        }

        Ok(path)
    }

    fn path_to_root(&self, mut frame_id: Id) -> Result<Vec<Id>, CoordinateError> {
        let mut path = Vec::new();
        let mut visited = std::collections::HashSet::new();

        loop {
            if !visited.insert(frame_id) {
                return Err(CoordinateError::TransformFailed(
                    "Circular reference in frame hierarchy".into(),
                ));
            }

            path.push(frame_id);

            let frame = self.frames.get(&frame_id)
                .ok_or(CoordinateError::FrameNotFound(frame_id))?;

            if let Some(parent) = frame.parent {
                frame_id = parent;
            } else {
                break;
            }
        }

        Ok(path)
    }

    fn find_common_ancestor(&self, path1: &[Id], path2: &[Id]) -> Result<Id, CoordinateError> {
        // Reverse paths to go from root to node
        let rev1: Vec<_> = path1.iter().rev().cloned().collect();
        let rev2: Vec<_> = path2.iter().rev().cloned().collect();

        // Find where paths diverge
        for i in 0..rev1.len().min(rev2.len()) {
            if rev1[i] == rev2[i] {
                if i == rev1.len() - 1 || i == rev2.len() - 1 || rev1[i + 1] != rev2[i + 1] {
                    return Ok(rev1[i]);
                }
            }
        }

        Err(CoordinateError::DisconnectedFrames)
    }

    fn apply_transform(&self, state: State, transform: &Transform, _epoch: OffsetDateTime) -> State {
        // Apply rotation then translation
        let rotated_pos = transform.rotation * state.position_m;
        let rotated_vel = transform.rotation * state.velocity_mps;

        // Add angular velocity contribution if rotating frame
        let velocity = if let Some(omega) = transform.angular_velocity_rps {
            rotated_vel + omega.cross(&rotated_pos)
        } else {
            rotated_vel
        };

        State {
            position_m: rotated_pos + transform.translation_m,
            velocity_mps: velocity,
        }
    }

    fn apply_inverse_transform(&self, state: State, transform: &Transform, _epoch: OffsetDateTime) -> State {
        // Inverse: subtract translation then inverse rotation
        let translated_pos = state.position_m - transform.translation_m;
        let inv_rotation = transform.rotation.inverse();

        // Remove angular velocity contribution if rotating frame
        let velocity = if let Some(omega) = transform.angular_velocity_rps {
            state.velocity_mps - omega.cross(&translated_pos)
        } else {
            state.velocity_mps
        };

        State {
            position_m: inv_rotation * translated_pos,
            velocity_mps: inv_rotation * velocity,
        }
    }

    // Get all frames that are children of the given frame
    pub fn get_child_frames(&self, parent_id: Id) -> Vec<&Frame> {
        self.frames
            .values()
            .filter(|f| f.parent == Some(parent_id))
            .collect()
    }

    // Convert between coordinate systems
    pub fn convert_coordinates(
        &self,
        position: Vec3,
        from_system: CoordinateSystem,
        to_system: CoordinateSystem,
    ) -> Vec3 {
        match (from_system, to_system) {
            (CoordinateSystem::Cartesian, CoordinateSystem::Spherical) => {
                let r = position.norm();
                let theta = position.y.atan2(position.x);
                let phi = (position.z / r).acos();
                Vec3::new(r, theta, phi)
            }
            (CoordinateSystem::Spherical, CoordinateSystem::Cartesian) => {
                let (r, theta, phi) = (position.x, position.y, position.z);
                Vec3::new(
                    r * phi.sin() * theta.cos(),
                    r * phi.sin() * theta.sin(),
                    r * phi.cos(),
                )
            }
            (CoordinateSystem::Cartesian, CoordinateSystem::Cylindrical) => {
                let r = (position.x * position.x + position.y * position.y).sqrt();
                let theta = position.y.atan2(position.x);
                Vec3::new(r, theta, position.z)
            }
            (CoordinateSystem::Cylindrical, CoordinateSystem::Cartesian) => {
                let (r, theta, z) = (position.x, position.y, position.z);
                Vec3::new(r * theta.cos(), r * theta.sin(), z)
            }
            _ if from_system == to_system => position,
            _ => {
                // Convert through Cartesian as intermediate
                let cartesian = self.convert_coordinates(position, from_system, CoordinateSystem::Cartesian);
                self.convert_coordinates(cartesian, CoordinateSystem::Cartesian, to_system)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TransformDirection {
    ToParent,
    FromParent,
}

// Cache for frequently used transforms
struct TransformCache {
    transforms: HashMap<(Id, Id), Transform>,
    valid: bool,
}

impl TransformCache {
    fn new() -> Self {
        Self {
            transforms: HashMap::new(),
            valid: true,
        }
    }

    fn invalidate(&mut self) {
        self.valid = false;
        self.transforms.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_hierarchy() {
        let mut hierarchy = FrameHierarchy::new();

        // Create galactic frame
        let galactic = Frame::new_galactic_iau("Milky Way");
        let galactic_id = galactic.id;
        hierarchy.add_frame(galactic).unwrap();

        // Create system barycenter
        let sol_pos = Vec3::new(8000.0 * crate::stellar_forge::core::Units::PARSEC, 0.0, 0.0);
        let sol = Frame::new_barycentric("Sol System", galactic_id, sol_pos, OffsetDateTime::now_utc());
        let sol_id = sol.id;
        hierarchy.add_frame(sol).unwrap();

        // Test path finding
        let path = hierarchy.path_to_root(sol_id).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], sol_id);
        assert_eq!(path[1], galactic_id);
    }

    #[test]
    fn test_coordinate_conversion() {
        let hierarchy = FrameHierarchy::new();

        // Test Cartesian to Spherical
        let cart = Vec3::new(1.0, 1.0, 1.0);
        let sph = hierarchy.convert_coordinates(cart, CoordinateSystem::Cartesian, CoordinateSystem::Spherical);
        let back = hierarchy.convert_coordinates(sph, CoordinateSystem::Spherical, CoordinateSystem::Cartesian);

        assert!((cart - back).norm() < 1e-10);
    }
}