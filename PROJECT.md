# SolarViewer Project

## Project Vision

SolarViewer is a Rust-based tool for extracting, storing, and visualizing stellar cartography data from Astrosynthesis save files. The goal is to create a better viewer and mapper that provides superior 2D visualization of 3D stellar data, with PostgreSQL/PostGIS as the data backend for advanced spatial queries and persistent storage.

## Core Goals

1. **Schema Extraction**: Map and document the complete Astrosynthesis SQLite schema ✅ **COMPLETE**
2. **Data Migration**: Extract data from .AstroDB files and migrate to PostgreSQL with PostGIS
3. **Multi-File Management**: Store multiple "slices" (named subsets) from different Astrosynthesis files
4. **2D Visualization**: Implement intelligent 2D projection of 3D stellar data using hybrid layout algorithms
5. **Performance**: Avoid reloading Astrosynthesis files repeatedly by maintaining persistent PostgreSQL storage

---

## Project Status & Progress

**Current Status**: Phase 1 Complete - Schema Exploration Tool Implemented
**Last Session**: 2025-10-30
**Repository**: https://github.com/rem5357/SolarViewer

### Quick Start for New Sessions

**Required Tools** (Installed):
- **Rust 1.90.0** via rustup
  - Location: `%USERPROFILE%\.cargo\bin\`
  - Use: `"$USERPROFILE/.cargo/bin/cargo.exe" [command]` or restart terminal
- **GitHub CLI 2.81.0**
  - Location: `C:\Program Files\GitHub CLI\gh.exe`
  - Authenticated as: **rem5357**
  - Use: `"C:\Program Files\GitHub CLI\gh.exe" [command]` or restart terminal

**Test Data**:
- File: `TotalSystem.AstroDB` (in project root)
- 14 tables, 627 bodies (stars/planets/moons), 45 routes
- Schema documentation: `docs/SCHEMA.md` (498 lines)

**Build & Test**:
```bash
# Build project (always after code changes)
cargo build

# Run schema exploration
cargo run -- schema --file TotalSystem.AstroDB --output docs/SCHEMA.md

