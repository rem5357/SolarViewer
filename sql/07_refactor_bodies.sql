-- StellarForge Schema Refactor: Proper Body Type Tables
-- This replaces the generic bodies table with specific tables for each astronomical body type

-- Drop old bodies table (after backup!)
-- DROP TABLE IF EXISTS stellar.bodies CASCADE;

-- =============================================================================
-- STARS - Primary gravitational centers
-- =============================================================================
CREATE TABLE IF NOT EXISTS stellar.stars (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    parent_star_id UUID REFERENCES stellar.stars(id) ON DELETE CASCADE, -- For hierarchical binaries

    -- Identity
    name VARCHAR(255) NOT NULL,
    catalog_designations TEXT[], -- HD 209458, HIP 108859, etc.

    -- Position (relative to system barycenter or parent star)
    position_relative GEOMETRY(PointZ, 4326), -- Relative position in AU
    position_absolute GEOMETRY(PointZ, 4326), -- Absolute galactic position (computed)

    -- Orbital elements (if orbiting another star or barycenter)
    semi_major_axis_au NUMERIC, -- Distance from barycenter/parent
    eccentricity NUMERIC,
    inclination_deg NUMERIC,
    longitude_ascending_node_deg NUMERIC,
    argument_periapsis_deg NUMERIC,
    mean_anomaly_deg NUMERIC,
    orbital_period_days NUMERIC,

    -- Physical properties
    spectral_class VARCHAR(20) NOT NULL, -- G2V, M4V, K0III, etc.
    luminosity_class VARCHAR(10), -- V (main sequence), III (giant), Ia (supergiant)
    mass_solar NUMERIC NOT NULL CHECK (mass_solar > 0),
    radius_solar NUMERIC CHECK (radius_solar > 0),
    luminosity_solar NUMERIC NOT NULL CHECK (luminosity_solar > 0),
    temperature_k NUMERIC NOT NULL CHECK (temperature_k > 0),

    -- Stellar evolution
    age_gyr NUMERIC CHECK (age_gyr >= 0),
    metallicity NUMERIC, -- [Fe/H]
    rotation_period_days NUMERIC,

    -- Magnetic/activity properties
    magnetic_field_gauss NUMERIC,
    stellar_activity_level VARCHAR(20), -- quiet, moderate, active, very_active

    -- Metadata
    discovered_date DATE,
    discovered_by VARCHAR(255),
    visible BOOLEAN DEFAULT true,
    notes TEXT,
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT now(),
    modified_at TIMESTAMPTZ DEFAULT now(),

    CONSTRAINT valid_eccentricity CHECK (eccentricity IS NULL OR (eccentricity >= 0 AND eccentricity < 1)),
    CONSTRAINT valid_inclination CHECK (inclination_deg IS NULL OR (inclination_deg >= 0 AND inclination_deg <= 180))
);

CREATE INDEX idx_stars_session ON stellar.stars(session_id);
CREATE INDEX idx_stars_system ON stellar.stars(system_id);
CREATE INDEX idx_stars_parent ON stellar.stars(parent_star_id);
CREATE INDEX idx_stars_position_rel ON stellar.stars USING GIST(position_relative);
CREATE INDEX idx_stars_position_abs ON stellar.stars USING GIST(position_absolute);
CREATE INDEX idx_stars_spectral ON stellar.stars(spectral_class);
CREATE INDEX idx_stars_mass ON stellar.stars(mass_solar);

