# StellarForge - Modern Data Structure for Stellar Cartography

## Overview

StellarForge is a comprehensive, modern data structure system for representing galaxies, star systems, planets, and other celestial bodies. It builds upon concepts from Astrosynthesis while providing a more flexible, container-based architecture with robust support for coordinate frames, orbital mechanics, and hierarchical organization.

## Key Features

- **Container-Based Hierarchy**: Everything is a container that can hold other objects
- **Flexible Coordinate Systems**: Support for multiple reference frames with transformations
- **Advanced Orbital Mechanics**: Keplerian, N-body, and custom motion models
- **Physical Properties**: Detailed modeling of stars, planets, moons, stations, etc.
- **Association System**: Tags and relationships for political, economic, and other groupings
- **Builder Pattern**: Easy construction of complex systems
- **Persistence**: JSON and binary serialization, PostgreSQL/PostGIS support

## Architecture

### Core Concepts

1. **StellarContainer**: Abstract container that can hold stellar bodies
2. **StellarBody**: Base type for all celestial objects (stars, planets, moons, stations)
3. **Frame**: Coordinate reference frame with transformations
4. **MotionModel**: How objects move through space (orbital, free, scripted)
5. **Physical**: Physical properties specific to each body type
6. **Association**: Relationships between objects (political, trade, etc.)

### Module Structure

```
stellar_forge/
├── core.rs          # Core traits and types
├── containers.rs    # Container implementations (Galaxy, System, Fleet)
├── bodies.rs        # Stellar body types and implementations
├── frames.rs        # Coordinate frames and transformations
├── motion.rs        # Orbital mechanics and motion models
├── physical.rs      # Physical properties (mass, radius, atmosphere)
├── associations.rs  # Tags and relationship system
├── services.rs      # Service layer for operations
├── builders.rs      # Builder patterns for easy construction
└── storage.rs       # Serialization and database persistence
```

## Usage Examples

### Creating a Simple Star System

```rust
use stellar_forge::builders::SystemBuilder;

let system = SystemBuilder::new("Alpha Centauri")
    .at_position(1.34, 0.0, 0.0)  // Position in parsecs
    .with_star("G2V")
    .with_planet(
        PlanetBuilder::terrestrial("Proxima b", 0.05)
            .with_mass_and_radius(1.27, 1.08)
            .with_earth_like_atmosphere()
    )
    .build();
```

### Creating a Binary Star System

```rust
let binary = SystemBuilder::new("Binary System")
    .with_binary_stars("G2V", "K1V", 23.0)  // 23 AU separation
    .with_planet(
        PlanetBuilder::terrestrial("Planet", 1.5)
            .with_moon(MoonBuilder::new("Moon"))
    )
    .build();
```

### Creating Sol System

```rust
use stellar_forge::builders::create_sol_like_system;

let sol = create_sol_like_system()
    .at_position(8000.0, 0.0, 0.0)  // 8 kpc from galactic center
    .build();
```

### Building a Galaxy

```rust
use stellar_forge::builders::GalaxyBuilder;

let galaxy = GalaxyBuilder::new("Milky Way")
    .with_size(100000.0, 100000.0, 1000.0)  // Size in light-years
    .with_random_systems(1000, seed)
    .with_system(sol_system)
    .with_system(alpha_centauri)
    .build();
```

### Using the Service Layer

```rust
use stellar_forge::services::StellarForgeService;

let mut service = StellarForgeService::new("My Galaxy");
service.initialize()?;

// Add systems
service.add_system(system)?;

// Query nearby systems
let nearby = service.query_systems_in_range(
    Vec3::new(0.0, 0.0, 0.0),
    10.0  // Within 10 light-years
);

// Find bodies with specific tags
let habitable = service.find_bodies_with_tag(&Tag::from("habitable"));
```

### Persistence

```rust
use stellar_forge::storage::{StellarForgeDataset, FileStorage};

// Save to JSON
let dataset = StellarForgeDataset::new(galaxy);
FileStorage::save_json(&dataset, "galaxy.json")?;

// Load from JSON
let loaded = FileStorage::load_json("galaxy.json")?;
```

## Data Model Details

### Coordinate Systems

- **Galactic**: Top-level coordinate system (ICRS/J2000 aligned)
- **Barycentric**: System center of mass
- **Stellar**: Star-centered frame
- **Planetary**: Planet-centered (non-rotating or rotating)
- **Local**: Station/vehicle local frames

### Motion Models

- **Keplerian**: Classical two-body orbits
- **Free**: Constant velocity (for rogue objects)
- **TableEphemeris**: High-precision sampled positions
- **Scripted**: Custom motion behaviors
- **TwoBody**: Two-body with perturbations
- **NBody**: Full N-body integration

### Physical Properties

#### Stars
- Mass, radius, luminosity, temperature
- Spectral type and class (O, B, A, F, G, K, M)
- Age, metallicity, magnetic field
- Rotation velocity

#### Planets
- Mass, radius, density, gravity
- Atmosphere (composition, pressure, breathability)
- Surface water percentage
- Habitability score
- Population

#### Stations
- Mass, cargo capacity
- Docking ports
- Population capacity
- Power generation
- Station type (research, military, trading, etc.)

### Associations & Tags

#### Tag Categories
- **Exploration**: unexplored, surveyed, mapped, colonizable
- **Resources**: resource_rich, mining_site, rare_minerals
- **Danger**: hazardous, radiation, unstable, quarantine
- **Civilization**: inhabited, colony, outpost, capital, trade_hub
- **Strategic**: choke_point, border_system, disputed, neutral_zone

#### Association Types
- **Political**: Nation membership, governance
- **Economic**: Trade routes, economic zones
- **Military**: Alliances, defense pacts
- **Scientific**: Research consortiums
- **Cultural**: Cultural groups, religious affiliations
- **Historical**: Discovery records, historical events

## Advanced Features

### Hierarchical Containers

Any body can be a container:
- Stars contain planets
- Planets contain moons and stations
- Stations contain docked vehicles
- Belts contain asteroids and mining stations

### Time-Aware System

- All states have epochs
- Motion models propagate to any time
- Associations can have time ranges
- Historical tracking of changes

### Spatial Queries (with PostGIS)

```sql
-- Find bodies within radius
SELECT * FROM bodies
WHERE ST_DWithin(position, point, radius);

-- Find nearest neighbors
SELECT * FROM bodies
ORDER BY position <-> point
LIMIT 10;
```

## Command-Line Usage

### Create a new galaxy
```bash
solarviewer create-galaxy --name "My Galaxy" --systems 100 --output galaxy.json
```

### Create Sol system
```bash
solarviewer create-sol --output sol.json
```

## Integration with Astrosynthesis

StellarForge can import data from Astrosynthesis:

```rust
use stellar_forge::storage::ImportExport;

let galaxy = ImportExport::import_from_astrosynthesis("TotalSystem.AstroDB")?;
```

## Performance Considerations

- Lazy evaluation of orbital positions
- Spatial indexing for queries
- Frame transformation caching
- Hierarchical bounds for culling
- Optional async database operations

## Future Enhancements

- [ ] Web API for remote access
- [ ] 3D visualization integration
- [ ] Route planning algorithms
- [ ] Trade network simulation
- [ ] Population dynamics
- [ ] Resource management
- [ ] Conflict simulation
- [ ] Procedural system generation
- [ ] Real astronomical data import

## License

Part of the SolarViewer project. See main LICENSE file.