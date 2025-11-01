-- Groups, sectors, and organizational structures
-- For organizing systems into logical groups and regions

SET search_path TO stellar, political, routing, public;

-- Sectors (spatial divisions of space)
CREATE TABLE IF NOT EXISTS stellar.sectors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    sector_code VARCHAR(20),  -- e.g., "Alpha-7", "Rim-3"

    -- Spatial bounds
    bounds GEOMETRY(PolygonZ, 4326) NOT NULL,
    center_point GEOMETRY(PointZ, 4326),
    volume_ly3 DOUBLE PRECISION,  -- Volume in cubic light-years

    -- Sector properties
    sector_type VARCHAR(50) DEFAULT 'standard',  -- 'core', 'inner', 'mid', 'outer', 'rim', 'frontier'
    security_level INTEGER DEFAULT 50,           -- 0-100 (0=lawless, 100=highly secure)
    development_level INTEGER DEFAULT 50,        -- 0-100 (0=unexplored, 100=fully developed)

    -- Administrative
    administrated_by UUID REFERENCES political.entities(id),

    -- Statistics
    system_count INTEGER DEFAULT 0,
    population_total BIGINT DEFAULT 0,

    -- Visual properties
    color_rgb INTEGER[3],
    border_color_rgb INTEGER[3],
    opacity NUMERIC(3, 2) DEFAULT 0.1,

    -- Metadata
    description TEXT,
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_sector_name_per_session UNIQUE(session_id, name)
);

CREATE INDEX idx_sectors_session ON stellar.sectors(session_id);
CREATE INDEX idx_sectors_bounds ON stellar.sectors USING gist(bounds);
CREATE INDEX idx_sectors_center ON stellar.sectors USING gist(center_point);
CREATE INDEX idx_sectors_admin ON stellar.sectors(administrated_by);
CREATE INDEX idx_sectors_tags ON stellar.sectors USING gin(tags);

-- Groups (logical collections of systems)
CREATE TABLE IF NOT EXISTS stellar.groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    group_type VARCHAR(50) NOT NULL,  -- 'cluster', 'constellation', 'trade_network', 'exploration_target', 'custom'

    -- Group properties
    parent_group_id UUID REFERENCES stellar.groups(id),  -- For hierarchical groups

    -- Visual properties
    show_boundary BOOLEAN DEFAULT FALSE,
    boundary_color_rgb INTEGER[3],
    highlight_members BOOLEAN DEFAULT FALSE,
    member_color_rgb INTEGER[3],

    -- Metadata
    description TEXT,
    purpose TEXT,
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_group_name_per_session UNIQUE(session_id, name)
);

CREATE INDEX idx_groups_session ON stellar.groups(session_id);
CREATE INDEX idx_groups_parent ON stellar.groups(parent_group_id);
CREATE INDEX idx_groups_type ON stellar.groups(group_type);
CREATE INDEX idx_groups_tags ON stellar.groups USING gin(tags);

-- Group membership
CREATE TABLE IF NOT EXISTS stellar.group_members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES stellar.groups(id) ON DELETE CASCADE,
    system_id UUID REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    body_id UUID REFERENCES stellar.bodies(id) ON DELETE CASCADE,

    -- Member properties
    member_role VARCHAR(50),  -- 'core', 'associate', 'candidate', etc.
    joined_date DATE DEFAULT CURRENT_DATE,

    -- Metadata
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT check_member_type CHECK(
        (system_id IS NOT NULL AND body_id IS NULL) OR
        (system_id IS NULL AND body_id IS NOT NULL)
    ),
    CONSTRAINT unique_system_member UNIQUE(group_id, system_id),
    CONSTRAINT unique_body_member UNIQUE(group_id, body_id)
);

CREATE INDEX idx_group_members_group ON stellar.group_members(group_id);
CREATE INDEX idx_group_members_system ON stellar.group_members(system_id);
CREATE INDEX idx_group_members_body ON stellar.group_members(body_id);

-- Regions (named areas that can span multiple sectors)
CREATE TABLE IF NOT EXISTS stellar.regions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    region_type VARCHAR(50) DEFAULT 'geographic',  -- 'geographic', 'economic', 'cultural', 'military', 'custom'

    -- Spatial definition
    boundary GEOMETRY(MultiPolygonZ, 4326),

    -- Region properties
    parent_region_id UUID REFERENCES stellar.regions(id),
    importance_level INTEGER DEFAULT 50,  -- 0-100

    -- Visual
    fill_color_rgb INTEGER[3],
    border_color_rgb INTEGER[3],
    border_width NUMERIC(3, 1) DEFAULT 1.0,
    opacity NUMERIC(3, 2) DEFAULT 0.2,
    label_position GEOMETRY(PointZ, 4326),

    -- Metadata
    description TEXT,
    historical_notes TEXT,
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_region_name_per_session UNIQUE(session_id, name)
);

CREATE INDEX idx_regions_session ON stellar.regions(session_id);
CREATE INDEX idx_regions_boundary ON stellar.regions USING gist(boundary);
CREATE INDEX idx_regions_parent ON stellar.regions(parent_region_id);
CREATE INDEX idx_regions_tags ON stellar.regions USING gin(tags);

