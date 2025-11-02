//! Data mapping between Astrosynthesis and StellarForge schemas

use serde::{Deserialize, Serialize};

/// Astrosynthesis body record from SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstroBody {
    pub id: i64,
    pub system_id: i64,
    pub parent_id: i64,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub radius: f64,
    pub mass: f64,
    pub temperature: f64,
    pub luminosity: f64,
    pub spectral_type: Option<String>,
    pub body_type: String,
    pub description: Option<String>,
}

/// Astrosynthesis route record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstroRoute {
    pub id: i64,
    pub start_body_id: i64,
    pub end_body_id: i64,
    pub distance: f64,
    pub route_type: Option<String>,
}

/// Map Astrosynthesis body_type to StellarForge BodyKind
pub fn map_body_kind(astro_type: &str) -> String {
    match astro_type.to_lowercase().as_str() {
        "star" => "star",
        "planet" => "planet",
        "moon" => "moon",
        "asteroid" => "asteroid",
        "asteroid belt" | "asteroidbelt" => "asteroid_belt",
        "comet" => "comet",
        "station" | "space station" => "station",
        "wreck" => "wreck",
        "artifact" => "artifact",
        "nebula" => "nebula",
        "lagrange point" | "lagrange" => "lagrange_point",
        _ => "custom",
    }
    .to_string()
}

/// Determine if a body is a star system container (multi-star)
pub fn is_multi_star_container(body: &AstroBody) -> bool {
    body.system_id == body.id
        && body.parent_id == 0
        && (body.spectral_type.is_none() || body.spectral_type.as_deref() == Some(""))
        && body.body_type.to_lowercase() == "star"
}

/// Determine if a body is a single star system
pub fn is_single_star_system(body: &AstroBody) -> bool {
    body.system_id == body.id
        && body.parent_id == 0
        && body.spectral_type.is_some()
        && !body.spectral_type.as_deref().unwrap_or("").is_empty()
}

/// Determine if a body is a component star in a multi-star system
pub fn is_component_star(body: &AstroBody, container_id: i64) -> bool {
    body.parent_id == container_id
        && body.spectral_type.is_some()
        && !body.spectral_type.as_deref().unwrap_or("").is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_kind_mapping() {
        assert_eq!(map_body_kind("star"), "star");
        assert_eq!(map_body_kind("planet"), "planet");
        assert_eq!(map_body_kind("asteroid belt"), "asteroid_belt");
        assert_eq!(map_body_kind("unknown"), "custom");
    }

    #[test]
    fn test_star_system_detection() {
        // Single star system
        let single_star = AstroBody {
            id: 100,
            system_id: 100,
            parent_id: 0,
            name: "Sol".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            radius: 1.0,
            mass: 1.0,
            temperature: 5778.0,
            luminosity: 1.0,
            spectral_type: Some("G2V".to_string()),
            body_type: "star".to_string(),
            description: None,
        };

        assert!(is_single_star_system(&single_star));
        assert!(!is_multi_star_container(&single_star));

        // Multi-star container
        let container = AstroBody {
            id: 200,
            system_id: 200,
            parent_id: 0,
            name: "Binary System".to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            radius: 0.0,
            mass: 0.0,
            temperature: 0.0,
            luminosity: 0.0,
            spectral_type: Some("".to_string()),
            body_type: "star".to_string(),
            description: None,
        };

        assert!(!is_single_star_system(&container));
        assert!(is_multi_star_container(&container));
    }
}
