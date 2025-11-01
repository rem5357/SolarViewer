// Database models for StellarForge PostgreSQL tables

use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

/// Session (saved galaxy or subsection)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbSession {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub session_type: String,
    pub parent_session_id: Option<Uuid>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub created_by: Option<String>,

    // Spatial bounds (PostGIS geometry stored as WKB)
    #[sqlx(skip)]
    pub bounds_center: Option<Vec<u8>>,
    pub bounds_radius_ly: Option<f64>,
    #[sqlx(skip)]
    pub bounds_polygon: Option<Vec<u8>>,

    // Statistics
    pub total_systems: Option<i32>,
    pub total_stars: Option<i32>,
    pub total_planets: Option<i32>,
    pub total_populated_worlds: Option<i32>,
    pub total_routes: Option<i32>,
    pub total_political_entities: Option<i32>,

    // Metadata
    pub metadata: Option<Json<serde_json::Value>>,
    pub tags: Option<Vec<String>>,
}

/// Star system
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbStarSystem {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub system_type: String,

    // Spatial data (PostGIS geometry as WKB)
    #[sqlx(skip)]
    pub position: Vec<u8>,
    pub galactic_longitude: Option<f64>,
    pub galactic_latitude: Option<f64>,
    pub distance_from_sol_ly: Option<f64>,

    // Legacy support
    pub legacy_position: Option<Vec<f64>>,

    // System properties
    #[sqlx(skip)]
    pub barycenter: Option<Vec<u8>>,
    pub total_mass_solar: Option<f64>,
    pub habitable_zone_inner_au: Option<f64>,
    pub habitable_zone_outer_au: Option<f64>,

    // Discovery
    pub discovered_date: Option<NaiveDate>,
    pub discovered_by: Option<String>,
    pub catalog_designation: Option<String>,

    // Metadata
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Json<serde_json::Value>>,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Celestial body (star, planet, moon, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbBody {
    pub id: Uuid,
    pub session_id: Uuid,
    pub system_id: Option<Uuid>,
    pub parent_body_id: Option<Uuid>,

    pub name: String,
    pub body_kind: String,

    // Position
    #[sqlx(skip)]
    pub position: Option<Vec<u8>>,
    pub relative_position: Option<Vec<f64>>,

    // Orbital parameters
    pub orbital_radius_au: Option<f64>,
    pub eccentricity: Option<f64>,
    pub inclination_deg: Option<f64>,
    pub longitude_ascending_node_deg: Option<f64>,
    pub argument_periapsis_deg: Option<f64>,
    pub mean_anomaly_deg: Option<f64>,
    pub orbital_period_days: Option<f64>,

    // Physical properties (JSONB)
    pub physical_properties: Option<Json<PhysicalProperties>>,

    // Rotation
    pub rotation_period_hours: Option<f64>,
    pub axial_tilt_deg: Option<f64>,
    pub retrograde_rotation: Option<bool>,

    // Discovery
    pub discovered_date: Option<NaiveDate>,
    pub discovered_by: Option<String>,
    pub catalog_designation: Option<String>,

    // Visual
    pub visible: Option<bool>,
    pub color_rgb: Option<Vec<i32>>,
    pub render_distance_ly: Option<f64>,

    // Metadata
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Json<serde_json::Value>>,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Physical properties for bodies (stored as JSONB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalProperties {
    // Star properties
    pub spectral_type: Option<String>,
    pub mass_solar: Option<f64>,
    pub radius_solar: Option<f64>,
    pub luminosity_solar: Option<f64>,
    pub temperature_k: Option<f64>,
    pub age_gyr: Option<f64>,

    // Planet properties
    pub mass_earth: Option<f64>,
    pub radius_earth: Option<f64>,
    pub density_gcc: Option<f64>,
    pub gravity_g: Option<f64>,
    pub water_percent: Option<f64>,
    pub population: Option<i64>,
    pub habitability: Option<f64>,

    // Atmosphere
    pub atmosphere: Option<Atmosphere>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atmosphere {
    pub composition: Vec<AtmosphericComponent>,
    pub surface_pressure_kpa: f64,
    pub breathable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphericComponent {
    pub gas: String,
    pub percentage: f64,
}

/// Political entity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbPoliticalEntity {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub short_name: Option<String>,
    pub government_type: String,

    // Capital
    pub capital_system_id: Option<Uuid>,
    pub founded_date: Option<NaiveDate>,
    pub dissolved_date: Option<NaiveDate>,

    // Properties
    pub population_total: Option<i64>,
    pub gdp_credits: Option<sqlx::types::Decimal>,
    pub military_strength: Option<i32>,
    pub tech_level: Option<i32>,
    pub stability_index: Option<sqlx::types::Decimal>,

    // Visual
    pub primary_color_rgb: Option<Vec<i32>>,
    pub secondary_color_rgb: Option<Vec<i32>>,
    pub flag_symbol: Option<String>,
    pub influence_opacity: Option<sqlx::types::Decimal>,

    // Metadata
    pub description: Option<String>,
    pub ideology: Option<String>,
    pub leader_name: Option<String>,
    pub leader_title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Json<serde_json::Value>>,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// System membership (political control)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbSystemMembership {
    pub id: Uuid,
    pub session_id: Uuid,
    pub system_id: Uuid,
    pub political_entity_id: Uuid,

    pub control_type: String,
    pub control_strength: Option<sqlx::types::Decimal>,

    pub acquired_date: Option<NaiveDate>,
    pub lost_date: Option<NaiveDate>,

    pub economic_value: Option<i32>,
    pub strategic_value: Option<i32>,

    pub metadata: Option<Json<serde_json::Value>>,
}

/// Influence zone
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbInfluenceZone {
    pub id: Uuid,
    pub session_id: Uuid,
    pub political_entity_id: Uuid,

    #[sqlx(skip)]
    pub zone_geometry: Vec<u8>,  // PostGIS geometry
    pub zone_type: String,

    pub base_radius_ly: f64,
    pub strength_multiplier: Option<sqlx::types::Decimal>,

    pub render_layer: Option<i32>,
    pub opacity_override: Option<sqlx::types::Decimal>,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Route
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbRoute {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub route_type_id: Option<Uuid>,

    pub is_active: Option<bool>,
    pub is_bidirectional: Option<bool>,
    pub traffic_volume: Option<i32>,
    pub danger_level: Option<i32>,
    pub travel_time_days: Option<f64>,

    pub trade_value_credits: Option<sqlx::types::Decimal>,
    pub toll_cost_credits: Option<sqlx::types::Decimal>,

    pub controlled_by_entity_id: Option<Uuid>,

    // Visual
    pub color_rgb: Option<Vec<i32>>,
    pub line_style: Option<String>,
    pub line_width: Option<sqlx::types::Decimal>,
    pub render_priority: Option<i32>,

    // Metadata
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Json<serde_json::Value>>,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Route waypoint
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbRouteWaypoint {
    pub id: Uuid,
    pub route_id: Uuid,
    pub sequence_number: i32,

    pub system_id: Option<Uuid>,
    #[sqlx(skip)]
    pub position: Option<Vec<u8>>,  // PostGIS geometry

    pub waypoint_type: Option<String>,
    pub stop_duration_hours: Option<f64>,
    pub services_available: Option<Vec<String>>,

    pub hazard_description: Option<String>,
    pub hazard_level: Option<i32>,

    pub metadata: Option<Json<serde_json::Value>>,
}

/// Sector
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbSector {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub sector_code: Option<String>,

    #[sqlx(skip)]
    pub bounds: Vec<u8>,  // PostGIS geometry
    #[sqlx(skip)]
    pub center_point: Option<Vec<u8>>,
    pub volume_ly3: Option<f64>,

    pub sector_type: Option<String>,
    pub security_level: Option<i32>,
    pub development_level: Option<i32>,

    pub administrated_by: Option<Uuid>,

    pub system_count: Option<i32>,
    pub population_total: Option<i64>,

    // Visual
    pub color_rgb: Option<Vec<i32>>,
    pub border_color_rgb: Option<Vec<i32>>,
    pub opacity: Option<sqlx::types::Decimal>,

    // Metadata
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Json<serde_json::Value>>,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Group
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbGroup {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub group_type: String,

    pub parent_group_id: Option<Uuid>,

    // Visual
    pub show_boundary: Option<bool>,
    pub boundary_color_rgb: Option<Vec<i32>>,
    pub highlight_members: Option<bool>,
    pub member_color_rgb: Option<Vec<i32>>,

    // Metadata
    pub description: Option<String>,
    pub purpose: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Json<serde_json::Value>>,

    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Group member
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub system_id: Option<Uuid>,
    pub body_id: Option<Uuid>,

    pub member_role: Option<String>,
    pub joined_date: Option<NaiveDate>,

    pub notes: Option<String>,
    pub metadata: Option<Json<serde_json::Value>>,
}