// Repository layer for database operations

use anyhow::Result;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use super::connection::ConnectionPool;
use super::models::*;

/// Repository for session operations
pub struct SessionRepository<'a> {
    pool: &'a ConnectionPool,
}

impl<'a> SessionRepository<'a> {
    pub fn new(pool: &'a ConnectionPool) -> Self {
        Self { pool }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        name: &str,
        description: Option<&str>,
        session_type: &str,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO stellar.sessions (id, name, description, session_type)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(session_type)
        .execute(self.pool.pool())
        .await?;

        Ok(id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: Uuid) -> Result<DbSession> {
        let session = sqlx::query_as::<_, DbSession>(
            r#"
            SELECT * FROM stellar.sessions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool.pool())
        .await?;

        Ok(session)
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<DbSession>> {
        let sessions = sqlx::query_as::<_, DbSession>(
            r#"
            SELECT * FROM stellar.sessions
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.pool.pool())
        .await?;

        Ok(sessions)
    }

    /// Update session statistics
    pub async fn update_session_stats(&self, session_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            SELECT stellar.update_session_stats($1)
            "#,
        )
        .bind(session_id)
        .execute(self.pool.pool())
        .await?;

        Ok(())
    }

    /// Create a subsection from an existing session
    pub async fn create_subsection(
        &self,
        parent_id: Uuid,
        name: &str,
        center_x: f64,
        center_y: f64,
        center_z: f64,
        radius_ly: f64,
    ) -> Result<Uuid> {
        let result: (Uuid,) = sqlx::query_as(
            r#"
            SELECT stellar.create_subsection(
                $1, $2,
                ST_MakePoint($3, $4, $5)::geometry,
                $6
            )
            "#,
        )
        .bind(parent_id)
        .bind(name)
        .bind(center_x * 9.461e15)  // Convert to meters
        .bind(center_y * 9.461e15)
        .bind(center_z * 9.461e15)
        .bind(radius_ly)
        .fetch_one(self.pool.pool())
        .await?;

        Ok(result.0)
    }
}

/// Repository for star system operations
pub struct SystemRepository<'a> {
    pool: &'a ConnectionPool,
}

impl<'a> SystemRepository<'a> {
    pub fn new(pool: &'a ConnectionPool) -> Self {
        Self { pool }
    }