-- Associations between different entities
CREATE TABLE IF NOT EXISTS stellar.associations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,

    -- Association endpoints (polymorphic references)
    from_entity_type VARCHAR(50) NOT NULL,  -- 'system', 'body', 'entity', 'group', 'sector'
    from_entity_id UUID NOT NULL,
    to_entity_type VARCHAR(50) NOT NULL,
    to_entity_id UUID NOT NULL,

    -- Association properties
    association_type VARCHAR(100) NOT NULL,  -- 'allied_with', 'trades_with', 'at_war_with', etc.
    association_strength NUMERIC(3, 2) DEFAULT 0.5,  -- 0.0 to 1.0

    -- Temporal
    start_date DATE,
    end_date DATE,

    -- Metadata
    description TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_association UNIQUE(
        session_id,
        from_entity_type,
        from_entity_id,
        to_entity_type,
        to_entity_id,
        association_type
    )
);

CREATE INDEX idx_associations_session ON stellar.associations(session_id);
CREATE INDEX idx_associations_from ON stellar.associations(from_entity_type, from_entity_id);
CREATE INDEX idx_associations_to ON stellar.associations(to_entity_type, to_entity_id);
CREATE INDEX idx_associations_type ON stellar.associations(association_type);

-- Triggers
CREATE TRIGGER update_sectors_modified
    BEFORE UPDATE ON stellar.sectors
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_groups_modified
    BEFORE UPDATE ON stellar.groups
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_regions_modified
    BEFORE UPDATE ON stellar.regions
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

-- Function to create sectors by dividing space into a grid
CREATE OR REPLACE FUNCTION stellar.create_sector_grid(
    p_session_id UUID,
    p_min_x DOUBLE PRECISION,
    p_max_x DOUBLE PRECISION,
    p_min_y DOUBLE PRECISION,
    p_max_y DOUBLE PRECISION,
    p_min_z DOUBLE PRECISION,
    p_max_z DOUBLE PRECISION,
    p_sectors_per_axis INTEGER DEFAULT 10
)
RETURNS INTEGER AS $$
DECLARE
    v_sector_count INTEGER := 0;
    v_x_step DOUBLE PRECISION;
    v_y_step DOUBLE PRECISION;
    v_z_step DOUBLE PRECISION;
    v_x DOUBLE PRECISION;
    v_y DOUBLE PRECISION;
    v_z DOUBLE PRECISION;
    v_bounds GEOMETRY;
    v_center GEOMETRY;
    v_sector_name VARCHAR(255);
BEGIN
    -- Calculate step sizes
    v_x_step := (p_max_x - p_min_x) / p_sectors_per_axis;
    v_y_step := (p_max_y - p_min_y) / p_sectors_per_axis;
    v_z_step := (p_max_z - p_min_z) / p_sectors_per_axis;

    -- Create sectors
    FOR i IN 0..(p_sectors_per_axis - 1) LOOP
        FOR j IN 0..(p_sectors_per_axis - 1) LOOP
            FOR k IN 0..(p_sectors_per_axis - 1) LOOP
                v_x := p_min_x + (i * v_x_step);
                v_y := p_min_y + (j * v_y_step);
                v_z := p_min_z + (k * v_z_step);

                -- Create 3D box geometry
                v_bounds := ST_MakeEnvelope(
                    v_x * 9.461e15,
                    v_y * 9.461e15,
                    (v_x + v_x_step) * 9.461e15,
                    (v_y + v_y_step) * 9.461e15,
                    4326
                );

                -- Calculate center point
                v_center := ST_MakePoint(
                    (v_x + v_x_step/2) * 9.461e15,
                    (v_y + v_y_step/2) * 9.461e15,
                    (v_z + v_z_step/2) * 9.461e15
                );

                v_sector_name := format('Sector-%s-%s-%s',
                    chr(65 + i),  -- A, B, C, ...
                    j + 1,
                    k + 1
                );

                INSERT INTO stellar.sectors (
                    session_id,
                    name,
                    sector_code,
                    bounds,
                    center_point,
                    volume_ly3
                ) VALUES (
                    p_session_id,
                    v_sector_name,
                    format('%s%s%s', chr(65 + i), j + 1, k + 1),
                    v_bounds,
                    v_center,
                    v_x_step * v_y_step * v_z_step
                );

                v_sector_count := v_sector_count + 1;
            END LOOP;
        END LOOP;
    END LOOP;

    RETURN v_sector_count;
END;
$$ LANGUAGE plpgsql;

-- Function to assign systems to sectors
CREATE OR REPLACE FUNCTION stellar.assign_systems_to_sectors(
    p_session_id UUID
)
RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER := 0;
BEGIN
    -- Update sector statistics based on contained systems
    UPDATE stellar.sectors s
    SET
        system_count = (
            SELECT COUNT(*)
            FROM stellar.star_systems sys
            WHERE sys.session_id = p_session_id
            AND ST_Contains(s.bounds, sys.position)
        ),
        population_total = (
            SELECT COALESCE(SUM((b.physical_properties->>'population')::BIGINT), 0)
            FROM stellar.star_systems sys
            JOIN stellar.bodies b ON b.system_id = sys.id
            WHERE sys.session_id = p_session_id
            AND ST_Contains(s.bounds, sys.position)
        )
    WHERE s.session_id = p_session_id;

    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- Function to create a convex hull boundary around a group of systems
CREATE OR REPLACE FUNCTION stellar.create_group_boundary(
    p_group_id UUID
)
RETURNS GEOMETRY AS $$
DECLARE
    v_boundary GEOMETRY;
BEGIN
    SELECT ST_ConvexHull(ST_Collect(s.position))
    INTO v_boundary
    FROM stellar.group_members gm
    JOIN stellar.star_systems s ON s.id = gm.system_id
    WHERE gm.group_id = p_group_id;

    RETURN v_boundary;
END;
$$ LANGUAGE plpgsql;