# Check for errors without building
cargo check
```

### Completed Work (Session 2025-10-30)

#### Infrastructure Setup
- [x] Created consolidated PROJECT.md merging AstroSQL.md and StarMap2D_Visualization.md
- [x] Initialized Rust project structure with Cargo.toml
- [x] Created CLI skeleton with schema/import/map subcommands
- [x] Set up git repository with .gitignore
- [x] Installed GitHub CLI (gh) via winget
- [x] Authenticated GitHub CLI as rem5357
- [x] Created GitHub repository: https://github.com/rem5357/SolarViewer
- [x] Pushed initial code to GitHub
- [x] Installed Rust 1.90.0 via rustup
- [x] Added development workflow notes to PROJECT.md

#### Phase 1: Schema Exploration (COMPLETE)
- [x] Created `src/schema/mod.rs` module structure
- [x] Implemented `src/schema/discovery.rs`:
  - SchemaExplorer connects to SQLite .AstroDB files
  - Discovers all tables, columns, types, constraints
  - Extracts foreign key relationships
  - Samples data from each table
  - Counts rows per table
- [x] Implemented `src/schema/documentation.rs`:
  - Generates comprehensive markdown documentation
  - Table of contents with hyperlinks
  - Summary statistics table
  - Detailed column information with types and constraints
  - Foreign key relationship display
  - Sample data preview (5 rows per table)
  - Text-based relationship diagram
- [x] Integrated schema command into main.rs
- [x] Added chrono dependency for timestamps
- [x] Tested with TotalSystem.AstroDB - SUCCESS
- [x] Generated docs/SCHEMA.md (498 lines)

#### Key Findings from TotalSystem.AstroDB

**Database Structure**:
- **14 tables** discovered
- **No formal foreign key constraints** (relationships via parent_id/system_id columns)
- Hierarchical data model (parent_id references)

**Critical Tables**:

1. **bodies** (627 rows, 63 columns)
   - All celestial objects: stars, planets, moons, asteroids, stations
   - Columns: id, system_id, parent_id, name, type_id, body_type, spectral
   - **3D Coordinates**: x, y, z (double precision, in light-years)
   - Physical properties: distance, radius, density, mass, rotation
   - Stellar properties: luminosity, temperature, spectral type
   - Planetary properties: albedo, atmosphere, hydrosphere, greenhouse
   - Orbital parameters: semi_major, eccentricity, inclination, long_asc_node
   - Surface conditions: temp, pressure, gravity
   - Life/habitability indicators

2. **routes** (45 rows, 13 columns)
   - Connections between star systems
   - Columns: id, name, start_body_id, end_body_id, length, color
   - Route metadata: hidden, locked, width, system_color

3. **route_waypoints** (90 rows, 7 columns)
   - Path details for routes
   - Columns: route_id, sequence, x, y, z, label, locked

4. **sector_info** (1 row, 36 columns)
   - Overall sector metadata
   - Map dimensions, projection settings, background images
   - Default generation parameters

5. **sector_views** (4 rows, 12 columns)
   - Saved view configurations
   - Camera position and orientation

6. **subsectors** (0 rows, 22 columns)
   - Grid-based spatial divisions (none in test file)

7. **atm_components** (28 rows, 4 columns)
   - Atmospheric gas composition for bodies
   - Columns: id, body_id, gas, percent

8. **system_data_config** (110 rows, 8 columns)
   - Configuration/property definitions

**Data Model Insights**:
- Bodies are hierarchical: system_id (star) → parent_id (planet) → children (moons)
- Stars have system_id = id (they are the system)
- Planets have system_id = star.id, parent_id = star.id
- Moons have system_id = star.id, parent_id = planet.id
- Routes connect bodies via start_body_id/end_body_id
- Coordinates are 3D Cartesian (x, y, z) in light-years from sector origin
- Astrosynthesis uses rotated coordinate system vs standard Galactic

**PostgreSQL Migration Considerations**:
- Need to model hierarchical relationships explicitly
- Add PostGIS geometry columns for spatial queries
- Create indices on system_id, parent_id for joins
- Consider partitioning bodies by type_id or body_type
- Store original Astrosynthesis IDs for reference
- Track source_file_id for multi-file management

### Files Created This Session

**Source Code**:
- `src/main.rs` - CLI with clap, integrated schema command
- `src/schema/mod.rs` - Module exports
- `src/schema/discovery.rs` - Schema exploration (182 lines)
- `src/schema/documentation.rs` - Markdown generation (125 lines)

**Documentation**:
- `PROJECT.md` - Comprehensive project document (this file)
- `README.md` - User-facing documentation
- `GITHUB_SETUP.md` - GitHub repository setup instructions
- `docs/SCHEMA.md` - Generated schema documentation (498 lines)

**Configuration**:
- `Cargo.toml` - Rust dependencies and project metadata
- `.gitignore` - Excludes .AstroDB files, target/, build artifacts

**Reference Materials** (preserved):
- `AstroSQL.md` - Astrosynthesis technical background
- `StarMap2D_Visualization.md` - 2D projection algorithms

### What Works Right Now

```bash
# Explore any .AstroDB file and generate documentation
cargo run -- schema --file YourFile.AstroDB --output docs/OUTPUT.md

# The tool will:
# 1. Connect to the SQLite database
# 2. Discover all tables and their structure
# 3. Extract column types, constraints, primary keys
# 4. Find foreign key relationships
# 5. Count rows in each table
# 6. Sample 5 rows from each table
# 7. Generate comprehensive markdown documentation
```

### Next Steps (Phase 2: PostgreSQL Setup)

**Immediate Tasks**:
1. Design PostgreSQL schema based on Astrosynthesis findings
2. Create SQL migration scripts
3. Set up PostGIS extension
4. Implement connection pooling
5. Create basic CRUD operations for bodies, routes, sectors

**PostgreSQL Schema Design** (from discovered structure):
```sql
-- Track source files for multi-file management
CREATE TABLE source_files (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    original_path TEXT,
    imported_at TIMESTAMP DEFAULT NOW(),
    sector_name TEXT,
    metadata JSONB
);