    /// Create a new star system
    pub async fn create_system(
        &self,
        session_id: Uuid,
        name: &str,
        x_ly: f64,
        y_ly: f64,
        z_ly: f64,
        system_type: &str,
    ) -> Result<Uuid> {
        use crate::stellar_forge::coordinates::GalacticCoordinates;

        let id = Uuid::new_v4();

        // Convert to galactic coordinates
        let cart = nalgebra::Vector3::new(
            x_ly * crate::stellar_forge::core::Units::LIGHT_YEAR,
            y_ly * crate::stellar_forge::core::Units::LIGHT_YEAR,
            z_ly * crate::stellar_forge::core::Units::LIGHT_YEAR,
        );
        let galactic = GalacticCoordinates::from_cartesian(cart);

        sqlx::query(
            r#"
            INSERT INTO stellar.star_systems (
                id, session_id, name, system_type,
                position,
                galactic_longitude, galactic_latitude, distance_from_sol_ly
            ) VALUES (
                $1, $2, $3, $4,
                ST_MakePoint($5, $6, $7)::geometry,
                $8, $9, $10
            )
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(name)
        .bind(system_type)
        .bind(x_ly * 9.461e15)  // Convert to meters for PostGIS
        .bind(y_ly * 9.461e15)
        .bind(z_ly * 9.461e15)
        .bind(galactic.longitude_deg())
        .bind(galactic.latitude_deg())
        .bind(galactic.distance_ly())
        .execute(self.pool.pool())
        .await?;

        Ok(id)
    }

    /// Get systems within radius of a point
    pub async fn find_systems_within(
        &self,
        session_id: Uuid,
        center_x: f64,
        center_y: f64,
        center_z: f64,
        radius_ly: f64,
    ) -> Result<Vec<DbStarSystem>> {
        let systems = sqlx::query_as::<_, DbStarSystem>(
            r#"
            SELECT * FROM stellar.star_systems
            WHERE session_id = $1
            AND ST_3DDWithin(
                position,
                ST_MakePoint($2, $3, $4)::geometry,
                $5
            )
            ORDER BY ST_3DDistance(
                position,
                ST_MakePoint($2, $3, $4)::geometry
            )
            "#,
        )
        .bind(session_id)
        .bind(center_x * 9.461e15)
        .bind(center_y * 9.461e15)
        .bind(center_z * 9.461e15)
        .bind(radius_ly * 9.461e15)
        .fetch_all(self.pool.pool())
        .await?;

        Ok(systems)
    }

    /// Find nearest systems to a given system
    pub async fn find_nearest_systems(
        &self,
        session_id: Uuid,
        system_id: Uuid,
        limit: i32,
    ) -> Result<Vec<(DbStarSystem, f64)>> {
        let rows = sqlx::query(
            r#"
            SELECT s.*,
                   ST_3DDistance(s.position, ref.position) / 9.461e15 as distance_ly
            FROM stellar.star_systems s,
                 (SELECT position FROM stellar.star_systems WHERE id = $2) ref
            WHERE s.session_id = $1
            AND s.id != $2
            ORDER BY s.position <-> ref.position
            LIMIT $3
            "#,
        )
        .bind(session_id)
        .bind(system_id)
        .bind(limit)
        .fetch_all(self.pool.pool())
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let system = DbStarSystem {
                id: row.get("id"),
                session_id: row.get("session_id"),
                name: row.get("name"),
                system_type: row.get("system_type"),
                position: Vec::new(),  // Would need proper geometry handling
                galactic_longitude: row.get("galactic_longitude"),
                galactic_latitude: row.get("galactic_latitude"),
                distance_from_sol_ly: row.get("distance_from_sol_ly"),
                legacy_position: row.get("legacy_position"),
                barycenter: None,
                total_mass_solar: row.get("total_mass_solar"),
                habitable_zone_inner_au: row.get("habitable_zone_inner_au"),
                habitable_zone_outer_au: row.get("habitable_zone_outer_au"),
                discovered_date: row.get("discovered_date"),
                discovered_by: row.get("discovered_by"),
                catalog_designation: row.get("catalog_designation"),
                notes: row.get("notes"),
                tags: row.get("tags"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
            };
            let distance: f64 = row.get("distance_ly");
            results.push((system, distance));
        }

        Ok(results)
    }
}

/// Repository for body operations
pub struct BodyRepository<'a> {
    pool: &'a ConnectionPool,
}

impl<'a> BodyRepository<'a> {
    pub fn new(pool: &'a ConnectionPool) -> Self {
        Self { pool }
    }

    /// Create a new celestial body
    pub async fn create_body(
        &self,
        session_id: Uuid,
        system_id: Uuid,
        name: &str,
        body_kind: &str,
        physical: Option<PhysicalProperties>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO stellar.bodies (
                id, session_id, system_id, name, body_kind, physical_properties
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(system_id)
        .bind(name)
        .bind(body_kind)
        .bind(sqlx::types::Json(physical))
        .execute(self.pool.pool())
        .await?;

        Ok(id)
    }

    /// Get all bodies in a system
    pub async fn get_system_bodies(&self, system_id: Uuid) -> Result<Vec<DbBody>> {
        let bodies = sqlx::query_as::<_, DbBody>(
            r#"
            SELECT * FROM stellar.bodies
            WHERE system_id = $1
            ORDER BY
                CASE body_kind
                    WHEN 'star' THEN 1
                    WHEN 'planet' THEN 2
                    WHEN 'moon' THEN 3
                    ELSE 4
                END,
                orbital_radius_au NULLS LAST
            "#,
        )
        .bind(system_id)
        .fetch_all(self.pool.pool())
        .await?;

        Ok(bodies)
    }

    /// Find habitable planets
    pub async fn find_habitable_planets(&self, session_id: Uuid) -> Result<Vec<DbBody>> {
        let bodies = sqlx::query_as::<_, DbBody>(
            r#"
            SELECT * FROM stellar.bodies
            WHERE session_id = $1
            AND body_kind IN ('planet', 'moon')
            AND (physical_properties->>'habitability')::FLOAT > 0.5
            ORDER BY (physical_properties->>'habitability')::FLOAT DESC
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool.pool())
        .await?;

        Ok(bodies)
    }
}

/// Repository for political entity operations
pub struct PoliticalRepository<'a> {
    pool: &'a ConnectionPool,
}

