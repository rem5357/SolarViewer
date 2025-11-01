# StellarForge Database Documentation

## Overview

StellarForge uses PostgreSQL with PostGIS extension to provide a powerful spatial database for stellar cartography. This system stores multiple galaxy sessions, political territories with influence zones, trade routes, and complex spatial relationships.

## Key Features

### 1. Session-Based Save System
- Store multiple complete galaxies or subsections
- Parent-child relationships for subsections
- Automatic statistics tracking
- Spatial bounds for efficient querying

### 2. Spatial Data with PostGIS
- 3D spatial indexing for millions of star systems
- Efficient nearest-neighbor queries
- Spatial intersection for political territories
- Route path analysis

### 3. Political Influence Zones
- "Translucent aura" effect around controlled systems
- Strength-based influence falloff
- Automatic detection of disputed territories
- Contested system identification

### 4. Advanced Routing
- Multi-waypoint trade routes
- Safe path finding avoiding hostile space
- Chokepoint identification
- Trade flow analysis

## Database Architecture

### Schemas
- `stellar` - Core astronomical objects (sessions, systems, bodies)
- `political` - Political entities and influence zones
- `routing` - Trade routes and connections

### Core Tables

#### stellar.sessions
Stores saved galaxies and subsections with spatial bounds.

#### stellar.star_systems
Star systems with 3D positions in both Cartesian and Galactic coordinates.
- Uses IAU standard Galactic Coordinate System
- Supports legacy Astrosynthesis coordinate conversion
- PostGIS POINT geometry for spatial queries

#### stellar.bodies
Celestial bodies (stars, planets, moons) with orbital parameters.
- JSONB storage for flexible physical properties
- Hierarchical parent-child relationships
- Keplerian orbital elements

#### political.entities
Political organizations with government types and properties.

#### political.influence_zones
3D spatial geometries representing political control.
- MultiPolygonZ geometries for complex shapes
- Strength multipliers for influence falloff
- Automatic zone generation from controlled systems

#### routing.routes
Trade and travel routes with economic data.
- Multiple waypoint support
- Trade value calculations
- Visual styling properties

## Coordinate System

StellarForge uses the IAU (International Astronomical Union) standard Galactic Coordinate System:

- **X-axis**: Points toward the Galactic Center (l = 0°, b = 0°)
- **Y-axis**: Points in direction of galactic rotation (l = 90°, b = 0°)
- **Z-axis**: Points toward North Galactic Pole (b = 90°)
- **Origin**: Sun's position

### Conversion from Astrosynthesis
Astrosynthesis uses a non-standard coordinate system. StellarForge provides automatic conversion:
```rust
let galactic = GalacticCoordinates::from_astrosynthesis_xyz(x, y, z);
```

## Setup Instructions

### Prerequisites
1. PostgreSQL 13+ installed
2. PostGIS 3.0+ extension
3. Rust toolchain

### Installation

1. **Run the setup script** (Windows PowerShell):
```powershell
.\setup_database.ps1
```

2. **Initialize the database**:
```bash
cargo run --bin stellarforge -- init
```

3. **Create your first session**:
```bash
cargo run --bin stellarforge -- session create --name "Milky Way"
```

### Manual Setup (Alternative)

1. **Create database**:
```sql
CREATE DATABASE stellarforge;
\c stellarforge
CREATE EXTENSION postgis;
CREATE EXTENSION postgis_topology;
```

2. **Run migrations**:
```bash
cargo run --bin stellarforge -- init
```

## CLI Usage Examples

### Session Management
```bash
# Create a new galaxy session
stellarforge session create --name "My Galaxy" --description "Test galaxy"

# List all sessions
stellarforge session list

# Create a subsection
stellarforge session subsection --parent-id <UUID> --name "Core Worlds" \
  --center-x 0 --center-y 0 --center-z 0 --radius 100
```

### Star Systems
```bash
# Add a system
stellarforge system add --session-id <UUID> --name "Sol" \
  --x 0 --y 0 --z 0 --system-type single

# Find nearby systems
stellarforge system near --session-id <UUID> \
  --x 10 --y 20 --z 5 --radius 50

# Find neighbors
stellarforge system neighbors --session-id <UUID> \
  --system-id <UUID> --limit 10
```

### Political Entities
```bash
# Create political entity
stellarforge political create --session-id <UUID> \
  --name "United Earth" --government-type federation

# Add system control
stellarforge political control --session-id <UUID> \
  --entity-id <UUID> --system-id <UUID> \
  --control-type sovereign --strength 1.0

# Generate influence zone
stellarforge political influence --session-id <UUID> \
  --entity-id <UUID> --base-radius 15.0

# Find contested systems
stellarforge political contested --session-id <UUID>
```

### Routes
```bash
# Create trade route
stellarforge route create --session-id <UUID> \
  --name "Sol-Alpha Trade Lane" \
  --systems <UUID1>,<UUID2>,<UUID3> \
  --route-type trade

# Find systems along route
stellarforge route along --route-id <UUID> --max-distance 5.0
```

