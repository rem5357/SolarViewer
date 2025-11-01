# SolarViewer & StellarForge

A comprehensive Rust-based toolkit for extracting, storing, and visualizing stellar cartography data from Astrosynthesis save files, featuring the modern StellarForge data structure system.

## Overview

SolarViewer reads Astrosynthesis .AstroDB files (SQLite databases), extracts stellar data including 3D coordinates, system hierarchies, and route information, then stores it in PostgreSQL with PostGIS for advanced spatial queries. It includes intelligent 2D projection algorithms to create readable maps from 3D stellar data.

**New: StellarForge** - A modern stellar cartography data structure system that improves upon Astrosynthesis with:
- Container-based polymorphic architecture (everything can contain everything)
- Proper IAU astronomical coordinate systems
- PostgreSQL/PostGIS spatial database backend
- Political influence zones with spatial mathematics
- Advanced routing and pathfinding capabilities

## Features

### Implemented
- **Schema Discovery**: ✅ Automatically explore and document Astrosynthesis database schemas
- **2D Star Map Visualization**: ✅ Generate PNG maps with spectral colors and glows
- **StellarForge Data Structure**: ✅ Modern container-based architecture with proper IAU coordinates
- **PostgreSQL/PostGIS Integration**: ✅ Spatial database with 3D indexing and queries
- **Political Influence Zones**: ✅ 3D territories with strength-based falloff
- **Advanced Spatial Queries**: ✅ Chokepoints, contested systems, safe routes

### In Progress
- **Data Extraction**: Read .AstroDB SQLite files and migrate to PostgreSQL
- **Text Rendering**: Star names and distance labels on maps
- **Route Visualization**: Trade routes and connections on 2D maps

### Planned
- **Multiple Export Formats**: SVG and other formats beyond PNG
- **Temporal Data**: Historical borders and territory changes
- **Economic Simulation**: Trade flows and commodity modeling

## Architecture

### Data Flow
```
Astrosynthesis (.AstroDB)
    ↓ [SQLite Reader]
PostgreSQL + PostGIS
    ↓ [Layout Engine: PCA + Force-Directed]
2D Map (SVG/PNG)
```

### Key Technologies
- **Rust**: Performance and safety
- **rusqlite**: Read Astrosynthesis SQLite databases
- **PostgreSQL + PostGIS**: Persistent storage with spatial queries
- **petgraph**: Graph algorithms for layout and routing
- **nalgebra**: Linear algebra for PCA projections

## Installation

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **PostgreSQL with PostGIS**:
   ```bash
   # Install PostgreSQL
   # Then add PostGIS extension
   psql -d your_database -c "CREATE EXTENSION postgis;"
   ```

### Build from Source

```bash
git clone https://github.com/rem5357/SolarViewer.git
cd SolarViewer
cargo build --release
```

## Usage

### Explore Schema

Discover and document the structure of an Astrosynthesis database:

```bash
solarviewer schema --file sector.AstroDB --output docs/SCHEMA.md
```

### Import Data

Extract data from an Astrosynthesis file and load into PostgreSQL:

```bash
solarviewer import \
  --file sector.AstroDB \
  --name "Rimward Sector" \
  --database postgresql://localhost/solarviewer
```

### Generate Map

Create a 2D visualization from stored data:

```bash
solarviewer map \
  --name "Rimward Sector" \
  --output map.svg \
  --algorithm hybrid \
  --database postgresql://localhost/solarviewer
```

### StellarForge CLI

The new StellarForge system includes a comprehensive CLI for database operations:

```bash
# Initialize the database with PostGIS
stellarforge init

# Create a new galaxy session
stellarforge session create --name "My Galaxy"

# Add star systems
stellarforge system add --session-id <UUID> --name "Sol" --x 0 --y 0 --z 0

# Create political entities and influence zones
stellarforge political create --session-id <UUID> --name "Federation"
stellarforge political influence --session-id <UUID> --entity-id <UUID> --base-radius 20

# Analyze spatial relationships
stellarforge analyze chokepoints --session-id <UUID>
stellarforge analyze contested --session-id <UUID>
```

