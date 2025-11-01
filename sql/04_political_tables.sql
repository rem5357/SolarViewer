-- Political entities and influence zones with spatial support
-- Stores governments, territories, and spheres of influence

SET search_path TO stellar, political, routing, public;

-- Political entities (nations, empires, federations, etc.)
CREATE TABLE IF NOT EXISTS political.entities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    short_name VARCHAR(50),
    government_type political.government_type NOT NULL,

    -- Capital and founding
    capital_system_id UUID REFERENCES stellar.star_systems(id),
    founded_date DATE,
    dissolved_date DATE,

    -- Political properties
    population_total BIGINT,
    gdp_credits NUMERIC(20, 2),
    military_strength INTEGER,  -- Abstract strength value 0-100
    tech_level INTEGER,          -- Tech level 1-10
    stability_index NUMERIC(3, 2) DEFAULT 0.5, -- 0.0 to 1.0

    -- Visual properties for rendering
    primary_color_rgb INTEGER[3],
    secondary_color_rgb INTEGER[3],
    flag_symbol VARCHAR(50),
    influence_opacity NUMERIC(3, 2) DEFAULT 0.3, -- Transparency for influence zones

    -- Metadata
    description TEXT,
    ideology TEXT,
    leader_name VARCHAR(255),
    leader_title VARCHAR(100),
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_entity_name_per_session UNIQUE(session_id, name)
);

CREATE INDEX idx_political_entities_session ON political.entities(session_id);
CREATE INDEX idx_political_entities_capital ON political.entities(capital_system_id);
CREATE INDEX idx_political_entities_tags ON political.entities USING gin(tags);

-- System membership and control
CREATE TABLE IF NOT EXISTS political.system_membership (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    political_entity_id UUID NOT NULL REFERENCES political.entities(id) ON DELETE CASCADE,

    -- Control level
    control_type VARCHAR(50) NOT NULL DEFAULT 'controlled',  -- 'controlled', 'disputed', 'claimed', 'influenced'
    control_strength NUMERIC(3, 2) DEFAULT 1.0,  -- 0.0 to 1.0, affects influence radius

    -- Dates
    acquired_date DATE,
    lost_date DATE,

    -- Economic importance
    economic_value INTEGER DEFAULT 0,
    strategic_value INTEGER DEFAULT 0,

    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT unique_system_control UNIQUE(session_id, system_id, political_entity_id)
);

CREATE INDEX idx_system_membership_session ON political.system_membership(session_id);
CREATE INDEX idx_system_membership_system ON political.system_membership(system_id);
CREATE INDEX idx_system_membership_entity ON political.system_membership(political_entity_id);

-- Political influence zones (spatial representation)
CREATE TABLE IF NOT EXISTS political.influence_zones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    political_entity_id UUID NOT NULL REFERENCES political.entities(id) ON DELETE CASCADE,

    -- Influence zone geometry
    zone_geometry GEOMETRY(MultiPolygonZ, 4326) NOT NULL,  -- 3D influence zone
    zone_type VARCHAR(50) NOT NULL DEFAULT 'territory',     -- 'territory', 'influence', 'claimed', 'disputed'

    -- Influence properties
    base_radius_ly DOUBLE PRECISION NOT NULL DEFAULT 10.0,  -- Base influence radius in light-years
    strength_multiplier NUMERIC(3, 2) DEFAULT 1.0,         -- Multiplier for influence strength

    -- Visual properties
    render_layer INTEGER DEFAULT 0,  -- Rendering order
    opacity_override NUMERIC(3, 2),  -- Override entity default opacity

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_influence_zones_session ON political.influence_zones(session_id);
CREATE INDEX idx_influence_zones_entity ON political.influence_zones(political_entity_id);
CREATE INDEX idx_influence_zones_geometry ON political.influence_zones USING gist(zone_geometry);

-- Diplomatic relations between entities
CREATE TABLE IF NOT EXISTS political.diplomatic_relations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    from_entity_id UUID NOT NULL REFERENCES political.entities(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES political.entities(id) ON DELETE CASCADE,

    relation_type VARCHAR(50) NOT NULL,  -- 'allied', 'friendly', 'neutral', 'hostile', 'war', 'trade_partner', etc.
    relation_value INTEGER DEFAULT 0,     -- -100 to 100

    -- Treaties and agreements
    treaty_name VARCHAR(255),
    treaty_signed_date DATE,
    treaty_expires_date DATE,

    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT unique_diplomatic_relation UNIQUE(session_id, from_entity_id, to_entity_id),
    CONSTRAINT check_different_entities CHECK(from_entity_id != to_entity_id)
);

CREATE INDEX idx_diplomatic_relations_session ON political.diplomatic_relations(session_id);
CREATE INDEX idx_diplomatic_relations_from ON political.diplomatic_relations(from_entity_id);
CREATE INDEX idx_diplomatic_relations_to ON political.diplomatic_relations(to_entity_id);

