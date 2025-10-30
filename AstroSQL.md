# Astrosynthesis File Format & Data Extraction Skill

## Overview

Astrosynthesis is a 3D stellar cartography application created by NBOS Software, designed for sci-fi gamers, authors, and space enthusiasts. It allows users to map large portions of space, plotting stars, interstellar routes, subsectors, and detailed solar systems in full 3D.

**Key Capabilities:**
- 3D visualization of stellar sectors
- Procedural star system generation
- Planet and moon generation with orbital mechanics
- Import/export of stellar data
- Route planning between star systems
- Integration with Fractal Mapper for planetary surface maps

## Version History

### Astrosynthesis 2.0
- Used proprietary file format
- Supported XML import/export
- Had its own scripting API (documented at http://www.nbos.com/support/docs/AlienAPI/)
- Plugin system for extensions
- Limited programmatic access

### Astrosynthesis 3.0 (Current)
- **Major Change:** Migrated to SQLite database format
- File extension: `.AstroDB`
- Uses SQLite for data storage (confirmed by NBOS developer)
- Largely compatible with v2 API but not 100%
- Improved system generator
- Enhanced plugin capabilities via AstroScript

## File Format Details

### AstroDB Files (.AstroDB)

**Technical Specifications:**
- Format: SQLite 3 database
- Can be opened with any SQLite-compatible tool
- Contains relational tables for hierarchical stellar data
- Single-file database architecture

**Data Hierarchy:**
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

**Coordinate System:**
- Uses 3D Cartesian coordinates (X, Y, Z)
- Coordinate values in light-years
- **Important:** Astrosynthesis uses a rotated coordinate system compared to standard Galactic coordinates
  - Astrosynthesis XYZ files must ONLY be used for importing into Astrosynthesis
  - For any other purpose, convert to standard Galactic XYZ coordinates

## Data Extraction Methods

### 1. Direct SQLite Access (Recommended)

**Advantages:**
- Complete access to all data
- Most flexible and powerful method
- Works with any programming language that supports SQLite
- No need for Astrosynthesis to be running

**Tools:**
- SQLite command-line interface
- DB Browser for SQLite (GUI)
- Programming libraries:
  - Python: `sqlite3` (built-in)
  - Rust: `rusqlite`
  - C#: `System.Data.SQLite` or `Microsoft.Data.Sqlite`
  - Any language with SQLite bindings

**Basic Access Pattern:**
```python
import sqlite3

# Open AstroDB file
conn = sqlite3.connect('sector.AstroDB')
cursor = conn.cursor()

# List all tables
cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
tables = cursor.fetchall()
print("Tables:", tables)

# Query star data (example - actual schema may vary)
cursor.execute("SELECT * FROM stars LIMIT 10")
stars = cursor.fetchall()
```

**Rust Example:**
```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("sector.AstroDB")?;
    
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

### 2. XML Export Method

**Process:**
1. Open sector in Astrosynthesis 3.0
2. Use `Plugins -> XML Export` from menu
3. Creates XML file with complete sector data
4. Parse XML in your target application

**Advantages:**
- Official export method
- Documented format (by example)
- Includes all data fields
- Version-independent (can export from v2 or v3)

**Disadvantages:**
- Requires Astrosynthesis to be running
- Manual process for each sector
- XML parsing overhead

**XML Import Plugin:**
- Available from NBOS: http://www.nbos.com/nox/index.php?action=1001&id=7
- Place in `/Plugins` directory and restart Astrosynthesis
- Allows importing XML data back into Astrosynthesis

### 3. CSV Import/Export

**Capabilities:**
- Limited to basic star data
- Primarily for star positions and basic properties
- Not suitable for complete system data (planets, moons, etc.)

**CSV Import Format (from manual, pg 101-103):**
```
Star,10000,,,100.5,50.2,-30.1,,,M0 V,,25.5
```

**Fields:**
- Column A: "Star" (type identifier)
- Column B: Unique index number
- Column C-E: (varies, see manual)
- Column F-H: X, Y, Z coordinates in light-years
- Column I-J: (blank/varies)
- Column K: Spectral type and size (e.g., "M0 V")
- Column L: (blank)
- Column M: Distance from Sol in light-years

**Limitations:**
- Only imports/exports star-level data
- Cannot import detailed planet/moon information
- Limited field support
- Manual says: "only designed to be a very rudimentary importer"

### 4. Direct Database Writing

For advanced users creating data programmatically:

**Approach:**
1. Create blank .AstroDB file in Astrosynthesis
2. Connect to SQLite database file
3. Write directly to tables using SQL INSERT statements
4. Maintain referential integrity (parent-child relationships)

**Advantages:**
- Fastest for bulk data generation
- Complete control over all fields
- Can automate entire sector creation

**Challenges:**
- Requires understanding of schema
- Must maintain data integrity
- Need to handle auto-increment IDs correctly
- Risk of creating invalid data if schema not properly understood

## Known Schema Information (Verified from TotalSystem.AstroDB)

### Complete Schema Documentation

The Astrosynthesis 3.0 schema has been reverse-engineered from analysis of TotalSystem.AstroDB:

**14 Tables Discovered:**
1. `bodies` (627 rows, 63 columns) - Core table: stars, planets, moons, asteroids, stations
2. `routes` (45 rows, 13 columns) - Interstellar travel routes
3. `route_waypoints` (90 rows, 7 columns) - Path details for routes
4. `atm_components` (28 rows, 4 columns) - Atmospheric composition for bodies
5. `sector_info` (1 row, 36 columns) - Sector-level metadata
6. `sector_views` (4 rows, 12 columns) - Saved camera views
7. `sector_view_arrays` (8 rows, 5 columns) - View configuration data
8. `subsectors` (0 rows, 22 columns) - Grid-based spatial divisions
9. `system_data_config` (110 rows, 8 columns) - Property definitions
10. `config` (1 row, 3 columns) - Database configuration
11. `custom_fields` (0 rows, 5 columns) - Custom properties
12. `fwe_color_groups` (0 rows, 11 columns) - Color mapping for surface features
13. `resources` (2 rows, 4 columns) - Binary data storage
14. `surface_maps` (0 rows, 5 columns) - Planetary surface map data

**No formal foreign key constraints** - Relationships maintained via parent_id/system_id columns

### The Bodies Table (Most Important)

**Core Columns (63 total):**

**Identification:**
- `id`: Unique body ID
- `id_string`: GUID for cross-reference
- `name`: Display name

**Hierarchy & System Classification:**
- `system_id`: ID of the star system (system_id = id for stars)
- `parent_id`: ID of the body this orbits (0 for root bodies)
- `type_id`: Numeric type identifier
- `body_type`: Text classification (e.g., "Terrestrial", "Gas Giant")

**Stellar Properties (for stars):**
- `spectral`: Spectral classification (e.g., "G3V", "M4V", "O1V")
- `luminosity`: Stellar luminosity in solar units (0 = not applicable)
- `mass`: Body mass (solar masses for stars, Earth masses for planets)
- `radius`: Body radius (solar radii for stars, Earth radii for planets)
- `temp`: Surface temperature in Kelvin (0 if not calculated)
- `density`: Average density

**Orbital Parameters:**
- `distance`: Orbital distance from parent (AU for planets)
- `eccentricity`: Orbital eccentricity (0 = circular)
- `inclination`: Orbital inclination in degrees
- `rotation`: Rotation period in hours
- `axial_tilt`: Axial tilt in degrees
- `retrograde_orbit`: 1 if retrograde motion

**Physical Properties:**
- `albedo`: Surface albedo/reflectivity
- `atmosphere`: Atmospheric pressure (Earth atmospheres)
- `water`: Water coverage percentage
- `habitability`: Habitability rating (0-100)
- `population`: Current population

**3D Coordinates:**
- `x`, `y`, `z`: Absolute position in light-years from sector origin
- **IMPORTANT:** Astrosynthesis uses a rotated coordinate system; convert for external use

**Display/Rendering:**
- `visible`: 1 if shown in 3D view
- `color`: RGB color value
- `label_color`: Label text color
- `render_distance`: Maximum distance to render
- `label_distance`: Label visibility distance

### Multi-Star System Architecture (CRITICAL FINDING)

**Single-Star System:**
```
Root Body: system_id = id, parent_id = 0, spectral = "G3V" (has spectral type)
├─ Planets: system_id = star_id, parent_id = star_id
│  └─ Moons: system_id = star_id, parent_id = planet_id
└─ Other bodies
```

**Multi-Star Container:**
```
Container: system_id = id, parent_id = 0, spectral = "" OR NULL (NO spectral type)
├─ Component Star A: parent_id = container_id, spectral = "G5V" (HAS spectral type)
├─ Component Star B: parent_id = container_id, spectral = "K9V" (HAS spectral type)
├─ Component Star C: parent_id = container_id, spectral = "B5V" (optional third star)
├─ Planets: system_id = container_id, parent_id = container_id
│  └─ Moons: system_id = container_id, parent_id = planet_id
└─ Other bodies
```

**Identification Pattern:**
- **Single-Star:** WHERE `system_id = id` AND `parent_id = 0` AND `spectral != ''`
- **Multi-Star Container:** WHERE `system_id = id` AND `parent_id = 0` AND `spectral IS NULL` OR `spectral = ''`
- **Component Stars:** WHERE `parent_id = container_id` AND `spectral != ''`

### Verified Data from TotalSystem.AstroDB

**System Breakdown:**
- Single-star systems: 246
- Multi-star containers: 103
- Multi-star component stars: 236
- **Total actual stars: 482** (not 349)

**Spectral Type Distribution:**
- O-class: Multiple (e.g., Zen: O1V with 319,567× Sun's luminosity)
- B-class: Multiple bright stars
- A-class: Several main sequence stars
- F-class: Several sub-giants
- G-class: Many sun-like stars (including Amateru: G3V)
- K-class: Multiple orange dwarfs
- M-class: Many red dwarfs

**Example Multi-Star Systems:**
- Binary: Aorimo (A: G5V + B: K9V)
- Binary: Arivalari (A: B1V + B: M3V)
- Triple: Bra (A: M1V + B: M0V + C: B5V)
- Triple: Yota (A: O0V + B: G0V + C: B4V)
- Quadruple: Zaika (A: A4V + B: M0V + C: F5V + D: F8V)

**Luminosity Range:**
- Faintest: 0.005 Sol (M-type dwarfs)
- Typical: 0.5 - 10 Sol (G, F, A types)
- Brightest: 456,216 Sol (Yota A, O0V hypergiant)

### Query Patterns for Star Extraction

**Extract All Single-Star Systems:**
```sql
SELECT id, name, spectral, radius, mass, luminosity, temp, x, y, z
FROM bodies
WHERE system_id = id AND parent_id = 0
AND spectral != '' AND spectral IS NOT NULL
ORDER BY name;
```
Returns: 246 rows

**Extract All Multi-Star Container Names (for reference):**
```sql
SELECT id, name, x, y, z,
       (SELECT COUNT(*) FROM bodies b2
        WHERE b2.parent_id = bodies.id AND b2.spectral != '')
       as component_count
FROM bodies
WHERE system_id = id AND parent_id = 0
AND (spectral = '' OR spectral IS NULL)
ORDER BY name;
```
Returns: 103 rows

**Extract Component Stars from Multi-Star Systems:**
```sql
SELECT b.id, b.name, b.spectral, b.radius, b.mass, b.luminosity, b.temp,
       b.x, b.y, b.z, c.name, c.x, c.y, c.z
FROM bodies b
JOIN bodies c ON b.parent_id = c.id
WHERE c.system_id = c.id AND c.parent_id = 0
AND (c.spectral = '' OR c.spectral IS NULL)
AND b.spectral != '' AND b.spectral IS NOT NULL
ORDER BY c.name, b.name;
```
Returns: 236 rows

**Get All Stars with System Context (482 total):**
```sql
-- Single stars (no system_name)
SELECT id, name, spectral, radius, mass, luminosity, temp,
       x, y, z, NULL as system_name, x as system_x, y as system_y, z as system_z
FROM bodies
WHERE system_id = id AND parent_id = 0
AND spectral != '' AND spectral IS NOT NULL

UNION ALL

-- Multi-star components (with system_name)
SELECT b.id, b.name, b.spectral, b.radius, b.mass, b.luminosity, b.temp,
       b.x, b.y, b.z, c.name, c.x, c.y, c.z
FROM bodies b
JOIN bodies c ON b.parent_id = c.id
WHERE c.system_id = c.id AND c.parent_id = 0
AND (c.spectral = '' OR c.spectral IS NULL)
AND b.spectral != '' AND b.spectral IS NOT NULL
ORDER BY name;
```

### Routes Table Structure

- `id`: Unique route ID
- `name`: Route name (optional)
- `count`: Number of waypoints
- `round_trip`: 1 if bidirectional
- `visible`: 1 if shown
- `red, green, blue`: RGB color values (0-1 scale)
- `line_width`: Line thickness
- `line_style`: Line pattern
- `line_stipple`: Stipple pattern

Routes DO NOT have explicit start/end body references in the schema—waypoints reference routes and optionally reference bodies.

### Known Issues & Quirks

1. **Temperature Field Zero:** Many bodies have `temp = 0` even with valid spectral types. Astrosynthesis may calculate on-demand rather than store.

2. **No Foreign Keys:** Despite relational structure, SQLite foreign key constraints are not defined. Maintaining referential integrity is application responsibility.

3. **Optional Fields:** Many columns allow NULL or 0 values; must check for actual vs. placeholder values.

4. **Component Stars Same Position:** In multi-star systems, component stars share the container's position (no barycenter offset in the database). Orbital positions would need external calculation.

5. **Hierarchical Ambiguity:** Planets can orbit system container OR directly orbit component star. Must check parent_id carefully.

### Exploring the Schema (Revised Process)

1. **Open .AstroDB with SQLite Browser**
   ```sql
   SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;
   ```

2. **Examine Bodies Table Structure**
   ```sql
   PRAGMA table_info(bodies);
   ```

3. **Count Stars by Type**
   ```sql
   SELECT CASE WHEN spectral != '' THEN 'Star' ELSE 'Container/Other' END,
          COUNT(*) FROM bodies WHERE system_id = id AND parent_id = 0
   GROUP BY spectral != '';
   ```

4. **Sample Multi-Star System**
   ```sql
   SELECT * FROM bodies WHERE system_id = 552 ORDER BY parent_id;
   -- Should show Aorimo container + Aorimo A + Aorimo B
   ```

5. **Validate Extract Queries**
   - Run single-star extract: should return 246 rows
   - Run multi-star extract: should return 236 rows
   - Total should be 482 actual stars

## Data Conversion Considerations

### Coordinate System Conversion

When importing real astronomical data or exporting to other systems:

**Galactic to Astrosynthesis Coordinates:**
- Astrosynthesis uses a rotated coordinate system
- Conversion required for accurate 3D positioning
- Tools available on community sites (Evil Dr. Ganymede's Stellar Mapping page)

**Excel Conversion Spreadsheets:**
Available tools include:
- `coordinate_converter.xls`: Converts between RA/Dec and Galactic Lat/Lon
- `bulk_converter.xls`: Batch conversion for multiple stars
- Provides both Galactic XYZ and Astrosynthesis XYZ output

### Real Astronomical Data

**Compatible Datasets:**
- RECONS (nearby stars)
- DENSE catalog
- CTIOPI data
- Hipparcos catalog
- Gliese catalog

**Data Sources:**
Community members have created pre-converted datasets available in both:
- Galactic XYZ format (for scientific/general use)
- Astrosynthesis XYZ format (for import to Astrosynthesis)

## Plugin System & Scripting

### AstroScript API

**Capabilities:**
- Create custom plugins
- Automate repetitive tasks
- Generate custom displays
- Modify world generation algorithms
- Calculate distances between stars
- Custom route coloring for different empires
- Dynamic label visibility based on object properties

**Documentation:**
- Version 2 API: http://www.nbos.com/support/docs/AlienAPI/
- Version 3 API: Not fully documented (largely compatible with v2)
- Community-created plugins available on NBOS forums

**Example Use Cases:**
- "Generate Contents and Keep Name" plugin: Auto-generates system contents for imported star lists
- Custom distance calculators
- Batch processing of multiple stars
- Realistic stellar type distribution controls

## Integration with Other Tools

### Fractal Mapper Integration
- Export planetary surface maps from Astrosynthesis
- Import into Fractal Mapper for detailed world mapping
- Add cities, boundaries, terrain features in FM

### Fractal Terrains Integration
- Use Fractal Terrains as planet generation engine
- Generate and edit planetary surfaces
- Export to Campaign Cartographer

### Character Sheets (NBOS)
- Can attach character sheets to celestial bodies
- Add game-specific stats beyond default properties
- Useful for RPG campaigns

## Best Practices

### For Data Extraction:

1. **Make Backups:** Always work on copies of .AstroDB files
2. **Test First:** Create small test sectors to understand structure
3. **Document Schema:** Record your findings about table structures
4. **Validate Data:** After extraction, verify data integrity
5. **Use Transactions:** When writing to SQLite, use transactions for data integrity

### For Data Migration:

1. **Start Simple:** Begin with star positions only
2. **Incremental Addition:** Add complexity gradually (planets, then moons, etc.)
3. **Maintain Hierarchy:** Preserve parent-child relationships
4. **Test in Astrosynthesis:** Verify imported data displays correctly
5. **Keep Source Data:** Maintain original export files

### For Development:

1. **Use SQLite Browser:** Essential for understanding structure
2. **Export Examples:** Create diverse test sectors and export to learn format
3. **Community Resources:** Check NBOS forums for community solutions
4. **Version Control:** Track .AstroDB files in version control for experiments

## Common Pitfalls

### Coordinate System Confusion
**Problem:** Using Galactic XYZ coordinates directly in Astrosynthesis
**Solution:** Always convert to Astrosynthesis coordinate system for imports

### CSV Import Limitations
**Problem:** Expecting CSV import to handle complete systems
**Solution:** Use XML or direct database access for detailed data

### Missing Documentation
**Problem:** Lack of official schema documentation
**Solution:** Reverse-engineer through experimentation and XML exports

### ID Management
**Problem:** Conflicting or duplicate IDs when writing directly to database
**Solution:** Query for max ID values and increment properly; use SQLite auto-increment features

### Referential Integrity
**Problem:** Orphaned records (planets without parent stars, etc.)
**Solution:** Maintain proper foreign key relationships; test by opening in Astrosynthesis

## Resources

### Official Sources
- NBOS Software: https://www.nbos.com/products/astrosynthesis
- NBOS Forums: https://forum.nbos.com
- Plugin Repository: http://www.nbos.com/nox/index.php

### Community Resources
- Evil Dr. Ganymede's Stellar Mapping: http://evildrganymede.net/wp/rpgs/stellar-mapping/
- Pre-converted stellar databases
- Coordinate conversion tools
- Realistic stellar data for sci-fi campaigns

### Tools
- DB Browser for SQLite: https://sqlitebrowser.org/
- SQLite Documentation: https://www.sqlite.org/docs.html
- Various language-specific SQLite libraries

## Version Compatibility Notes

### Forward Compatibility
- v2 files cannot be directly opened in v3
- Must export from v2 and import to v3
- Some v2 plugins may work with v3 (not guaranteed)

### Backward Compatibility
- v3 files cannot be opened in v2
- XML export from v3 may include fields not understood by v2
- API differences between versions

### Cross-Platform
- SQLite is cross-platform compatible
- .AstroDB files can be read on Windows, Mac, Linux
- Astrosynthesis application is Windows-only (may work under Wine/CrossOver on Mac/Linux)

## Star Extraction Implementation

### Overview
A complete star extraction implementation has been created for SolarViewer that handles both single-star and multi-star systems from Astrosynthesis .AstroDB files.

### Data Extraction Results

**From TotalSystem.AstroDB:**
- Single-star systems: 246 actual stars
- Multi-star component stars: 236 actual stars (in 103 containers)
- **Total: 482 stars** (not the apparent 349 system records)

### Rust Implementation (src/extraction/)

**Star Struct:**
```rust
pub struct Star {
    pub id: i32,
    pub name: String,
    pub spectral_type: String,      // Astronomical classification
    pub radius_solar: f64,           // In solar radii
    pub mass_solar: f64,             // In solar masses
    pub luminosity_solar: f64,       // In solar luminosities
    pub temperature_k: f64,          // In Kelvin
    pub x: f64,                      // Star's position
    pub y: f64,
    pub z: f64,
    pub system_name: Option<String>, // Container name if multi-star
    pub system_x: f64,               // System container position
    pub system_y: f64,
    pub system_z: f64,
}
```

**StarReader Implementation:**
```rust
impl StarReader {
    pub fn new(db_path: &str) -> Result<Self> { }

    pub fn read_all_stars(&self) -> Result<Vec<Star>> {
        // Query 1: Single-star systems (246)
        // Query 2: Multi-star components (236)
        // Returns sorted combined list (482 total)
    }

    pub fn count_stars(&self) -> Result<i64> {
        // Returns 482 (single + multi components)
    }
}
```

**CSV Export Format:**
```csv
Name,Spectral Type,Radius (Solar),Mass (Solar),Luminosity (Solar),Temperature (K),Star X,Star Y,Star Z,System Name,System X,System Y,System Z
"Amateru","G3V",0.959796,0.95,0.835666,0.0,35.8,-1.2,0.9,"",35.8,-1.2,0.9
"Aorimo A","G5V",0.919166,0.9,0.691590,0.0,-35.4,6.5,-1.2,"Aorimo",-35.4,6.5,-1.2
"Aorimo B","K9V",0.619855,0.55,0.123387,0.0,-35.4,6.5,-1.2,"Aorimo",-35.4,6.5,-1.2
```

### CLI Integration

**Extract Command:**
```bash
solarviewer extract --file TotalSystem.AstroDB --output stars.csv
```

Output:
```
✓ Star extraction complete!
  Stars extracted: 482
  CSV file: stars.csv
```

**Diagnostic Command (future):**
```bash
solarviewer multistar --file TotalSystem.AstroDB
```

Displays:
- All multi-star containers
- Component stars for each container
- System statistics

### Validation Tools

Three Python scripts created for verification and testing:

1. **analyze_multistar.py**
   - Finds all multi-star containers
   - Lists component stars
   - Shows system positions

2. **analyze_empty_systems.py**
   - Detailed breakdown of system types
   - Body composition analysis
   - Hierarchical relationships

3. **export_stars_full.py**
   - Complete star extraction to CSV
   - Proper CSV escaping
   - System context included

### Output Files

- **stars_complete.csv** - All 482 stars with proper multi-star handling (validation reference)
- **MULTISTAR_SUMMARY.md** - Technical documentation of structure and findings
- **MULTI_STAR_NEXT_STEPS.md** - Enhancement suggestions and usage patterns

### Key Learnings

1. **Spectral Type as Discriminator**
   - Empty/NULL spectral type = system container
   - Non-empty spectral type = actual star

2. **Position Handling**
   - Multi-star components share container position
   - Component offsets not stored in database
   - Need external calculation for orbital positions

3. **Hierarchical Navigation**
   - parent_id = 0: Root bodies (stars or containers)
   - parent_id = star/container: Planets, moons, components
   - system_id: Always points to defining star (for single) or container (for multi)

4. **Data Integrity**
   - No enforced foreign keys—application responsible
   - Some fields are 0 for placeholder values
   - Temperature often 0 despite valid spectral type

5. **Route Data**
   - Routes don't explicitly reference bodies
   - Waypoints connect to routes
   - Waypoints optionally reference bodies
   - Routes exist between any bodies

### PostgreSQL Schema Design

Recommended mapping for SolarViewer's PostgreSQL:

```sql
CREATE TABLE star_systems (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER,
    original_id INTEGER,
    name TEXT NOT NULL,
    system_type TEXT,  -- 'single' or 'multi'
    num_components INTEGER,
    position GEOMETRY(PointZ, 4326),
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION
);

CREATE TABLE stars (
    id SERIAL PRIMARY KEY,
    system_id INTEGER REFERENCES star_systems(id),
    source_file_id INTEGER,
    original_id INTEGER,
    name TEXT NOT NULL,
    spectral_type TEXT,
    radius_solar DOUBLE PRECISION,
    mass_solar DOUBLE PRECISION,
    luminosity_solar DOUBLE PRECISION,
    temperature_k DOUBLE PRECISION,
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION,
    component_letter TEXT,  -- 'A', 'B', 'C' for multi-star
    position GEOMETRY(PointZ, 4326)
);
```

## For SolarViewer Project

### Completed Work

1. **Schema Discovery Phase:** ✅
   - Reverse-engineered complete 14-table schema
   - Generated comprehensive SCHEMA.md documentation
   - Tested with TotalSystem.AstroDB (627 bodies, 349 apparent systems, 482 actual stars)

2. **Star Extraction:** ✅
   - Implemented StarReader for both single and multi-star systems
   - CSV export with complete stellar data
   - Proper handling of multi-star containers
   - Python validation tools

3. **Data Analysis:** ✅
   - Identified multi-star system architecture
   - Determined spectral type as key discriminator
   - Verified complete dataset statistics

### Recommended Approach for Next Phases

1. **PostgreSQL Setup (Phase 2):**
   - Use provided schema design
   - Create PostGIS geometry columns for 3D queries
   - Build data importer from Astrosynthesis

2. **Route Integration:**
   - Extract and import routes
   - Build waypoint path visualization
   - Calculate distances between multi-star systems

3. **2D Visualization:**
   - Decide on multi-star component rendering
   - Test PCA projection with binary/triple systems
   - Handle overlap resolution for close components

### Benefits of This Approach

- Learn from mature data model
- Import existing Astrosynthesis sectors
- Provide migration path for Astrosynthesis users
- Understand established stellar cartography patterns
- Leverage community knowledge and datasets

## Conclusion

Astrosynthesis 3.0's use of SQLite for its .AstroDB format provides excellent opportunities for data extraction, migration, and integration. While official documentation is limited, the standard SQLite format ensures complete programmatic access to all stored data. Combined with XML export capabilities and community resources, developers have multiple pathways to work with Astrosynthesis data in their own applications.

The lack of official schema documentation requires some reverse-engineering effort, but the payoff is complete access to a mature stellar cartography data model that has been refined through years of use by the sci-fi gaming and worldbuilding community.
