-- Routes, connections, and travel networks
-- Stores trade routes, jump lanes, and other connections between systems

SET search_path TO stellar, political, routing, public;

-- Route types and classifications
CREATE TABLE IF NOT EXISTS routing.route_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    default_color_rgb INTEGER[3],
    default_line_style VARCHAR(20) DEFAULT 'solid',  -- 'solid', 'dashed', 'dotted'
    default_line_width NUMERIC(3, 1) DEFAULT 1.0
);

-- Insert default route types
INSERT INTO routing.route_types (name, description, default_color_rgb, default_line_style) VALUES
    ('trade', 'Major trade route', '{0, 255, 0}', 'solid'),
    ('military', 'Military patrol route', '{255, 0, 0}', 'dashed'),
    ('exploration', 'Exploration path', '{0, 0, 255}', 'dotted'),
    ('migration', 'Population migration route', '{255, 255, 0}', 'solid'),
    ('communication', 'FTL communication relay', '{255, 128, 0}', 'dashed'),
    ('jump_lane', 'Established jump lane', '{128, 255, 255}', 'solid')
ON CONFLICT (name) DO NOTHING;

-- Main routes table
CREATE TABLE IF NOT EXISTS routing.routes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    route_type_id UUID REFERENCES routing.route_types(id),

    -- Route properties
    is_active BOOLEAN DEFAULT TRUE,
    is_bidirectional BOOLEAN DEFAULT TRUE,
    traffic_volume INTEGER DEFAULT 0,        -- Abstract traffic level 0-100
    danger_level INTEGER DEFAULT 0,          -- Danger/piracy level 0-100
    travel_time_days DOUBLE PRECISION,       -- Average travel time

    -- Economic properties
    trade_value_credits NUMERIC(20, 2),
    toll_cost_credits NUMERIC(10, 2),

    -- Political control
    controlled_by_entity_id UUID REFERENCES political.entities(id),

    -- Visual properties
    color_rgb INTEGER[3],
    line_style VARCHAR(20),
    line_width NUMERIC(3, 1),
    render_priority INTEGER DEFAULT 0,

    -- Metadata
    description TEXT,
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_route_name_per_session UNIQUE(session_id, name)
);

CREATE INDEX idx_routes_session ON routing.routes(session_id);
CREATE INDEX idx_routes_type ON routing.routes(route_type_id);
CREATE INDEX idx_routes_controlled_by ON routing.routes(controlled_by_entity_id);
CREATE INDEX idx_routes_tags ON routing.routes USING gin(tags);

-- Waypoints along routes
CREATE TABLE IF NOT EXISTS routing.route_waypoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    route_id UUID NOT NULL REFERENCES routing.routes(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- Waypoint location (can be a system or arbitrary point)
    system_id UUID REFERENCES stellar.star_systems(id),
    position GEOMETRY(PointZ, 4326),  -- Either system position or custom point

    -- Waypoint properties
    waypoint_type VARCHAR(50) DEFAULT 'transit',  -- 'origin', 'destination', 'transit', 'refuel', 'checkpoint'
    stop_duration_hours DOUBLE PRECISION,
    services_available TEXT[],

    -- Navigation hazards
    hazard_description TEXT,
    hazard_level INTEGER DEFAULT 0,

    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT unique_waypoint_sequence UNIQUE(route_id, sequence_number),
    CONSTRAINT waypoint_location CHECK(
        (system_id IS NOT NULL) OR (position IS NOT NULL)
    )
);

CREATE INDEX idx_waypoints_route ON routing.route_waypoints(route_id);
CREATE INDEX idx_waypoints_system ON routing.route_waypoints(system_id);
CREATE INDEX idx_waypoints_position ON routing.route_waypoints USING gist(position);

