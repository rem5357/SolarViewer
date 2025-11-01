# Coordinate Systems in StellarForge

## Overview

StellarForge uses **proper astronomical coordinate systems** that follow International Astronomical Union (IAU) standards, unlike Astrosynthesis which uses a non-standard system. This ensures compatibility with real astronomical data and tools.

## Standard Coordinate Systems Implemented

### 1. IAU Galactic Coordinates (Primary System)

This is the standard coordinate system used by astronomers worldwide:

- **Origin**: The Sun (Sol)
- **X-axis**: Points toward the Galactic Center (l=0°, b=0°)
- **Y-axis**: Points in the direction of galactic rotation (l=90°, b=0°)
- **Z-axis**: Points toward the North Galactic Pole (b=90°)

**Spherical representation**:
- `l` (longitude): 0° to 360°, measured in the galactic plane
- `b` (latitude): -90° to +90°, measured from the galactic plane
- `d` (distance): Distance from Sun in parsecs or light-years

**Key reference points**:
- Galactic Center: l=0°, b=0°, d≈8,178 pc
- North Galactic Pole: l=arbitrary, b=90°
- Anti-center: l=180°, b=0°

### 2. ICRS (International Celestial Reference System)

The modern standard replacing J2000.0:

- **Origin**: Solar System Barycenter (or Earth/Sun for nearby stars)
- **Axes**: Fixed relative to distant quasars
- **Essentially**: J2000.0 equatorial coordinates

**Representation**:
- Right Ascension (RA): 0h to 24h
- Declination (Dec): -90° to +90°
- Distance: parsecs or light-years

### 3. Equatorial Coordinates (J2000.0)

Traditional astronomical coordinates:

- **Origin**: Earth (or Sun for stellar work)
- **Reference plane**: Earth's equator
- **Zero point**: Vernal equinox at J2000.0 epoch

## Transformation Between Systems

### Galactic ↔ Equatorial

StellarForge implements the standard IAU transformation matrix:

```rust
// Standard transformation matrix (Reid & Brunthaler 2004)
// Equatorial to Galactic
T = [[-0.054876, -0.873437, -0.483835],
     [+0.494109, -0.444830, +0.746982],
     [-0.867666, -0.198076, +0.455984]]
```

### Example Conversions

```rust
use stellar_forge::coordinates::{GalacticCoordinates, CoordinateTransform};

// Alpha Centauri in galactic coordinates
let alpha_cen = GalacticCoordinates::new(315.8, -0.68, 1.34);

// Convert to equatorial
let eq = CoordinateTransform::galactic_to_equatorial(alpha_cen);

// Convert to cartesian (X toward GC, Y toward rotation, Z toward NGP)
let cart = alpha_cen.to_cartesian();
```

## Astrosynthesis Coordinate Conversion

Astrosynthesis uses a non-standard coordinate system that needs conversion:

### Astrosynthesis System (Non-Standard)
- **X**: Points "right" on the 2D map view
- **Y**: Points "up" on the 2D map view
- **Z**: Points "out" of the screen (toward viewer)
- **Units**: Light-years
- **Origin**: Arbitrary (often sector center)

### Conversion to IAU Galactic

```rust
use stellar_forge::coordinates::CoordinateTransform;

// Convert Astrosynthesis coordinates to proper galactic
let galactic = CoordinateTransform::astrosynthesis_to_galactic(
    astro_x,  // Astrosynthesis X in light-years
    astro_y,  // Astrosynthesis Y in light-years
    astro_z   // Astrosynthesis Z in light-years
);

// Or when importing a system
let system = StarSystem::from_astrosynthesis(
    "System Name",
    astro_x, astro_y, astro_z,
    star_body
);
```

The conversion approximately maps:
- Astro X → Galactic X (toward galactic center)
- Astro Y → Galactic Z (toward NGP)
- Astro Z → Galactic -Y (opposite rotation direction)

**Note**: The exact transformation may need calibration based on known star positions in your Astrosynthesis files.

## Reference Star Positions

StellarForge includes correct positions for reference stars:

| Star | l (°) | b (°) | Distance (pc) | Notes |
|------|-------|-------|---------------|-------|
| Sol | 0 | 0 | 0 | By definition |
| Alpha Centauri | 315.8 | -0.68 | 1.34 | Nearest star system |
| Polaris | 123.3 | -17.1 | 132.6 | North Star |
| Vega | 67.4 | 19.2 | 7.68 | Bright nearby star |
| Betelgeuse | 199.8 | -8.96 | 168.1 | Red supergiant |
| Sagittarius A* | 0.0 | 0.0 | 8,178 | Galactic center |