-- =============================================================================
-- PLANETS - Bodies orbiting stars
-- =============================================================================
CREATE TABLE IF NOT EXISTS stellar.planets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    parent_star_id UUID NOT NULL REFERENCES stellar.stars(id) ON DELETE CASCADE,

    -- Identity
    name VARCHAR(255) NOT NULL,
    designation VARCHAR(100), -- "b", "c", "d" for exoplanets (Kepler-16b)

    -- Classification
    planet_type VARCHAR(50) NOT NULL, -- terrestrial, gas_giant, ice_giant, super_earth, sub_neptune

    -- Orbital elements (required - planets must orbit)
    semi_major_axis_au NUMERIC NOT NULL CHECK (semi_major_axis_au > 0),
    eccentricity NUMERIC NOT NULL CHECK (eccentricity >= 0 AND eccentricity < 1),
    inclination_deg NUMERIC,
    longitude_ascending_node_deg NUMERIC,
    argument_periapsis_deg NUMERIC,
    mean_anomaly_deg NUMERIC,
    orbital_period_days NUMERIC NOT NULL CHECK (orbital_period_days > 0),

    -- Physical properties
    mass_earth NUMERIC CHECK (mass_earth > 0),
    mass_jupiter NUMERIC CHECK (mass_jupiter > 0),
    radius_earth NUMERIC NOT NULL CHECK (radius_earth > 0),
    radius_jupiter NUMERIC CHECK (radius_jupiter > 0),
    density_gcc NUMERIC CHECK (density_gcc > 0),
    surface_gravity_g NUMERIC CHECK (surface_gravity_g > 0),
    escape_velocity_kms NUMERIC CHECK (escape_velocity_kms > 0),

    -- Temperature & habitability
    equilibrium_temperature_k NUMERIC,
    surface_temperature_k NUMERIC,
    in_habitable_zone BOOLEAN DEFAULT false,
    habitability_score NUMERIC CHECK (habitability_score >= 0 AND habitability_score <= 1),

    -- Atmosphere
    has_atmosphere BOOLEAN DEFAULT false,
    atmosphere_type VARCHAR(50), -- none, thin, dense, toxic, breathable
    atmosphere_composition JSONB, -- {"N2": 78, "O2": 21, "Ar": 1, ...}
    atmosphere_pressure_atm NUMERIC CHECK (atmosphere_pressure_atm >= 0),
    greenhouse_effect_k NUMERIC,

    -- Surface properties
    surface_type VARCHAR(50), -- rocky, icy, oceanic, molten, gas
    surface_water_percent NUMERIC CHECK (surface_water_percent >= 0 AND surface_water_percent <= 100),
    axial_tilt_deg NUMERIC CHECK (axial_tilt_deg >= 0 AND axial_tilt_deg <= 180),
    rotation_period_hours NUMERIC,
    tidally_locked BOOLEAN DEFAULT false,

    -- Magnetic field
    has_magnetic_field BOOLEAN DEFAULT false,
    magnetic_field_strength NUMERIC,

    -- Rings
    has_rings BOOLEAN DEFAULT false,
    ring_inner_radius_rp NUMERIC, -- In planetary radii
    ring_outer_radius_rp NUMERIC,

    -- Population & development
    population BIGINT CHECK (population >= 0),
    development_level VARCHAR(50), -- uninhabited, outpost, colony, developed, homeworld

    -- Metadata
    discovered_date DATE,
    discovered_by VARCHAR(255),
    visible BOOLEAN DEFAULT true,
    color_rgb INTEGER[3],
    notes TEXT,
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT now(),
    modified_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_planets_session ON stellar.planets(session_id);
CREATE INDEX idx_planets_system ON stellar.planets(system_id);
CREATE INDEX idx_planets_star ON stellar.planets(parent_star_id);
CREATE INDEX idx_planets_type ON stellar.planets(planet_type);
CREATE INDEX idx_planets_habitable ON stellar.planets(in_habitable_zone) WHERE in_habitable_zone = true;
CREATE INDEX idx_planets_population ON stellar.planets(population) WHERE population > 0;

