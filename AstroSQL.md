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

## Known Schema Information

### Limitations
**CRITICAL:** The Astrosynthesis database schema is not officially documented. Users have reported limited support from NBOS for advanced database access features.

### What We Know

**Table Structure:**
- Multiple star systems: Position values of component stars are in AU, relative to the system center
- Orbital parameters: Available fields exist but Astrosynthesis doesn't fully track stellar orbital information
- Custom fields: Can be added and displayed in system data views
- Referential integrity: Parent-child relationships maintained via foreign keys

**Coordinate Fields:**
- X, Y, Z positions stored in light-years
- Distance from Sol calculated and stored
- Subsector grid positions

**Body Types:**
The CSV importer recognizes limited types:
- "Star"
- "White Dwarf"  
- "Multiple" (for multiple star systems)

However, the full database likely includes many more types (planets, moons, asteroids, stations, nebulae, etc.)

**Known Issues:**
- Users report the built-in CSV importer is very limited
- No official data dictionary available
- Schema may change between versions
- Some fields may be version 3.0 specific and not in XML export

### Exploring the Schema

**Recommended Process:**
1. Create a small test sector with various objects:
   - Single stars
   - Multiple star systems
   - Planets with different types (terrestrial, gas giant, etc.)
   - Moons
   - Asteroid belts
   - Routes between systems

2. Export to XML to see the data structure

3. Open .AstroDB in SQLite browser and examine:
   ```sql
   -- List all tables
   SELECT name FROM sqlite_master WHERE type='table';
   
   -- View table structure
   PRAGMA table_info(table_name);
   
   -- Sample data from each table
   SELECT * FROM table_name LIMIT 5;
   ```

4. Document the schema for your own use

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

## For SolarCrafter Design Project

Given your specific use case:

### Recommended Approach:

1. **Schema Discovery Phase:**
   - Create diverse test sectors in Astrosynthesis
   - Export to XML for structure understanding
   - Open .AstroDB files with SQLite browser
   - Document complete schema

2. **Rust SQLite Access:**
   ```rust
   // Use rusqlite for direct access
   use rusqlite::{Connection, Result};
   
   pub struct AstroSynthesisImporter {
       conn: Connection,
   }
   
   impl AstroSynthesisImporter {
       pub fn new(path: &str) -> Result<Self> {
           Ok(Self {
               conn: Connection::open(path)?
           })
       }
       
       pub fn extract_stars(&self) -> Result<Vec<StarData>> {
           // Implementation based on discovered schema
       }
   }
   ```

3. **PostgreSQL Migration:**
   - Map Astrosynthesis schema to your PostgreSQL schema
   - Handle coordinate system conversion
   - Preserve hierarchical relationships
   - Add PostGIS spatial capabilities

4. **Benefits:**
   - Learn from mature data model
   - Import existing Astrosynthesis sectors
   - Provide migration path for Astrosynthesis users
   - Understand established stellar cartography patterns

### Schema Mapping Considerations:

**Astrosynthesis → SolarCrafter Design:**
- Stars → Your `stars` table (with coordinate conversion)
- Multiple star systems → Handle as related star records
- Planets → Your `planets` table with enhanced properties
- Moons → Your `moons` table
- Routes → Could become your navigation/route planning feature
- Subsectors → Could become spatial indexing/regions

Your PostgreSQL + PostGIS setup will give you advantages:
- Better spatial queries
- More robust relational integrity
- Custom extensions
- Modern tooling
- Server/client architecture flexibility

## Conclusion

Astrosynthesis 3.0's use of SQLite for its .AstroDB format provides excellent opportunities for data extraction, migration, and integration. While official documentation is limited, the standard SQLite format ensures complete programmatic access to all stored data. Combined with XML export capabilities and community resources, developers have multiple pathways to work with Astrosynthesis data in their own applications.

The lack of official schema documentation requires some reverse-engineering effort, but the payoff is complete access to a mature stellar cartography data model that has been refined through years of use by the sci-fi gaming and worldbuilding community.
