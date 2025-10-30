# SolarViewer Project

## Project Vision

SolarViewer is a Rust-based tool for extracting, storing, and visualizing stellar cartography data from Astrosynthesis save files. The goal is to create a better viewer and mapper that provides superior 2D visualization of 3D stellar data, with PostgreSQL/PostGIS as the data backend for advanced spatial queries and persistent storage.

## Core Goals

1. **Schema Extraction**: Map and document the complete Astrosynthesis SQLite schema
2. **Data Migration**: Extract data from .AstroDB files and migrate to PostgreSQL with PostGIS
3. **Multi-File Management**: Store multiple "slices" (named subsets) from different Astrosynthesis files
4. **2D Visualization**: Implement intelligent 2D projection of 3D stellar data using hybrid layout algorithms
5. **Performance**: Avoid reloading Astrosynthesis files repeatedly by maintaining persistent PostgreSQL storage

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

## Development Workflow

### Current Priority: Schema Discovery

**Immediate Next Steps**:
1. Set up Rust project with Cargo
2. Add rusqlite dependency
3. Create schema exploration tool
4. Test with sample .AstroDB file
5. Generate SCHEMA.md documentation

### Git Workflow

- `main` branch: Stable releases
- `develop` branch: Integration branch
- Feature branches: `feature/schema-discovery`, `feature/postgres-setup`, etc.
- Commit frequently with descriptive messages

### Testing Strategy

- Unit tests for data transformations
- Integration tests with sample .AstroDB files
- Benchmark tests for layout algorithms
- Visual regression tests for rendering

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

**Last Updated**: 2025-10-30
**Project Status**: Phase 1 - Schema Discovery (In Progress)
**Repository**: https://github.com/rem5357/SolarViewer