-- =============================================================================
-- MOONS - Bodies orbiting planets
-- =============================================================================
CREATE TABLE IF NOT EXISTS stellar.moons (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    parent_planet_id UUID NOT NULL REFERENCES stellar.planets(id) ON DELETE CASCADE,

    -- Identity
    name VARCHAR(255) NOT NULL,
    designation VARCHAR(100), -- Roman numerals, letters, or names

    -- Classification
    moon_type VARCHAR(50), -- regular, irregular, captured, trojan

    -- Orbital elements
    semi_major_axis_km NUMERIC NOT NULL CHECK (semi_major_axis_km > 0),
    eccentricity NUMERIC CHECK (eccentricity >= 0 AND eccentricity < 1),
    inclination_deg NUMERIC,
    orbital_period_days NUMERIC NOT NULL CHECK (orbital_period_days > 0),

    -- Physical properties
    mass_earth NUMERIC CHECK (mass_earth > 0),
    radius_km NUMERIC NOT NULL CHECK (radius_km > 0),
    density_gcc NUMERIC,
    surface_gravity_g NUMERIC,

    -- Properties
    tidally_locked BOOLEAN DEFAULT true,
    rotation_period_hours NUMERIC,
    surface_temperature_k NUMERIC,
    has_atmosphere BOOLEAN DEFAULT false,
    atmosphere_type VARCHAR(50),
    surface_type VARCHAR(50),

    -- Interesting features
    geological_activity BOOLEAN DEFAULT false,
    subsurface_ocean BOOLEAN DEFAULT false,
    potential_life BOOLEAN DEFAULT false,

    -- Metadata
    visible BOOLEAN DEFAULT true,
    color_rgb INTEGER[3],
    notes TEXT,
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT now(),
    modified_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_moons_session ON stellar.moons(session_id);
CREATE INDEX idx_moons_system ON stellar.moons(system_id);
CREATE INDEX idx_moons_planet ON stellar.moons(parent_planet_id);

-- =============================================================================
-- ORBITAL OBJECTS - Small bodies orbiting stars, planets, or moons
-- (Asteroids, Comets, Space Stations, Wrecks, Asteroid Belts)
-- =============================================================================
CREATE TABLE IF NOT EXISTS stellar.orbital_objects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,

    -- Parent (what it orbits)
    parent_star_id UUID REFERENCES stellar.stars(id) ON DELETE CASCADE,
    parent_planet_id UUID REFERENCES stellar.planets(id) ON DELETE CASCADE,
    parent_moon_id UUID REFERENCES stellar.moons(id) ON DELETE CASCADE,

    -- Identity
    name VARCHAR(255) NOT NULL,
    object_type VARCHAR(50) NOT NULL, -- asteroid, comet, asteroid_belt, station, wreck, artifact

    -- Orbital elements
    semi_major_axis_au NUMERIC CHECK (semi_major_axis_au > 0),
    eccentricity NUMERIC CHECK (eccentricity >= 0 AND eccentricity < 2), -- Comets can be > 1
    inclination_deg NUMERIC,
    orbital_period_days NUMERIC,

    -- Physical properties (varies by type)
    mass_kg NUMERIC,
    radius_m NUMERIC,

    -- Asteroid belt specific
    belt_inner_radius_au NUMERIC,
    belt_outer_radius_au NUMERIC,
    belt_thickness_au NUMERIC,
    asteroid_count BIGINT,

    -- Station/artificial object specific
    station_type VARCHAR(50), -- orbital, lagrange, deep_space
    population BIGINT,
    operational BOOLEAN,
    faction_id UUID, -- Could reference political.entities

    -- Metadata
    visible BOOLEAN DEFAULT true,
    notes TEXT,
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT now(),
    modified_at TIMESTAMPTZ DEFAULT now(),

    -- Ensure exactly one parent
    CONSTRAINT has_one_parent CHECK (
        (parent_star_id IS NOT NULL)::integer +
        (parent_planet_id IS NOT NULL)::integer +
        (parent_moon_id IS NOT NULL)::integer = 1
    )
);

CREATE INDEX idx_orbital_objects_session ON stellar.orbital_objects(session_id);
CREATE INDEX idx_orbital_objects_system ON stellar.orbital_objects(system_id);
CREATE INDEX idx_orbital_objects_star ON stellar.orbital_objects(parent_star_id);
CREATE INDEX idx_orbital_objects_planet ON stellar.orbital_objects(parent_planet_id);
CREATE INDEX idx_orbital_objects_type ON stellar.orbital_objects(object_type);