### Algorithms

- `pca`: Fast PCA projection (deterministic, good for overview)
- `force`: Force-directed graph layout (organic, emphasizes connectivity)
- `mds`: Multidimensional Scaling (best distance preservation)
- `hybrid`: PCA + overlap resolution + force refinement (recommended)

## Project Status

**Current Phase**: StellarForge Implementation Complete, Integration In Progress

### Key Documentation
- [PROJECT.md](PROJECT.md) - Detailed architecture, implementation phases, and design decisions
- [STELLARFORGE.md](STELLARFORGE.md) - Initial StellarForge design specification
- [STELLARFORGE_DATABASE.md](STELLARFORGE_DATABASE.md) - Complete database documentation
- [STELLARFORGE_SUMMARY.md](STELLARFORGE_SUMMARY.md) - Implementation summary
- [COORDINATE_SYSTEMS.md](COORDINATE_SYSTEMS.md) - IAU coordinate system documentation

## Development

### Project Structure

```
solarviewer/
├── src/
│   ├── main.rs              # CLI interface
│   ├── bin/
│   │   └── stellarforge.rs  # StellarForge CLI binary
│   ├── schema/              # Schema exploration
│   ├── extraction/          # SQLite reading
│   ├── visualization/       # 2D rendering with spectral colors
│   └── stellar_forge/       # Modern data structure system
│       ├── core.rs          # Traits and types
│       ├── coordinates.rs   # IAU coordinate systems
│       ├── containers.rs    # Container implementations
│       ├── database/        # PostgreSQL/PostGIS layer
│       └── cli.rs           # CLI operations
├── sql/                     # Database schema files
│   ├── 01_create_database.sql
│   ├── 02_session_tables.sql
│   ├── 03_stellar_tables.sql
│   ├── 04_political_tables.sql
│   ├── 05_routes_tables.sql
│   └── 06_groups_sectors_tables.sql
├── tests/                   # Integration tests
└── docs/                    # Generated documentation
```

### Running Tests

```bash
cargo test
```

### Benchmarks

```bash
cargo bench
```

## Configuration

Create `solarviewer.toml` in your working directory:

```toml
[database]
postgres_url = "postgresql://user:pass@localhost/solarviewer"

[layout]
algorithm = "hybrid"
min_separation_pixels = 20

[visualization]
show_distance_labels = true
show_z_coordinates = true
depth_color_encoding = true
```

## Background

### Astrosynthesis

Astrosynthesis is a 3D stellar cartography application by NBOS Software. Version 3.0 uses SQLite databases (.AstroDB files) to store:

- Star systems with 3D coordinates
- Planets and moons with orbital parameters
- Routes between systems
- Subsector organization

See [AstroSQL.md](AstroSQL.md) for detailed technical information.

### 2D Projection Challenge

Projecting 3D stellar data to 2D requires balancing multiple constraints:
- Avoid visual overlaps
- Preserve neighborhood relationships
- Maintain readability
- Show accurate distances

SolarViewer uses a hybrid approach combining PCA projection, collision resolution, and force-directed refinement. See [StarMap2D_Visualization.md](StarMap2D_Visualization.md) for algorithm details.

## Resources

- [NBOS Software - Astrosynthesis](https://www.nbos.com/products/astrosynthesis)
- [DB Browser for SQLite](https://sqlitebrowser.org/)
- [PostgreSQL + PostGIS Documentation](https://postgis.net/)

## License

MIT OR Apache-2.0

## Contributing

Contributions welcome! Please open an issue or pull request.

## Acknowledgments

- NBOS Software for Astrosynthesis
- Community resources from stellar mapping enthusiasts
- Rust community for excellent libraries
