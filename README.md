# SolarViewer

A Rust-based tool for extracting, storing, and visualizing stellar cartography data from Astrosynthesis save files.

## Overview

SolarViewer reads Astrosynthesis .AstroDB files (SQLite databases), extracts stellar data including 3D coordinates, system hierarchies, and route information, then stores it in PostgreSQL with PostGIS for advanced spatial queries. It includes intelligent 2D projection algorithms to create readable maps from 3D stellar data.

## Features (Planned)

- **Schema Discovery**: Automatically explore and document Astrosynthesis database schemas
- **Data Extraction**: Read .AstroDB SQLite files and extract hierarchical stellar data
- **PostgreSQL Storage**: Store multiple sectors with PostGIS spatial indexing
- **2D Projection**: Generate readable 2D maps from 3D coordinates using hybrid layout algorithms
- **Distance Preservation**: Annotate maps with real 3D distances
- **Multiple Export Formats**: SVG, PNG, and more

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
git clone https://github.com/yourusername/SolarViewer.git
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

### Algorithms

- `pca`: Fast PCA projection (deterministic, good for overview)
- `force`: Force-directed graph layout (organic, emphasizes connectivity)
- `mds`: Multidimensional Scaling (best distance preservation)
- `hybrid`: PCA + overlap resolution + force refinement (recommended)

## Project Status

**Current Phase**: Schema Discovery and Documentation

See [PROJECT.md](PROJECT.md) for detailed architecture, implementation phases, and design decisions.

## Development

### Project Structure

```
solarviewer/
├── src/
│   ├── main.rs              # CLI interface
│   ├── schema/              # Schema exploration
│   ├── extraction/          # SQLite reading
│   ├── storage/             # PostgreSQL operations
│   ├── projection/          # Layout algorithms
│   └── visualization/       # Rendering
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