-- Star systems with PostGIS spatial support
CREATE TABLE star_systems (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER REFERENCES source_files(id),
    original_body_id INTEGER,  -- ID from Astrosynthesis bodies table
    name TEXT,
    position GEOMETRY(PointZ, 4326),  -- PostGIS 3D point
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION,
    spectral_type TEXT,
    luminosity DOUBLE PRECISION,
    temperature DOUBLE PRECISION,
    mass DOUBLE PRECISION,
    radius DOUBLE PRECISION,
    metadata JSONB,  -- Store all other Astrosynthesis fields
    UNIQUE(source_file_id, original_body_id)
);

-- Planets
CREATE TABLE planets (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER REFERENCES source_files(id),
    star_system_id INTEGER REFERENCES star_systems(id) ON DELETE CASCADE,
    original_body_id INTEGER,
    name TEXT,
    body_type TEXT,  -- terrestrial, gas giant, ice giant, etc.
    parent_id INTEGER,  -- For moons of moons (rare but possible)

    -- Orbital parameters
    semi_major_axis_au DOUBLE PRECISION,
    eccentricity DOUBLE PRECISION,
    inclination DOUBLE PRECISION,

    -- Physical properties
    mass_earth_masses DOUBLE PRECISION,
    radius_km DOUBLE PRECISION,
    density DOUBLE PRECISION,
    gravity DOUBLE PRECISION,

    -- Surface conditions
    temperature DOUBLE PRECISION,
    atmospheric_pressure DOUBLE PRECISION,

    metadata JSONB,
    UNIQUE(source_file_id, original_body_id)
);

-- Moons (similar structure to planets)
CREATE TABLE moons (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER REFERENCES source_files(id),
    planet_id INTEGER REFERENCES planets(id) ON DELETE CASCADE,
    star_system_id INTEGER REFERENCES star_systems(id) ON DELETE CASCADE,
    original_body_id INTEGER,
    name TEXT,

    -- Orbital and physical properties
    semi_major_axis_km DOUBLE PRECISION,
    mass_earth_masses DOUBLE PRECISION,
    radius_km DOUBLE PRECISION,

    metadata JSONB,
    UNIQUE(source_file_id, original_body_id)
);

-- Routes between star systems
CREATE TABLE routes (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER REFERENCES source_files(id),
    original_route_id INTEGER,
    name TEXT,
    from_system_id INTEGER REFERENCES star_systems(id) ON DELETE CASCADE,
    to_system_id INTEGER REFERENCES star_systems(id) ON DELETE CASCADE,
    distance_ly DOUBLE PRECISION,
    color TEXT,
    metadata JSONB,
    UNIQUE(source_file_id, original_route_id)
);

-- Route waypoints for detailed paths
CREATE TABLE route_waypoints (
    id SERIAL PRIMARY KEY,
    route_id INTEGER REFERENCES routes(id) ON DELETE CASCADE,
    sequence INTEGER,
    position GEOMETRY(PointZ, 4326),
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION,
    label TEXT,
    UNIQUE(route_id, sequence)
);

-- Spatial indices for fast 3D queries
CREATE INDEX idx_star_position ON star_systems USING GIST(position);
CREATE INDEX idx_star_source ON star_systems(source_file_id);
CREATE INDEX idx_star_coords ON star_systems(x, y, z);

CREATE INDEX idx_planet_star ON planets(star_system_id);
CREATE INDEX idx_planet_source ON planets(source_file_id);

CREATE INDEX idx_moon_planet ON moons(planet_id);
CREATE INDEX idx_moon_system ON moons(star_system_id);

CREATE INDEX idx_route_from ON routes(from_system_id);
CREATE INDEX idx_route_to ON routes(to_system_id);
CREATE INDEX idx_route_source ON routes(source_file_id);

