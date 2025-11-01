-- Session and save management tables
-- These track different saved galaxies and subsections

SET search_path TO stellar, political, routing, public;

-- Sessions represent saved galaxy states or subsections
CREATE TABLE IF NOT EXISTS stellar.sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    session_type VARCHAR(50) NOT NULL CHECK (session_type IN ('full_galaxy', 'subsection', 'import')),
    parent_session_id UUID REFERENCES stellar.sessions(id) ON DELETE CASCADE,

    -- Session metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),

    -- Spatial bounds (for subsections)
    bounds_center GEOMETRY(PointZ, 4326),  -- Center point for subsections
    bounds_radius_ly DOUBLE PRECISION,      -- Radius in light-years for subsections
    bounds_polygon GEOMETRY(PolygonZ, 4326), -- Custom polygon bounds

    -- Statistics
    total_systems INTEGER DEFAULT 0,
    total_stars INTEGER DEFAULT 0,
    total_planets INTEGER DEFAULT 0,
    total_populated_worlds INTEGER DEFAULT 0,
    total_routes INTEGER DEFAULT 0,
    total_political_entities INTEGER DEFAULT 0,

    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    tags TEXT[] DEFAULT '{}',

    -- Search
    search_vector tsvector GENERATED ALWAYS AS (
        to_tsvector('english', COALESCE(name, '') || ' ' || COALESCE(description, ''))
    ) STORED
);

-- Index for full-text search
CREATE INDEX idx_sessions_search ON stellar.sessions USING gin(search_vector);
CREATE INDEX idx_sessions_created_at ON stellar.sessions(created_at DESC);
CREATE INDEX idx_sessions_parent ON stellar.sessions(parent_session_id);
CREATE INDEX idx_sessions_bounds_center ON stellar.sessions USING gist(bounds_center);
CREATE INDEX idx_sessions_tags ON stellar.sessions USING gin(tags);

-- Session history for tracking changes
CREATE TABLE IF NOT EXISTS stellar.session_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,  -- 'created', 'modified', 'exported', 'imported', etc.
    description TEXT,
    changed_by VARCHAR(255),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changes JSONB  -- Detailed change log
);

CREATE INDEX idx_session_history_session ON stellar.session_history(session_id);
CREATE INDEX idx_session_history_changed_at ON stellar.session_history(changed_at DESC);

-- Session relationships (for linking related saves)
CREATE TABLE IF NOT EXISTS stellar.session_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    from_session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    to_session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) NOT NULL,  -- 'derived_from', 'merged_with', 'split_from', etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT unique_session_relationship UNIQUE(from_session_id, to_session_id, relationship_type)
);

CREATE INDEX idx_session_rel_from ON stellar.session_relationships(from_session_id);
CREATE INDEX idx_session_rel_to ON stellar.session_relationships(to_session_id);

-- Trigger to update modified_at
CREATE OR REPLACE FUNCTION stellar.update_modified_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_sessions_modified
    BEFORE UPDATE ON stellar.sessions
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

-- Function to create a subsection from an existing session
CREATE OR REPLACE FUNCTION stellar.create_subsection(
    p_parent_session_id UUID,
    p_name VARCHAR(255),
    p_center_point GEOMETRY,
    p_radius_ly DOUBLE PRECISION,
    p_description TEXT DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_new_session_id UUID;
    v_radius_meters DOUBLE PRECISION;
BEGIN
    -- Convert light-years to meters for spatial calculations
    v_radius_meters := p_radius_ly * 9.461e15;

    -- Create the new subsection session
    INSERT INTO stellar.sessions (
        name,
        description,
        session_type,
        parent_session_id,
        bounds_center,
        bounds_radius_ly
    ) VALUES (
        p_name,
        p_description,
        'subsection',
        p_parent_session_id,
        p_center_point,
        p_radius_ly
    ) RETURNING id INTO v_new_session_id;

    -- Log the creation
    INSERT INTO stellar.session_history (
        session_id,
        action,
        description
    ) VALUES (
        v_new_session_id,
        'created',
        'Subsection created from parent session'
    );

    RETURN v_new_session_id;
END;
$$ LANGUAGE plpgsql;

-- Function to update session statistics
CREATE OR REPLACE FUNCTION stellar.update_session_stats(p_session_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE stellar.sessions
    SET
        total_systems = (
            SELECT COUNT(*) FROM stellar.star_systems
            WHERE session_id = p_session_id
        ),
        total_stars = (
            SELECT COUNT(*) FROM stellar.bodies
            WHERE session_id = p_session_id
            AND body_kind = 'star'
        ),
        total_planets = (
            SELECT COUNT(*) FROM stellar.bodies
            WHERE session_id = p_session_id
            AND body_kind IN ('planet', 'rogue_planet')
        ),
        total_populated_worlds = (
            SELECT COUNT(*) FROM stellar.bodies
            WHERE session_id = p_session_id
            AND body_kind IN ('planet', 'moon')
            AND (physical_properties->>'population')::NUMERIC > 0
        ),
        total_routes = (
            SELECT COUNT(*) FROM routing.routes
            WHERE session_id = p_session_id
        ),
        total_political_entities = (
            SELECT COUNT(DISTINCT political_entity_id)
            FROM political.system_membership
            WHERE session_id = p_session_id
        )
    WHERE id = p_session_id;
END;
$$ LANGUAGE plpgsql;