## Units Convention

### Internal Storage (SI)
- **Distance**: meters
- **Velocity**: meters per second
- **Mass**: kilograms
- **Time**: seconds
- **Angles**: radians

### Display/Input Units
- **Stellar distances**: parsecs (pc) or light-years (ly)
- **Planetary distances**: Astronomical Units (AU)
- **Angles**: degrees for display, hours for RA
- **Galactic coordinates**: degrees for l and b

### Conversion Constants

```rust
1 parsec = 3.086e16 m = 3.26 light-years
1 light-year = 9.461e15 m = 0.3066 parsecs
1 AU = 1.496e11 m = 4.85e-6 parsecs
```

## Usage in StellarForge

### Creating Systems with Proper Coordinates

```rust
// Method 1: Direct galactic coordinates
let system = SystemBuilder::new("Kepler-452")
    .at_galactic(290.7, 47.5, 430.0)  // l=290.7°, b=47.5°, d=430 pc
    .with_star("G2V")
    .build();

// Method 2: Cartesian from Sun (parsecs)
let system = SystemBuilder::new("Alpha Centauri")
    .at_position_pc(1.34, 0.0, 0.0)  // 1.34 pc in X direction
    .with_binary_stars("G2V", "K1V", 23.0)
    .build();

// Method 3: Import from Astrosynthesis (auto-converts)
let system = StarSystem::from_astrosynthesis(
    "Imported System",
    10.5,   // Astro X in light-years
    -3.2,   // Astro Y in light-years
    1.7,    // Astro Z in light-years
    star
);
```

### Querying Systems

```rust
// Find systems within radius (using proper galactic coordinates)
let center = GalacticCoordinates::new(0.0, 0.0, 100.0).to_cartesian();
let nearby = galaxy.systems_within(center, 10.0 * Units::PARSEC);

// Get distance between systems
let dist = (system1.galactic_position() - system2.galactic_position()).norm();
let dist_ly = dist / Units::LIGHT_YEAR;
```

## Local Standard of Rest (LSR)

For proper motion studies, StellarForge can apply LSR corrections:

**Solar motion relative to LSR** (Schönrich et al. 2010):
- U = 11.1 km/s (toward galactic center)
- V = 12.24 km/s (in rotation direction)
- W = 7.25 km/s (toward NGP)

```rust
let velocity_lsr = CoordinateTransform::apply_lsr_correction(velocity_heliocentric);
```

## Coordinate Display Formatting

```rust
use stellar_forge::coordinates::CoordinateFormatter;

// Format for display
let gal = GalacticCoordinates::new(315.8, -0.68, 1.34);
println!("{}", CoordinateFormatter::format_galactic(&gal));
// Output: "l=315.80°, b=-0.68°, d=1.3 pc"

let eq = EquatorialCoordinates::new(14.66, -60.84, 1.34);
println!("{}", CoordinateFormatter::format_equatorial(&eq));
// Output: "RA 14h 39m 36.00s, Dec -60° 50' 24.0", d=1.3 pc"
```

## Benefits of Using Standard Coordinates

1. **Scientific Accuracy**: Compatible with real astronomical databases
2. **Tool Compatibility**: Works with standard astronomy software
3. **Real Data Integration**: Can import actual star catalogs
4. **Clear Documentation**: Well-defined transformations
5. **Future-Proof**: Follows international standards

## Migration from Astrosynthesis

When importing Astrosynthesis data:

1. Positions are automatically converted to IAU galactic coordinates
2. Original positions are preserved in `legacy_position` field
3. Systems can be queried using either coordinate system
4. Export functions can convert back if needed

```rust
// Import preserves both coordinate systems
let system = StarSystem::from_astrosynthesis(name, x, y, z, star);
let galactic_pos = system.galactic_position();      // IAU standard
let legacy_pos = system.legacy_position;            // Original Astro coords
```

## References

- IAU Resolution B2 (2000): Definition of galactic coordinate system
- Reid & Brunthaler (2004): Galactic coordinate transformation matrices
- Schönrich et al. (2010): Solar motion relative to LSR
- ICRS: International Celestial Reference System standards
- Astrosynthesis documentation (for legacy conversion)