CREATE INDEX idx_waypoint_route ON route_waypoints(route_id);
CREATE INDEX idx_waypoint_position ON route_waypoints USING GIST(position);
```

**Migration Strategy**:
1. Read bodies table from Astrosynthesis
2. Identify stars (where system_id = id)
3. Insert stars into star_systems table
4. Identify planets (where parent_id = system_id and type indicates planet)
5. Insert planets into planets table
6. Identify moons (where parent_id != system_id)
7. Insert moons into moons table
8. Import routes table
9. Import route_waypoints table
10. Handle coordinate transformation if needed

---

## Architecture Overview

### Technology Stack

**Core:**
- **Language**: Rust (for performance, safety, and excellent SQLite/PostgreSQL support)
- **Source Database**: SQLite (Astrosynthesis .AstroDB files)
- **Target Database**: PostgreSQL with PostGIS extension (3D spatial queries)
- **Graph Processing**: petgraph (for force-directed layouts and route planning)
- **Linear Algebra**: nalgebra or ndarray (for PCA, transformations)

**Key Crates (Planned):**
- `rusqlite` - Read Astrosynthesis SQLite databases
- `tokio-postgres` or `diesel` - PostgreSQL async/ORM access
- `petgraph` - Graph structures for star networks
- `nalgebra` - Vector/matrix operations for projections
- `plotters` or `tiny-skia` - 2D visualization rendering
- `serde` - Serialization for data exchange

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     SolarViewer System                       │
└─────────────────────────────────────────────────────────────┘

┌──────────────────┐         ┌─────────────────────────────┐
│  Astrosynthesis  │────────>│   Schema Extractor          │
│  .AstroDB Files  │         │   - Discover tables         │
│  (SQLite)        │         │   - Map relationships       │
└──────────────────┘         │   - Document schema         │
                             └─────────────────────────────┘
                                        │
                                        v
                             ┌─────────────────────────────┐
                             │   Data Transformer          │
                             │   - Extract star systems    │
                             │   - Convert coordinates     │
                             │   - Validate hierarchy      │
                             └─────────────────────────────┘
                                        │
                                        v
                             ┌─────────────────────────────┐
                             │   PostgreSQL + PostGIS      │
                             │   - Named file collections  │
                             │   - Spatial indexing        │
                             │   - 3D queries              │
                             └─────────────────────────────┘
                                        │
                                        v
                             ┌─────────────────────────────┐
                             │   Layout Engine             │
                             │   - PCA projection          │
                             │   - Overlap resolution      │
                             │   - Force refinement        │
                             └─────────────────────────────┘
                                        │
                                        v
                             ┌─────────────────────────────┐
                             │   2D Renderer               │
                             │   - Distance annotations    │
                             │   - Depth encoding          │
                             │   - Export formats          │
                             └─────────────────────────────┘
```

### Database Schema Design

**PostgreSQL Organization:**

```sql
-- Track source files
CREATE TABLE source_files (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    original_path TEXT,
    imported_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB
);

-- Store star systems with spatial data
CREATE TABLE star_systems (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER REFERENCES source_files(id),
    original_id INTEGER,  -- ID from Astrosynthesis
    name TEXT,
    position GEOMETRY(PointZ, 4326),  -- PostGIS 3D point
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION,
    spectral_type TEXT,
    luminosity_class TEXT,
    distance_from_sol DOUBLE PRECISION,
    metadata JSONB
);

-- Store planets
CREATE TABLE planets (
    id SERIAL PRIMARY KEY,
    star_system_id INTEGER REFERENCES star_systems(id),
    original_id INTEGER,
    name TEXT,
    orbital_radius_au DOUBLE PRECISION,
    planet_type TEXT,
    mass_earth_masses DOUBLE PRECISION,
    metadata JSONB
);

-- Store moons
CREATE TABLE moons (
    id SERIAL PRIMARY KEY,
    planet_id INTEGER REFERENCES planets(id),
    original_id INTEGER,
    name TEXT,
    orbital_radius_km DOUBLE PRECISION,
    metadata JSONB
);

-- Store routes/connections
CREATE TABLE routes (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER REFERENCES source_files(id),
    from_system_id INTEGER REFERENCES star_systems(id),
    to_system_id INTEGER REFERENCES star_systems(id),
    distance_ly DOUBLE PRECISION,
    metadata JSONB
);

-- Store computed 2D layouts (cached)
CREATE TABLE layout_cache (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER REFERENCES source_files(id),
    layout_algorithm TEXT,
    parameters JSONB,
    computed_at TIMESTAMP DEFAULT NOW(),
    layout_data JSONB  -- star_id -> {x, y} mappings
);

CREATE INDEX idx_star_position ON star_systems USING GIST(position);
CREATE INDEX idx_star_source ON star_systems(source_file_id);
CREATE INDEX idx_routes_source ON routes(source_file_id);
```

