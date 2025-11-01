-- Stellar objects tables with PostGIS spatial support
-- Stores stars, planets, and other celestial bodies

SET search_path TO stellar, political, routing, public;

-- Coordinate frames for reference
CREATE TABLE IF NOT EXISTS stellar.coordinate_frames (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    frame_type stellar.coordinate_system NOT NULL DEFAULT 'galactic_iau',
    parent_frame_id UUID REFERENCES stellar.coordinate_frames(id),

    -- Transformation to parent
    origin_position GEOMETRY(PointZ, 4326),
    orientation_quaternion DOUBLE PRECISION[4],

    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT unique_frame_per_session UNIQUE(session_id, name)
);

CREATE INDEX idx_frames_session ON stellar.coordinate_frames(session_id);
CREATE INDEX idx_frames_parent ON stellar.coordinate_frames(parent_frame_id);

-- Star systems (containers for stellar objects)
CREATE TABLE IF NOT EXISTS stellar.star_systems (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    system_type stellar.system_type NOT NULL DEFAULT 'single',

    -- Spatial data (IAU Galactic coordinates)
    position GEOMETRY(PointZ, 4326) NOT NULL,  -- 3D position in space
    galactic_longitude DOUBLE PRECISION,        -- l in degrees
    galactic_latitude DOUBLE PRECISION,         -- b in degrees
    distance_from_sol_ly DOUBLE PRECISION,      -- Distance in light-years

    -- Legacy support for Astrosynthesis imports
    legacy_position DOUBLE PRECISION[3],        -- Original [x,y,z] if imported

    -- System properties
    barycenter GEOMETRY(PointZ, 4326),         -- Center of mass
    total_mass_solar DOUBLE PRECISION,         -- Total system mass in solar masses
    habitable_zone_inner_au DOUBLE PRECISION,
    habitable_zone_outer_au DOUBLE PRECISION,

    -- Discovery and naming
    discovered_date DATE,
    discovered_by VARCHAR(255),
    catalog_designation VARCHAR(100),

    -- Metadata
    notes TEXT,
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_system_name_per_session UNIQUE(session_id, name)
);

-- Spatial indexes for efficient queries
CREATE INDEX idx_systems_position ON stellar.star_systems USING gist(position);
CREATE INDEX idx_systems_session ON stellar.star_systems(session_id);
CREATE INDEX idx_systems_distance ON stellar.star_systems(distance_from_sol_ly);
CREATE INDEX idx_systems_tags ON stellar.star_systems USING gin(tags);