-- =============================================================================
-- INTERSTELLAR OBJECTS - Not gravitationally bound to any system
-- (Rogue planets, interstellar stations, nebulae, brown dwarfs)
-- =============================================================================
CREATE TABLE IF NOT EXISTS stellar.interstellar_objects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,

    -- Identity
    name VARCHAR(255) NOT NULL,
    object_type VARCHAR(50) NOT NULL, -- rogue_planet, brown_dwarf, nebula, station, artifact

    -- Absolute galactic position (not relative to anything)
    position_galactic GEOMETRY(PointZ, 4326) NOT NULL,

    -- Motion through space
    velocity_xyz NUMERIC[3], -- [vx, vy, vz] in km/s
    velocity_kms NUMERIC, -- Total velocity

    -- Physical properties
    mass_jupiter NUMERIC,
    radius_jupiter NUMERIC,
    temperature_k NUMERIC,

    -- Nebula specific
    nebula_type VARCHAR(50), -- emission, reflection, dark, planetary
    nebula_extent_ly NUMERIC, -- Diameter in light years

    -- Station specific
    station_type VARCHAR(50), -- deep_space, generation_ship
    population BIGINT,
    operational BOOLEAN,

    -- Metadata
    visible BOOLEAN DEFAULT true,
    notes TEXT,
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT now(),
    modified_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_interstellar_session ON stellar.interstellar_objects(session_id);
CREATE INDEX idx_interstellar_position ON stellar.interstellar_objects USING GIST(position_galactic);
CREATE INDEX idx_interstellar_type ON stellar.interstellar_objects(object_type);

-- =============================================================================
-- Update triggers
-- =============================================================================

CREATE TRIGGER update_stars_modified
    BEFORE UPDATE ON stellar.stars
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_planets_modified
    BEFORE UPDATE ON stellar.planets
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_moons_modified
    BEFORE UPDATE ON stellar.moons
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_orbital_objects_modified
    BEFORE UPDATE ON stellar.orbital_objects
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_interstellar_objects_modified
    BEFORE UPDATE ON stellar.interstellar_objects
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

-- =============================================================================
-- Helpful views
-- =============================================================================

-- All bodies in a system (union view)
CREATE OR REPLACE VIEW stellar.system_bodies_v AS
SELECT
    'star'::text as body_category,
    id, session_id, system_id, name,
    position_absolute as position,
    mass_solar * 1047.348644 as mass_jupiter, -- Convert to jupiter masses
    'spectral: ' || spectral_class as description
FROM stellar.stars
UNION ALL
SELECT
    'planet'::text,
    id, session_id, system_id, name,
    NULL as position, -- Would need to compute from orbit
    mass_jupiter,
    planet_type
FROM stellar.planets
UNION ALL
SELECT
    'moon'::text,
    id, session_id, system_id, name,
    NULL as position,
    mass_earth / 317.8 as mass_jupiter, -- Convert earth to jupiter
    moon_type
FROM stellar.moons;

-- Stars with their systems
CREATE OR REPLACE VIEW stellar.stars_with_systems_v AS
SELECT
    s.*,
    sys.name as system_name,
    sys.system_type,
    sys.position as system_position
FROM stellar.stars s
JOIN stellar.star_systems sys ON s.system_id = sys.id;

COMMENT ON TABLE stellar.stars IS 'Primary gravitational centers - stars that anchor solar systems';
COMMENT ON TABLE stellar.planets IS 'Planets orbiting stars';
COMMENT ON TABLE stellar.moons IS 'Natural satellites orbiting planets';
COMMENT ON TABLE stellar.orbital_objects IS 'Small bodies orbiting stars/planets/moons - asteroids, stations, belts';
COMMENT ON TABLE stellar.interstellar_objects IS 'Objects not bound to any system - rogue planets, nebulae, deep space stations';
