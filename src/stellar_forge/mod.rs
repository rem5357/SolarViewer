// StellarForge - Modern data structure for stellar cartography
// Based on Astrosynthesis concepts with enhanced container-based architecture

pub mod core;
pub mod coordinates;
pub mod containers;
pub mod bodies;
pub mod frames;
pub mod motion;
pub mod physical;
pub mod associations;
pub mod services;
pub mod builders;
pub mod storage;
pub mod database;
pub mod cli;
pub mod import;

// Re-export main types for convenience
pub use self::core::*;
pub use self::containers::{StellarContainer, Galaxy};
pub use self::bodies::{StellarBody, BodyKind};
pub use self::frames::{Frame, FrameKind, CoordinateSystem};
pub use self::motion::{MotionModel, OrbitalElements};
pub use self::physical::{Physical, StarPhysical, PlanetPhysical};
pub use self::associations::{Association, Tag};
pub use self::services::{NodeService, FrameService, QueryService};
pub use self::builders::{SystemBuilder, GalaxyBuilder};