### Analysis
```bash
# Find strategic chokepoints
stellarforge analyze chokepoints --session-id <UUID> --min-routes 3

# Find frontier systems
stellarforge analyze frontier --session-id <UUID> \
  --neighbor-distance 15.0 --max-neighbors 3

# Calculate political influence at point
stellarforge analyze influence --session-id <UUID> \
  --x 100 --y 50 --z -20

# Get density distribution
stellarforge analyze density --session-id <UUID> --grid-size 50

# Political power rankings
stellarforge analyze rankings --session-id <UUID>

# Find safe route avoiding hostile space
stellarforge analyze safe-route --session-id <UUID> \
  --from <UUID> --to <UUID> --avoid <HOSTILE_UUID> --max-jumps 10
```

## Spatial Query Examples

### Find Contested Systems
Systems claimed by multiple political entities:
```sql
WITH system_claims AS (
    SELECT s.id, s.name,
           COUNT(DISTINCT iz.political_entity_id) as claim_count
    FROM stellar.star_systems s
    JOIN political.influence_zones iz
        ON ST_3DWithin(s.position, iz.zone_geometry, 0)
    WHERE s.session_id = $1
    GROUP BY s.id, s.name
)
SELECT * FROM system_claims WHERE claim_count > 1;
```

### Calculate Influence Strength
Political influence with distance falloff:
```sql
SELECT pe.name,
    CASE
        WHEN ST_3DWithin(point, iz.zone_geometry, 0) THEN 1.0
        ELSE GREATEST(0, 1.0 - (ST_3DDistance(point, iz.zone_geometry) /
             (iz.base_radius_ly * 9.461e15)))
    END * iz.strength_multiplier as influence
FROM political.entities pe
JOIN political.influence_zones iz ON iz.political_entity_id = pe.id
WHERE ST_3DDWithin(point, iz.zone_geometry, iz.base_radius_ly * 9.461e15 * 1.5);
```

### Find Strategic Chokepoints
Systems that many routes pass through:
```sql
SELECT s.name, COUNT(DISTINCT rw.route_id) as route_count
FROM stellar.star_systems s
JOIN routing.route_waypoints rw ON rw.system_id = s.id
GROUP BY s.id, s.name
HAVING COUNT(DISTINCT rw.route_id) >= 3
ORDER BY route_count DESC;
```

## Performance Considerations

### Spatial Indexing
PostGIS automatically creates spatial indexes on geometry columns:
```sql
CREATE INDEX idx_systems_position ON stellar.star_systems USING GIST (position);
CREATE INDEX idx_influence_zones ON political.influence_zones USING GIST (zone_geometry);
```

### Query Optimization
- Use `ST_3DDWithin` for radius searches instead of calculating distances
- Use `&&` operator for bounding box pre-filtering
- Leverage spatial indexes with proper query patterns

### Scaling
- Supports millions of star systems with proper indexing
- Subsection system allows working with smaller datasets
- Batch operations for bulk imports

## Integration with Astrosynthesis

### Import Process
1. Parse `.AstroDB` files using existing SolarViewer code
2. Convert coordinates from Astrosynthesis to IAU Galactic
3. Create session for import
4. Bulk insert systems and bodies
5. Generate political zones if needed

### Coordinate Conversion
```rust
// From Astrosynthesis non-standard to IAU Galactic
let galactic = GalacticCoordinates::from_astrosynthesis_xyz(
    astro_x, astro_y, astro_z
);

// Store both for compatibility
system.galactic_coordinates = galactic;
system.legacy_position = Some(vec![astro_x, astro_y, astro_z]);
```

## Advanced Features

### Influence Zone Generation
Political influence zones are generated using:
1. Convex hull of controlled systems
2. Buffer expansion based on strength
3. Merge overlapping zones of same entity
4. 3D Voronoi-like tessellation for borders

### Safe Route Finding
Pathfinding avoiding hostile territory:
1. Recursive CTE for graph traversal
2. Spatial exclusion of hostile zones
3. Jump distance constraints
4. Minimum path optimization

### Subsection Creation
Extract spherical regions:
1. Spatial intersection query
2. Automatic system copying
3. Maintain relationships
4. Update statistics

## Future Enhancements

### Planned Features
- [ ] Temporal data (historical borders)
- [ ] Wormhole/jump gate networks
- [ ] Economic simulation data
- [ ] Military fleet positions
- [ ] Exploration status tracking
- [ ] Anomaly and artifact locations
- [ ] Trade commodity flows
- [ ] Diplomatic relationships

### Performance Improvements
- [ ] Materialized views for complex queries
- [ ] Partitioning for very large galaxies
- [ ] Caching layer for frequent queries
- [ ] Parallel import processing

## Troubleshooting

### Common Issues

**PostGIS not found**
- Install via Stack Builder (Windows)
- Use package manager (Linux): `apt-get install postgis`
- Check version compatibility

**Connection refused**
- Verify PostgreSQL is running
- Check password in connection string
- Ensure localhost connections allowed in pg_hba.conf

**Slow spatial queries**
- Run `ANALYZE` to update statistics
- Check spatial indexes exist
- Consider increasing work_mem for complex queries

## Database Credentials

Default configuration:
- **Host**: localhost
- **Port**: 5432
- **Database**: stellarforge
- **User**: postgres
- **Password**: Beta5357

Connection string:
```
postgresql://postgres:Beta5357@localhost/stellarforge
```

## Support

For issues or questions about the StellarForge database:
1. Check this documentation
2. Review SQL files in `/sql` directory
3. Examine Rust code in `/src/stellar_forge/database`
4. Run tests: `cargo test stellar_forge::database`

---

*StellarForge - Modern stellar cartography with the power of spatial databases*