-- Individual celestial bodies (stars, planets, moons, etc.)
CREATE TABLE IF NOT EXISTS stellar.bodies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    system_id UUID REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    parent_body_id UUID REFERENCES stellar.bodies(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    body_kind stellar.body_kind NOT NULL,

    -- Position (relative to parent or system)
    position GEOMETRY(PointZ, 4326),           -- Absolute position
    relative_position DOUBLE PRECISION[3],      -- Position relative to parent

    -- Orbital parameters
    orbital_radius_au DOUBLE PRECISION,
    eccentricity DOUBLE PRECISION CHECK (eccentricity >= 0 AND eccentricity < 2),
    inclination_deg DOUBLE PRECISION,
    longitude_ascending_node_deg DOUBLE PRECISION,
    argument_periapsis_deg DOUBLE PRECISION,
    mean_anomaly_deg DOUBLE PRECISION,
    orbital_period_days DOUBLE PRECISION,

    -- Physical properties (stored as JSONB for flexibility)
    physical_properties JSONB DEFAULT '{}'::jsonb,
    /* Expected structure for different body types:
       Stars: {
         "spectral_type": "G2V",
         "mass_solar": 1.0,
         "radius_solar": 1.0,
         "luminosity_solar": 1.0,
         "temperature_k": 5778,
         "age_gyr": 4.6
       }
       Planets: {
         "mass_earth": 1.0,
         "radius_earth": 1.0,
         "density_gcc": 5.5,
         "gravity_g": 1.0,
         "atmosphere": {...},
         "water_percent": 71,
         "population": 7800000000,
         "habitability": 1.0
       }
    */

    -- Rotation
    rotation_period_hours DOUBLE PRECISION,
    axial_tilt_deg DOUBLE PRECISION,
    retrograde_rotation BOOLEAN DEFAULT FALSE,

    -- Discovery and classification
    discovered_date DATE,
    discovered_by VARCHAR(255),
    catalog_designation VARCHAR(100),

    -- Visual properties
    visible BOOLEAN DEFAULT TRUE,
    color_rgb INTEGER[3],
    render_distance_ly DOUBLE PRECISION,

    -- Metadata
    notes TEXT,
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bodies_session ON stellar.bodies(session_id);
CREATE INDEX idx_bodies_system ON stellar.bodies(system_id);
CREATE INDEX idx_bodies_parent ON stellar.bodies(parent_body_id);
CREATE INDEX idx_bodies_position ON stellar.bodies USING gist(position);
CREATE INDEX idx_bodies_kind ON stellar.bodies(body_kind);
CREATE INDEX idx_bodies_tags ON stellar.bodies USING gin(tags);
CREATE INDEX idx_bodies_physical ON stellar.bodies USING gin(physical_properties);

-- Create trigger for modified_at
CREATE TRIGGER update_bodies_modified
    BEFORE UPDATE ON stellar.bodies
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_systems_modified
    BEFORE UPDATE ON stellar.star_systems
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

-- Surface features and points of interest
CREATE TABLE IF NOT EXISTS stellar.surface_features (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    body_id UUID NOT NULL REFERENCES stellar.bodies(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    feature_type VARCHAR(50), -- 'city', 'spaceport', 'ruins', 'mining_site', etc.

    -- Location on body
    latitude_deg DOUBLE PRECISION,
    longitude_deg DOUBLE PRECISION,
    elevation_m DOUBLE PRECISION,

    -- Properties
    population BIGINT,
    importance_level INTEGER DEFAULT 0,

    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT check_coordinates CHECK (
        latitude_deg >= -90 AND latitude_deg <= 90 AND
        longitude_deg >= -180 AND longitude_deg <= 180
    )
);

CREATE INDEX idx_surface_features_body ON stellar.surface_features(body_id);

-- Asteroid belts and fields (regions containing many small bodies)
CREATE TABLE IF NOT EXISTS stellar.asteroid_fields (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,

    -- Orbital boundaries
    inner_radius_au DOUBLE PRECISION NOT NULL,
    outer_radius_au DOUBLE PRECISION NOT NULL,
    thickness_au DOUBLE PRECISION,

    -- Properties
    total_mass_earth DOUBLE PRECISION,
    average_density_gcc DOUBLE PRECISION,
    particle_count_estimate BIGINT,
    composition VARCHAR(255),

    -- Notable asteroids are stored as individual bodies

    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_asteroid_fields_system ON stellar.asteroid_fields(system_id);

-- Helper functions for spatial queries

-- Find all systems within radius of a point
CREATE OR REPLACE FUNCTION stellar.find_systems_within_radius(
    p_session_id UUID,
    p_center GEOMETRY,
    p_radius_ly DOUBLE PRECISION
)
RETURNS TABLE(
    system_id UUID,
    system_name VARCHAR(255),
    distance_ly DOUBLE PRECISION,
    position GEOMETRY
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id,
        s.name,
        ST_3DDistance(s.position, p_center) / 9.461e15 AS distance_ly,  -- Convert meters to ly
        s.position
    FROM stellar.star_systems s
    WHERE s.session_id = p_session_id
    AND ST_3DDWithin(s.position, p_center, p_radius_ly * 9.461e15)  -- Convert ly to meters
    ORDER BY distance_ly;
END;
$$ LANGUAGE plpgsql;

-- Find nearest neighbors to a system
CREATE OR REPLACE FUNCTION stellar.find_nearest_systems(
    p_session_id UUID,
    p_system_id UUID,
    p_limit INTEGER DEFAULT 10
)
RETURNS TABLE(
    system_id UUID,
    system_name VARCHAR(255),
    distance_ly DOUBLE PRECISION
) AS $$
DECLARE
    v_position GEOMETRY;
BEGIN
    SELECT position INTO v_position
    FROM stellar.star_systems
    WHERE id = p_system_id;

    RETURN QUERY
    SELECT
        s.id,
        s.name,
        ST_3DDistance(s.position, v_position) / 9.461e15 AS distance_ly
    FROM stellar.star_systems s
    WHERE s.session_id = p_session_id
    AND s.id != p_system_id
    ORDER BY s.position <-> v_position  -- KNN operator
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;