impl<'a> PoliticalRepository<'a> {
    pub fn new(pool: &'a ConnectionPool) -> Self {
        Self { pool }
    }

    /// Create a new political entity
    pub async fn create_entity(
        &self,
        session_id: Uuid,
        name: &str,
        government_type: &str,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO political.entities (id, session_id, name, government_type)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(name)
        .bind(government_type)
        .execute(self.pool.pool())
        .await?;

        Ok(id)
    }

    /// Add a system to a political entity
    pub async fn add_system_control(
        &self,
        session_id: Uuid,
        entity_id: Uuid,
        system_id: Uuid,
        control_type: &str,
        control_strength: f64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO political.system_membership (
                session_id, political_entity_id, system_id,
                control_type, control_strength
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (session_id, system_id, political_entity_id)
            DO UPDATE SET
                control_type = EXCLUDED.control_type,
                control_strength = EXCLUDED.control_strength
            "#,
        )
        .bind(session_id)
        .bind(entity_id)
        .bind(system_id)
        .bind(control_type)
        .bind(control_strength)
        .execute(self.pool.pool())
        .await?;

        Ok(())
    }

    /// Generate influence zone for an entity
    pub async fn generate_influence_zone(
        &self,
        session_id: Uuid,
        entity_id: Uuid,
        base_radius_ly: f64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO political.influence_zones (
                session_id, political_entity_id,
                zone_geometry, base_radius_ly
            )
            SELECT
                $1, $2,
                political.generate_influence_zone($1, $2, $3),
                $3
            "#,
        )
        .bind(session_id)
        .bind(entity_id)
        .bind(base_radius_ly)
        .execute(self.pool.pool())
        .await?;

        Ok(())
    }

    /// Find disputed zones
    pub async fn find_disputed_zones(&self, session_id: Uuid) -> Result<Vec<(Uuid, Uuid)>> {
        let rows = sqlx::query(
            r#"
            SELECT entity_a_id, entity_b_id
            FROM political.find_disputed_zones($1)
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool.pool())
        .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push((
                row.get("entity_a_id"),
                row.get("entity_b_id"),
            ));
        }

        Ok(results)
    }
}

/// Repository for route operations
pub struct RouteRepository<'a> {
    pool: &'a ConnectionPool,
}

impl<'a> RouteRepository<'a> {
    pub fn new(pool: &'a ConnectionPool) -> Self {
        Self { pool }
    }

    /// Create a route from a list of system IDs
    pub async fn create_route_from_systems(
        &self,
        session_id: Uuid,
        name: &str,
        system_ids: &[Uuid],
        route_type: &str,
    ) -> Result<Uuid> {
        let result: (Uuid,) = sqlx::query_as(
            r#"
            SELECT routing.create_route_from_systems($1, $2, $3, $4)
            "#,
        )
        .bind(session_id)
        .bind(name)
        .bind(system_ids)
        .bind(route_type)
        .fetch_one(self.pool.pool())
        .await?;

        Ok(result.0)
    }

    /// Get all routes in a session
    pub async fn get_routes(&self, session_id: Uuid) -> Result<Vec<DbRoute>> {
        let routes = sqlx::query_as::<_, DbRoute>(
            r#"
            SELECT * FROM routing.routes
            WHERE session_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool.pool())
        .await?;

        Ok(routes)
    }
}

/// Repository for group operations
pub struct GroupRepository<'a> {
    pool: &'a ConnectionPool,
}

impl<'a> GroupRepository<'a> {
    pub fn new(pool: &'a ConnectionPool) -> Self {
        Self { pool }
    }

    /// Create a new group
    pub async fn create_group(
        &self,
        session_id: Uuid,
        name: &str,
        group_type: &str,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO stellar.groups (id, session_id, name, group_type)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(name)
        .bind(group_type)
        .execute(self.pool.pool())
        .await?;

        Ok(id)
    }

    /// Add a system to a group
    pub async fn add_system_to_group(
        &self,
        group_id: Uuid,
        system_id: Uuid,
        role: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stellar.group_members (group_id, system_id, member_role)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(group_id)
        .bind(system_id)
        .bind(role)
        .execute(self.pool.pool())
        .await?;

        Ok(())
    }
}