-- Borders between political entities
CREATE TABLE IF NOT EXISTS political.borders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    entity_a_id UUID NOT NULL REFERENCES political.entities(id) ON DELETE CASCADE,
    entity_b_id UUID NOT NULL REFERENCES political.entities(id) ON DELETE CASCADE,

    -- Border geometry
    border_line GEOMETRY(MultiLineStringZ, 4326),
    disputed_area GEOMETRY(MultiPolygonZ, 4326),

    -- Border status
    border_status VARCHAR(50) DEFAULT 'recognized',  -- 'recognized', 'disputed', 'closed', 'open'
    tension_level INTEGER DEFAULT 0,                 -- 0-100

    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT unique_border UNIQUE(session_id, entity_a_id, entity_b_id),
    CONSTRAINT check_border_entities CHECK(entity_a_id < entity_b_id)  -- Ensure consistent ordering
);

CREATE INDEX idx_borders_session ON political.borders(session_id);
CREATE INDEX idx_borders_entities ON political.borders(entity_a_id, entity_b_id);
CREATE INDEX idx_borders_geometry ON political.borders USING gist(border_line);

-- Trigger for modified_at
CREATE TRIGGER update_entities_modified
    BEFORE UPDATE ON political.entities
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

CREATE TRIGGER update_influence_zones_modified
    BEFORE UPDATE ON political.influence_zones
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

-- Function to generate influence zones from controlled systems
CREATE OR REPLACE FUNCTION political.generate_influence_zone(
    p_session_id UUID,
    p_entity_id UUID,
    p_base_radius_ly DOUBLE PRECISION DEFAULT 10.0
)
RETURNS GEOMETRY AS $$
DECLARE
    v_zone GEOMETRY;
    v_radius_meters DOUBLE PRECISION;
    v_system_zones GEOMETRY[];
BEGIN
    -- Convert light-years to meters
    v_radius_meters := p_base_radius_ly * 9.461e15;

    -- Create influence spheres around each controlled system
    SELECT ARRAY_AGG(
        ST_Buffer(
            s.position,
            v_radius_meters * COALESCE(m.control_strength, 1.0)
        )
    )
    INTO v_system_zones
    FROM political.system_membership m
    JOIN stellar.star_systems s ON s.id = m.system_id
    WHERE m.session_id = p_session_id
    AND m.political_entity_id = p_entity_id
    AND m.control_type IN ('controlled', 'claimed');

    -- Union all influence spheres
    IF array_length(v_system_zones, 1) > 0 THEN
        v_zone := ST_Union(v_system_zones);
    END IF;

    RETURN v_zone;
END;
$$ LANGUAGE plpgsql;

-- Function to find overlapping influence zones (disputed territories)
CREATE OR REPLACE FUNCTION political.find_disputed_zones(
    p_session_id UUID
)
RETURNS TABLE(
    entity_a_id UUID,
    entity_a_name VARCHAR(255),
    entity_b_id UUID,
    entity_b_name VARCHAR(255),
    disputed_area GEOMETRY
) AS $$
BEGIN
    RETURN QUERY
    WITH zone_pairs AS (
        SELECT
            z1.political_entity_id AS entity_a,
            z2.political_entity_id AS entity_b,
            ST_Intersection(z1.zone_geometry, z2.zone_geometry) AS overlap
        FROM political.influence_zones z1
        JOIN political.influence_zones z2
            ON z1.session_id = z2.session_id
            AND z1.political_entity_id < z2.political_entity_id
            AND ST_Intersects(z1.zone_geometry, z2.zone_geometry)
        WHERE z1.session_id = p_session_id
    )
    SELECT
        zp.entity_a,
        e1.name,
        zp.entity_b,
        e2.name,
        zp.overlap
    FROM zone_pairs zp
    JOIN political.entities e1 ON e1.id = zp.entity_a
    JOIN political.entities e2 ON e2.id = zp.entity_b
    WHERE ST_Area(zp.overlap) > 0;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate political control strength at a point
CREATE OR REPLACE FUNCTION political.get_control_at_point(
    p_session_id UUID,
    p_point GEOMETRY
)
RETURNS TABLE(
    entity_id UUID,
    entity_name VARCHAR(255),
    control_strength NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        e.id,
        e.name,
        CASE
            -- Stronger influence closer to controlled systems
            WHEN MIN(ST_3DDistance(p_point, s.position)) < 5 * 9.461e15 THEN 1.0
            WHEN MIN(ST_3DDistance(p_point, s.position)) < 10 * 9.461e15 THEN 0.7
            WHEN MIN(ST_3DDistance(p_point, s.position)) < 20 * 9.461e15 THEN 0.4
            ELSE 0.1
        END::NUMERIC AS strength
    FROM political.entities e
    JOIN political.system_membership m ON m.political_entity_id = e.id
    JOIN stellar.star_systems s ON s.id = m.system_id
    WHERE e.session_id = p_session_id
    AND ST_Contains(
        (SELECT zone_geometry FROM political.influence_zones
         WHERE political_entity_id = e.id AND session_id = p_session_id),
        p_point
    )
    GROUP BY e.id, e.name
    ORDER BY strength DESC;
END;
$$ LANGUAGE plpgsql;