-- Direct connections between systems (jump gates, wormholes, etc.)
CREATE TABLE IF NOT EXISTS routing.system_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES stellar.sessions(id) ON DELETE CASCADE,
    from_system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,
    to_system_id UUID NOT NULL REFERENCES stellar.star_systems(id) ON DELETE CASCADE,

    connection_type VARCHAR(50) NOT NULL,  -- 'jump_gate', 'wormhole', 'hyperspace_lane', 'subspace_tunnel'
    is_bidirectional BOOLEAN DEFAULT TRUE,
    is_stable BOOLEAN DEFAULT TRUE,

    -- Connection properties
    distance_override_ly DOUBLE PRECISION,  -- If different from actual distance
    travel_time_hours DOUBLE PRECISION,
    energy_cost INTEGER,
    max_ship_size VARCHAR(20),  -- 'small', 'medium', 'large', 'capital'

    -- Discovery
    discovered_date DATE,
    discovered_by VARCHAR(255),

    metadata JSONB DEFAULT '{}'::jsonb,

    CONSTRAINT unique_connection UNIQUE(session_id, from_system_id, to_system_id),
    CONSTRAINT check_different_systems CHECK(from_system_id != to_system_id)
);

CREATE INDEX idx_connections_session ON routing.system_connections(session_id);
CREATE INDEX idx_connections_from ON routing.system_connections(from_system_id);
CREATE INDEX idx_connections_to ON routing.system_connections(to_system_id);

-- Route segments (computed geometry between waypoints)
CREATE TABLE IF NOT EXISTS routing.route_segments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    route_id UUID NOT NULL REFERENCES routing.routes(id) ON DELETE CASCADE,
    from_waypoint_id UUID NOT NULL REFERENCES routing.route_waypoints(id) ON DELETE CASCADE,
    to_waypoint_id UUID NOT NULL REFERENCES routing.route_waypoints(id) ON DELETE CASCADE,

    -- Segment geometry
    segment_line GEOMETRY(LineStringZ, 4326) NOT NULL,
    segment_length_ly DOUBLE PRECISION,

    -- Segment properties
    travel_time_hours DOUBLE PRECISION,
    danger_level INTEGER DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_segments_route ON routing.route_segments(route_id);
CREATE INDEX idx_segments_geometry ON routing.route_segments USING gist(segment_line);

-- Trigger for modified_at
CREATE TRIGGER update_routes_modified
    BEFORE UPDATE ON routing.routes
    FOR EACH ROW
    EXECUTE FUNCTION stellar.update_modified_at();

-- Function to create a route from a list of systems
CREATE OR REPLACE FUNCTION routing.create_route_from_systems(
    p_session_id UUID,
    p_route_name VARCHAR(255),
    p_system_ids UUID[],
    p_route_type VARCHAR(50) DEFAULT 'trade'
)
RETURNS UUID AS $$
DECLARE
    v_route_id UUID;
    v_route_type_id UUID;
    v_sequence INTEGER := 1;
    v_system_id UUID;
    v_prev_position GEOMETRY;
    v_curr_position GEOMETRY;
    v_prev_waypoint_id UUID;
    v_curr_waypoint_id UUID;