### Module Structure

```
solarviewer/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI interface
│   ├── lib.rs                  # Library exports
│   ├── schema/
│   │   ├── mod.rs
│   │   ├── discovery.rs        # Astrosynthesis schema exploration
│   │   └── documentation.rs    # Generate schema docs
│   ├── extraction/
│   │   ├── mod.rs
│   │   ├── sqlite_reader.rs    # Read .AstroDB files
│   │   ├── models.rs           # Data structures
│   │   └── validator.rs        # Data validation
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── postgres.rs         # PostgreSQL operations
│   │   ├── migrations.rs       # Schema migrations
│   │   └── queries.rs          # Common queries
│   ├── projection/
│   │   ├── mod.rs
│   │   ├── pca.rs              # PCA projection
│   │   ├── forces.rs           # Force-directed layout
│   │   ├── hybrid.rs           # Hybrid algorithm
│   │   └── collision.rs        # Overlap resolution
│   └── visualization/
│       ├── mod.rs
│       ├── renderer.rs         # SVG/PNG rendering
│       ├── styling.rs          # Visual styles
│       └── annotations.rs      # Labels, distances
├── tests/
│   └── integration_tests.rs
└── docs/
    └── SCHEMA.md               # Generated schema documentation
```

## Implementation Phases

### Phase 1: Schema Discovery and Documentation (CURRENT)
**Goal**: Understand and document the Astrosynthesis database structure

**Tasks**:
1. Create Rust CLI tool to connect to .AstroDB files
2. Query SQLite metadata tables to discover schema
3. Extract table definitions, column types, foreign keys
4. Generate comprehensive SCHEMA.md documentation
5. Create sample queries for each table type

**Deliverables**:
- Schema exploration tool
- SCHEMA.md with complete table documentation
- Sample .AstroDB file for testing

### Phase 2: PostgreSQL Setup
**Goal**: Establish target database with PostGIS

**Tasks**:
1. Design PostgreSQL schema (see above)
2. Create migration scripts
3. Set up PostGIS spatial indexing
4. Implement connection pooling
5. Create basic CRUD operations

**Deliverables**:
- PostgreSQL schema DDL
- Database connection module
- Migration tooling

### Phase 3: Data Extraction and Migration
**Goal**: Extract data from Astrosynthesis and load into PostgreSQL

**Tasks**:
1. Implement SQLite reading with rusqlite
2. Parse hierarchical data (sectors -> stars -> planets -> moons)
3. Transform coordinates (handle Astrosynthesis coordinate system)
4. Validate data integrity
5. Load into PostgreSQL with proper relationships
6. Handle multiple source files with naming

**Deliverables**:
- Data extraction pipeline
- Coordinate transformation functions
- Multi-file import capability

### Phase 4: 2D Projection Algorithm
**Goal**: Implement hybrid layout algorithm for 2D visualization

**Tasks**:
1. Implement PCA projection as initial pass
2. Implement collision detection
3. Implement overlap resolution with repulsive forces
4. Add force-directed refinement (optional)
5. Benchmark performance with various dataset sizes

**Deliverables**:
- Layout engine with multiple algorithms
- Performance benchmarks
- Layout caching system

### Phase 5: Visualization and Rendering
**Goal**: Generate high-quality 2D maps with annotations

**Tasks**:
1. Design visual style (colors, fonts, line weights)
2. Implement SVG renderer
3. Add distance annotations on edges
4. Add depth encoding (z-coordinate labels, color tinting)
5. Implement multi-view displays
6. Add legend and scale indicators

**Deliverables**:
- SVG/PNG export functionality
- Styled, annotated 2D maps
- Multiple projection view support

### Phase 6: Polish and Optimization
**Goal**: Production-ready tool with good UX

**Tasks**:
1. CLI improvements (progress bars, better error messages)
2. Configuration file support
3. Incremental updates (don't re-import unchanged files)
4. Spatial query examples using PostGIS
5. Documentation and examples

**Deliverables**:
- Polished CLI tool
- User documentation
- Example queries and use cases

---

## Astrosynthesis Technical Background

### File Format Overview

**Astrosynthesis 3.0** uses SQLite databases with `.AstroDB` extension:
- Format: SQLite 3 database
- Can be opened with any SQLite-compatible tool
- Contains relational tables for hierarchical stellar data
- Single-file database architecture

**Data Hierarchy**:
```
Sector
├── Subsectors (grid-based spatial divisions)
├── Stars (including multiple star systems)
│   ├── Star Properties (spectral type, luminosity, etc.)
│   ├── Planets
│   │   ├── Orbital parameters
│   │   ├── Physical properties
│   │   └── Moons
│   │       ├── Orbital parameters
│   │       └── Physical properties
│   └── Other bodies (asteroids, stations, etc.)
└── Routes (connections between systems)
```

**Coordinate System**:
- Uses 3D Cartesian coordinates (X, Y, Z) in light-years
- **IMPORTANT**: Astrosynthesis uses a rotated coordinate system compared to standard Galactic coordinates
- For scientific use, convert to standard Galactic XYZ coordinates
- Distance from Sol calculated and stored

### Known Schema Information

**Critical Note**: The Astrosynthesis database schema is NOT officially documented. Schema exploration is required.

**What We Know**:
- Multiple star systems: Position values in AU, relative to system center
- Orbital parameters: Fields exist but may not be fully populated
- Custom fields: Can be added and displayed
- Referential integrity: Parent-child relationships via foreign keys
- Distance fields: Stored in light-years

**Schema Exploration Process**:
```sql
-- List all tables
SELECT name FROM sqlite_master WHERE type='table';

-- View table structure
PRAGMA table_info(table_name);

-- Sample data from each table
SELECT * FROM table_name LIMIT 5;

-- Find foreign key relationships
PRAGMA foreign_key_list(table_name);
```

### Data Extraction Methods

**Direct SQLite Access (Our Approach)**:
```rust
use rusqlite::{Connection, Result};

fn explore_schema(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;

    // List tables
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table'"
    )?;

    let tables = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    })?;

    for table in tables {
        println!("Table: {:?}", table?);
    }

    Ok(())
}
```

**Advantages**:
- Complete access to all data
- Most flexible and powerful method
- No need for Astrosynthesis to be running
- Works with any programming language that supports SQLite

---

## 2D Visualization Strategy

### The Core Challenge

Projecting 3D stellar coordinates to 2D involves fundamental trade-offs:

**Must Preserve**:
- Relative neighborhood relationships
- Actual 3D distances for route planning
- Connectivity information

**Must Avoid**:
- Visual overlap (stars hiding each other)
- Misleading proximities (stars appearing close but far in 3D)
- Unreadable clutter in dense regions

**Philosophy**: Think subway map, not topographic map. Schematic clarity with annotated reality.

### Recommended Approach: Hybrid Algorithm

**Stage 1: Initial Projection (PCA)**
- Fast, deterministic starting point
- Preserves overall structure
- Finds "best viewing angle" through 3D space

**Stage 2: Overlap Resolution**
- Detect colliding star positions
- Apply repulsive forces to separate
- Maintain minimum separation threshold

**Stage 3: Force Refinement (Optional)**
- Apply force-directed layout for connected stars
- Make 2D distances proportional to 3D distances
- Balance between accuracy and readability

**Stage 4: Aesthetic Polish**
- Optimize label placement
- Balance white space
- Align to grid (optional)

### Visual Enhancements

**Distance Annotation** (Critical):
```
Star A ----3.2 LY---- Star B
```
Always show real 3D distances on edges - 2D distances are misleading!

**Edge Styling by Distance**:
- Solid thick line: 0-5 light-years (close neighbors)
- Solid thin line: 5-10 light-years
- Dashed line: 10-15 light-years
- Dotted line: 15-20 light-years
- No line: >20 light-years

**Depth Encoding**:
- Color tint: Blue (far) → White (middle) → Red (near)
- Size variation: Larger = closer to viewer
- Z-coordinate labels: `Star Name [z: +5.2]`

**Multi-View Support**:
- Main view: Optimized hybrid layout
- Side panels: XY, XZ, YZ orthographic projections
- Helps users build 3D mental model

### Algorithm Comparison

| Criterion | Force-Directed | MDS | PCA | Hierarchical | Hybrid |
|-----------|---------------|-----|-----|--------------|--------|
| Speed | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Distance Accuracy | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Overlap Avoidance | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Readability | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Deterministic | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Scalability | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

**For SolarViewer**: Hybrid approach offers the best balance of speed, accuracy, and readability.

### Implementation Priorities

**Minimum Viable Layout** (Fast Implementation):
1. Extract 3D coordinates from Astrosynthesis SQLite
2. Apply PCA projection (deterministic, fast)
3. Identify overlaps (stars within minimum separation)
4. Resolve overlaps with repulsive forces
5. Render with distance annotations on edges <15 LY
6. Add z-coordinate labels next to star names

**Future Enhancements**:
- Dynamic layouts based on zoom level
- Hierarchical expansion for dense clusters
- Semantic layout (weight by trade routes, not just distance)
- Interactive 3D rotation updating 2D projection

---

## Quality Metrics

### Quantitative Metrics

1. **Overlap Count**: Should be 0 (no stars closer than minimum separation)
2. **Distance Preservation Error**: Mean absolute percentage error <30% for connected stars
3. **Neighborhood Preservation**: >70% of k-nearest neighbors (k=5) preserved in 2D
4. **Layout Time**: <1 second for 100 stars, <10 seconds for 1000 stars

### Test Datasets

1. **Small** (10-20 stars): Development and debugging
2. **Medium** (100-200 stars): Representative sector
3. **Large** (1000+ stars): Stress test
4. **Pathological Cases**:
   - All stars in a line
   - All stars in a sphere
   - Dense clusters
   - Very sparse distributions

---

## Git Workflow

- `master` branch: Main development (currently used)
- Feature branches: `feature/postgres-setup`, `feature/data-import`, etc. (future)
- Commit frequently with descriptive messages
- All commits include Claude Code attribution

## Testing Strategy

- Unit tests for data transformations
- Integration tests with sample .AstroDB files (TotalSystem.AstroDB)
- Benchmark tests for layout algorithms (future)
- Visual regression tests for rendering (future)

---

## Resources

### Official Sources
- NBOS Software: https://www.nbos.com/products/astrosynthesis
- NBOS Forums: https://forum.nbos.com
- Plugin Repository: http://www.nbos.com/nox/index.php
- AstroScript API: http://www.nbos.com/support/docs/AlienAPI/

### Community Resources
- Evil Dr. Ganymede's Stellar Mapping: http://evildrganymede.net/wp/rpgs/stellar-mapping/
- Pre-converted stellar databases
- Coordinate conversion tools

### Tools
- DB Browser for SQLite: https://sqlitebrowser.org/
- PostgreSQL + PostGIS documentation
- Rust ecosystem: crates.io

---

## Configuration

### Expected Configuration File: `solarviewer.toml`

```toml
[database]
postgres_url = "postgresql://user:pass@localhost/solarviewer"

[extraction]
min_distance_threshold_ly = 20.0  # Only connect stars within this distance
coordinate_conversion = "galactic"  # or "astrosynthesis"

[layout]
algorithm = "hybrid"
min_separation_pixels = 20
max_iterations = 1000
pca_initial = true
force_refinement = true

[visualization]
output_format = "svg"
show_distance_labels = true
show_z_coordinates = true
edge_style_by_distance = true
depth_color_encoding = true

[style]
star_icon_size = 8
edge_thickness = 2
font_family = "Arial"
font_size = 10
```

---

## Success Criteria

**Phase 1 Complete When**:
- ✅ Can open any .AstroDB file
- ✅ Can list all tables and columns
- ✅ Can extract sample data from each table
- ✅ SCHEMA.md documents complete structure
- ✅ Understand hierarchical relationships

**Project Complete When**:
- ✅ Can import multiple .AstroDB files into PostgreSQL
- ✅ Can query spatial relationships using PostGIS
- ✅ Can generate readable 2D maps from 3D data
- ✅ Maps include distance annotations and depth cues
- ✅ Performance acceptable for 1000+ star sectors
- ✅ Can export maps in multiple formats (SVG, PNG)
- ✅ Documentation complete for users

---

## Development Workflow

### Session Continuity

This section contains important information for maintaining context across development sessions.

**Screenshot Directory**:
- Location: `D:\dropbox\screenshots`
- Screenshots are automatically stored here
- User will frequently reference screenshots for visual communication
- When user mentions "look at the SS" or "check the screenshot", refer to files in this directory

**Development Practices**:
- **Always rebuild after editing source files**: Run `cargo build` or `cargo run` after each code change to test
- **Test incrementally**: Don't accumulate multiple changes before testing
- **Verify compilation**: Ensure code compiles before moving to the next task
- **Run the tool**: Execute commands to verify functionality works as expected

**Typical Development Cycle**:
1. Make code changes
2. Run `cargo build --release` (or `cargo run -- [subcommand]` to build and test)
3. Test the specific functionality changed
4. Commit if working
5. Move to next task

**Build Commands**:
```bash
# Quick build (debug mode, faster compilation)
cargo build

# Optimized build (release mode, for performance testing)
cargo build --release

# Build and run with arguments
cargo run -- schema --file test.AstroDB --output docs/SCHEMA.md

# Run tests
cargo test

# Check for errors without building
cargo check
```

**Testing Workflow**:
- Keep a test .AstroDB file handy for quick iteration
- Test each subcommand as it's implemented
- Verify output files are generated correctly
- Check error messages are helpful

---

## Notes and Considerations

### Coordinate System Conversion

Astrosynthesis uses a rotated coordinate system. When importing real astronomical data or exporting to other systems, conversion is required. Community tools exist for this (see Evil Dr. Ganymede's resources).

### Schema Changes

Schema may change between Astrosynthesis versions. Our schema discovery tool should handle this gracefully and document version differences.

### Performance

- Use spatial indexing (PostGIS GIST indexes) for 3D queries
- Cache layout computations for frequently-viewed sectors
- Use Rayon for parallel force calculations
- Consider SIMD for vector operations
- Pre-compute distance matrices (expensive to compute repeatedly)

### Data Integrity

Always validate:
- Foreign key relationships preserved
- Coordinate values within reasonable bounds
- No orphaned records (planets without stars, etc.)
- Distance calculations correct after coordinate conversions

### Future Integration

Design with these potential features in mind:
- Web viewer (export to JSON for D3.js or similar)
- Real-time collaboration (multiple users viewing same sectors)
- Procedural generation (create new sectors programmatically)
- Route optimization (shortest path, refueling stops)
- Political/economic overlays (trade routes, territories)

---

**Last Updated**: 2025-10-30 (Session complete - Phase 1 finished)
**Project Status**: ✅ Phase 1 Complete - Schema Discovery Implemented and Tested
**Next Phase**: Phase 2 - PostgreSQL Setup & Data Migration
**Repository**: https://github.com/rem5357/SolarViewer
**Authenticated User**: rem5357