BEGIN
    -- Get route type
    SELECT id INTO v_route_type_id
    FROM routing.route_types
    WHERE name = p_route_type;

    -- Create the route
    INSERT INTO routing.routes (
        session_id,
        name,
        route_type_id
    ) VALUES (
        p_session_id,
        p_route_name,
        v_route_type_id
    ) RETURNING id INTO v_route_id;

    -- Create waypoints for each system
    FOREACH v_system_id IN ARRAY p_system_ids
    LOOP
        -- Get system position
        SELECT position INTO v_curr_position
        FROM stellar.star_systems
        WHERE id = v_system_id;

        -- Create waypoint
        INSERT INTO routing.route_waypoints (
            route_id,
            sequence_number,
            system_id,
            position,
            waypoint_type
        ) VALUES (
            v_route_id,
            v_sequence,
            v_system_id,
            v_curr_position,
            CASE
                WHEN v_sequence = 1 THEN 'origin'
                WHEN v_sequence = array_length(p_system_ids, 1) THEN 'destination'
                ELSE 'transit'
            END
        ) RETURNING id INTO v_curr_waypoint_id;

        -- Create segment if not first waypoint
        IF v_prev_waypoint_id IS NOT NULL THEN
            INSERT INTO routing.route_segments (
                route_id,
                from_waypoint_id,
                to_waypoint_id,
                segment_line,
                segment_length_ly
            ) VALUES (
                v_route_id,
                v_prev_waypoint_id,
                v_curr_waypoint_id,
                ST_MakeLine(v_prev_position, v_curr_position),
                ST_3DDistance(v_prev_position, v_curr_position) / 9.461e15
            );
        END IF;

        v_prev_position := v_curr_position;
        v_prev_waypoint_id := v_curr_waypoint_id;
        v_sequence := v_sequence + 1;
    END LOOP;

    RETURN v_route_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get complete route geometry
CREATE OR REPLACE FUNCTION routing.get_route_geometry(
    p_route_id UUID
)
RETURNS GEOMETRY AS $$
DECLARE
    v_geometry GEOMETRY;
BEGIN
    SELECT ST_Union(segment_line)
    INTO v_geometry
    FROM routing.route_segments
    WHERE route_id = p_route_id
    ORDER BY (
        SELECT sequence_number
        FROM routing.route_waypoints
        WHERE id = from_waypoint_id
    );

    RETURN v_geometry;
END;
$$ LANGUAGE plpgsql;

-- Function to find routes passing through or near a point
CREATE OR REPLACE FUNCTION routing.find_routes_near_point(
    p_session_id UUID,
    p_point GEOMETRY,
    p_radius_ly DOUBLE PRECISION
)
RETURNS TABLE(
    route_id UUID,
    route_name VARCHAR(255),
    distance_ly DOUBLE PRECISION
) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT
        r.id,
        r.name,
        MIN(ST_3DDistance(p_point, rs.segment_line) / 9.461e15) AS distance_ly
    FROM routing.routes r
    JOIN routing.route_segments rs ON rs.route_id = r.id
    WHERE r.session_id = p_session_id
    AND ST_3DDWithin(rs.segment_line, p_point, p_radius_ly * 9.461e15)
    GROUP BY r.id, r.name
    ORDER BY distance_ly;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate shortest path between systems (using connections)
CREATE OR REPLACE FUNCTION routing.find_shortest_path(
    p_session_id UUID,
    p_from_system_id UUID,
    p_to_system_id UUID
)
RETURNS TABLE(
    step INTEGER,
    system_id UUID,
    system_name VARCHAR(255),
    cumulative_distance_ly DOUBLE PRECISION
) AS $$
BEGIN
    -- This would use pgRouting if installed, or a custom implementation
    -- For now, return a simple result
    RETURN QUERY
    WITH RECURSIVE path AS (
        -- Start node
        SELECT
            1 AS step,
            s.id,
            s.name,
            0::DOUBLE PRECISION AS cumulative_distance_ly
        FROM stellar.star_systems s
        WHERE s.id = p_from_system_id

        UNION ALL

        -- Recursive step (simplified - would need proper pathfinding)
        SELECT
            p.step + 1,
            s.id,
            s.name,
            p.cumulative_distance_ly + ST_3DDistance(s.position, ps.position) / 9.461e15
        FROM path p
        JOIN stellar.star_systems ps ON ps.id = p.id
        JOIN routing.system_connections c ON c.from_system_id = ps.id
        JOIN stellar.star_systems s ON s.id = c.to_system_id
        WHERE p.step < 100  -- Prevent infinite recursion
        AND s.id = p_to_system_id
    )
    SELECT * FROM path
    ORDER BY step;
END;
$$ LANGUAGE plpgsql;