# Astrosynthesis Database Schema

**Source File**: `TotalSystem.AstroDB`

**Generated**: 2025-10-30 11:17:24

**Total Tables**: 14

## Overview

This schema represents the internal structure of Astrosynthesis 3.0 save files, which are SQLite databases containing stellar cartography data. The database models a hierarchical astronomical system with stars, planets, moons, and other celestial bodies, along with routes between star systems and various rendering/display configurations.

## Key Concepts

### Coordinate System
- All spatial coordinates (x, y, z) are in **light-years** from sector origin
- Astrosynthesis uses a rotated coordinate system that differs from standard astronomical conventions
- Conversion may be needed for scientific applications

### Hierarchical Structure
- **Stars**: Top-level objects (system_id = id, parent_id = 0)
- **Planets**: Orbit stars (system_id = star.id, parent_id = star.id)
- **Moons**: Orbit planets (system_id = star.id, parent_id = planet.id)
- **Stations/Asteroids**: Can orbit any body

### Relationships (No Foreign Keys Enforced)
While the database lacks formal foreign key constraints, relationships exist via:
- `body.system_id` → `body.id` (star that defines the system)
- `body.parent_id` → `body.id` (orbital parent)
- `route_waypoints.route_id` → `routes.id`
- `route_waypoints.body_id` → `bodies.id`
- `atm_components.body_id` → `bodies.id`

## Table of Contents

- [atm_components](#atm-components)
- [bodies](#bodies)
- [config](#config)
- [custom_fields](#custom-fields)
- [fwe_color_groups](#fwe-color-groups)
- [resources](#resources)
- [route_waypoints](#route-waypoints)
- [routes](#routes)
- [sector_info](#sector-info)
- [sector_view_arrays](#sector-view-arrays)
- [sector_views](#sector-views)
- [subsectors](#subsectors)
- [surface_maps](#surface-maps)
- [system_data_config](#system-data-config)

---

## Summary Statistics

| Table | Rows | Columns | Foreign Keys |
|-------|------|---------|-------------|
| atm_components | 28 | 4 | 0 |
| bodies | 627 | 63 | 0 |
| config | 1 | 3 | 0 |
| custom_fields | 0 | 5 | 0 |
| fwe_color_groups | 0 | 11 | 0 |
| resources | 2 | 4 | 0 |
| route_waypoints | 90 | 7 | 0 |
| routes | 45 | 13 | 0 |
| sector_info | 1 | 36 | 0 |
| sector_view_arrays | 8 | 5 | 0 |
| sector_views | 4 | 12 | 0 |
| subsectors | 0 | 22 | 0 |
| surface_maps | 0 | 5 | 0 |
| system_data_config | 110 | 8 | 0 |

---

## Detailed Table Information

### atm_components

**Purpose**: Stores atmospheric gas composition for bodies with atmospheres

**Row Count**: 28

**Description**: This table defines the chemical composition of planetary atmospheres, linking gas types to celestial bodies and their relative percentages. Essential for habitability calculations and scientific realism.

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | body_id | INTEGER |  | - |  |
| 2 | gas | TEXT |  | - |  |
| 3 | percent | double |  | - |  |

#### Sample Data

| percent | id | body_id | gas |
|---|---|---|---|
| NULL | NULL | NULL | N2 |
| NULL | NULL | NULL | O2 |
| NULL | NULL | NULL | H2O |
| NULL | NULL | NULL | CO2 |
| NULL | NULL | NULL | O2 |

---

### bodies

**Purpose**: Core table containing all celestial objects (stars, planets, moons, stations, asteroids, etc.)

**Row Count**: 627

#### Key Fields Explained

- **Identification**:
  - `id`: Unique identifier for the body
  - `id_string`: GUID for cross-referencing
  - `name`: Display name of the celestial body

- **Hierarchy & Location**:
  - `system_id`: ID of the star system this body belongs to
  - `parent_id`: ID of the body this orbits (0 for stars)
  - `x, y, z`: Absolute 3D coordinates in light-years

- **Physical Properties**:
  - `type_id`: Numeric type identifier
  - `body_type`: Text description (e.g., "Terrestrial", "Gas Giant")
  - `spectral`: Spectral class for stars (e.g., "M4V", "G2V")
  - `radius`: Body radius (units vary by type)
  - `mass`: Body mass (solar masses for stars, Earth masses for planets)
  - `density`: Average density
  - `temp`: Surface temperature
  - `luminosity`: Stellar luminosity (solar units)

- **Orbital Mechanics**:
  - `distance`: Orbital distance from parent (AU for planets, unknown for stars)
  - `eccentricity`: Orbital eccentricity (0 = circular)
  - `inclination`: Orbital inclination in degrees
  - `rotation`: Rotation period in hours
  - `axial_tilt`: Axial tilt in degrees
  - `retrograde_orbit`: 1 if retrograde motion

- **Habitability & Population**:
  - `habitability`: Habitability rating (0-100)
  - `population`: Current population
  - `water`: Water coverage percentage
  - `atmosphere`: Atmospheric pressure
  - `political_affiliation`: Political entity controlling this body

- **Rendering & Display**:
  - `visible`: 1 if shown in 3D view
  - `color`: RGB color value for display
  - `label_color`: Label text color
  - `render_distance`: Maximum distance to render

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | system_id | INTEGER |  | 0 |  |
| 2 | parent_id | INTEGER |  | 0 |  |
| 3 | id_string | TEXT |  | - |  |
| 4 | name | TEXT |  | - |  |
| 5 | type_id | INTEGER |  | 0 |  |
| 6 | body_type | TEXT |  | - |  |
| 7 | spectral | TEXT |  | - |  |
| 8 | distance | double |  | 0 |  |
| 9 | radius | double |  | 0 |  |
| 10 | density | double |  | 0 |  |
| 11 | mass | double |  | 0 |  |
| 12 | rotation | double |  | 0 |  |
| 13 | luminosity | double |  | 0 |  |
| 14 | albedo | double |  | 0 |  |
| 15 | temp | double |  | 0 |  |
| 16 | atmosphere | double |  | 0 |  |
| 17 | atm_notes | TEXT |  | - |  |
| 18 | water | double |  | 0 |  |
| 19 | population | double |  | 0 |  |
| 20 | child_population | double |  | 0 |  |
| 21 | habitability | INTEGER |  | 0 |  |
| 22 | child_habitability | INTEGER |  | 0 |  |
| 23 | body_count | INTEGER |  | 0 |  |
| 24 | political_affiliation | TEXT |  | - |  |
| 25 | sphere_influence | INTEGER |  | 0 |  |
| 26 | sphere_color | INTEGER |  | 0 |  |
| 27 | sphere_size | double |  | 0 |  |
| 28 | notes | TEXT |  | - |  |
| 29 | gm_notes | TEXT |  | - |  |
| 30 | color | INTEGER |  | - |  |
| 31 | label_color | INTEGER |  | 0 |  |
| 32 | font_style | INTEGER |  | 0 |  |
| 33 | dso_intensity | INTEGER |  | 0 |  |
| 34 | render_distance | double |  | 0 |  |
| 35 | label_distance | double |  | 0 |  |
| 36 | route_distance | double |  | 0 |  |
| 37 | visible | INTEGER |  | 0 |  |
| 38 | retrograde_orbit | INTEGER |  | 0 |  |
| 39 | eccentricity | double |  | 0 |  |
| 40 | inclination | double |  | 0 |  |
| 41 | angle_ascending_node | double |  | 0 |  |
| 42 | angle_periapsis | double |  | 0 |  |
| 43 | time_offset | double |  | 0 |  |
| 44 | axial_tilt | double |  | 0 |  |
| 45 | orbital_plane_x | double |  | 0 |  |
| 46 | orbital_plane_y | double |  | 0 |  |
| 47 | orbital_plane_z | double |  | 0 |  |
| 48 | orbit_color | INTEGER |  | 16711680 |  |
| 49 | x | double |  | 0 |  |
| 50 | y | double |  | 0 |  |
| 51 | z | double |  | 0 |  |
| 52 | random_seed | INTEGER |  | 0 |  |
| 53 | fwe_color_group | INTEGER |  | 0 |  |
| 54 | preview_image_file | TEXT |  | '' |  |
| 55 | blazon_image_file | TEXT |  | '' |  |
| 56 | blazon_display | INTEGER |  | 0 |  |
| 57 | blazon_wrap | INTEGER |  | 0 |  |
| 58 | has_atm_comp | INTEGER |  | 0 |  |
| 59 | has_fields | INTEGER |  | 0 |  |
| 60 | has_sdc | INTEGER |  | 0 |  |
| 61 | has_fcg | INTEGER |  | 0 |  |
| 62 | model | TEXT |  | - |  |

#### Sample Data

| sphere_influence | eccentricity | atmosphere | gm_notes | label_color | mass | angle_ascending_node | atm_notes | orbital_plane_y | luminosity | density | color | parent_id | temp | political_affiliation | albedo | preview_image_file | blazon_image_file | habitability | radius | child_population | child_habitability | id_string | type_id | sphere_size | z | notes | inclination | dso_intensity | fwe_color_group | name | font_style | id | route_distance | population | rotation | render_distance | retrograde_orbit | system_id | distance | water | angle_periapsis | axial_tilt | has_fields | has_sdc | orbital_plane_z | time_offset | y | has_atm_comp | orbital_plane_x | body_type | body_count | random_seed | model | sphere_color | visible | x | blazon_display | label_distance | spectral | has_fcg | orbit_color | blazon_wrap |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| NULL | NULL | NULL |  | NULL | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL |  |  | NULL | NULL | NULL | NULL | {D2C92D49-3965-4C77-9763-AB6A8DD10A4C} | NULL | NULL | NULL |  | NULL | NULL | NULL | Njikiba | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | M4V | NULL | NULL | NULL |
| NULL | NULL | NULL |  | NULL | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL |  |  | NULL | NULL | NULL | NULL | {9D18C19F-C0E6-40BF-94A1-539EA2E1CA83} | NULL | NULL | NULL |  | NULL | NULL | NULL | Kilikili | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | O4V | NULL | NULL | NULL |
| NULL | NULL | NULL |  | NULL | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL |  |  | NULL | NULL | NULL | NULL | {EC55F739-2355-42A4-BAAC-BE93919B61AE} | NULL | NULL | NULL |  | NULL | NULL | NULL | Val | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | M4V | NULL | NULL | NULL |
| NULL | NULL | NULL |  | NULL | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL |  |  | NULL | NULL | NULL | NULL | {742C96C5-E72B-4D83-A986-F7302A2BEDEA} | NULL | NULL | NULL |  | NULL | NULL | NULL | Bramarta | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | M3V | NULL | NULL | NULL |
| NULL | NULL | NULL |  | NULL | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL |  |  | NULL | NULL | NULL | NULL | {CF3E7652-695A-4384-AF78-B85022B509CC} | NULL | NULL | NULL |  | NULL | NULL | NULL | Shilan | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL |  | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | M1V | NULL | NULL | NULL |

---

### config

**Row Count**: 1

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | name | TEXT |  | - |  |
| 2 | value | TEXT |  | - |  |

#### Sample Data

| id | name | value |
|---|---|---|
| NULL | db_version | 303 |

---

### custom_fields

**Row Count**: 0

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | parent_id | INTEGER |  | - |  |
| 2 | type_id | INTEGER |  | - |  |
| 3 | name | TEXT |  | - |  |
| 4 | value | TEXT |  | - |  |

---

### fwe_color_groups

**Row Count**: 0

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | body_id | INTEGER |  | - |  |
| 2 | deep_water | INTEGER |  | - |  |
| 3 | low_water | INTEGER |  | - |  |
| 4 | beach | INTEGER |  | - |  |
| 5 | low_land | INTEGER |  | - |  |
| 6 | high_land | INTEGER |  | - |  |
| 7 | ice | INTEGER |  | - |  |
| 8 | dry | INTEGER |  | - |  |
| 9 | gen_atm | INTEGER |  | - |  |
| 10 | atm_color | INTEGER |  | - |  |

---

### resources

**Row Count**: 2

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | id_string | TEXT |  | - |  |
| 2 | name | TEXT |  | - |  |
| 3 | data | BLOB |  | - |  |

#### Sample Data

| id | id_string | name | data |
|---|---|---|---|
| NULL |  | d:\nbos\astrosynthesis3\starblazonicon.png | NULL |
| NULL |  | c:\users\rober\downloads\starblazon.png | NULL |

---

### route_waypoints

**Purpose**: Defines individual waypoints along interstellar routes

**Row Count**: 90

**Description**: Each route consists of multiple waypoints defining the path between star systems. These waypoints can reference specific bodies or arbitrary 3D positions in space, allowing for complex navigation paths.

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | route_id | INTEGER |  | 0 |  |
| 2 | id_string | TEXT |  | - |  |
| 3 | body_id | INTEGER |  | 0 |  |
| 4 | pos_x | float |  | 0 |  |
| 5 | pos_y | float |  | 0 |  |
| 6 | pos_z | float |  | 0 |  |

#### Sample Data

| pos_z | pos_y | route_id | id | id_string | body_id | pos_x |
|---|---|---|---|---|---|---|
| NULL | NULL | NULL | NULL | {720A1E17-D981-4570-9357-27B36F0940F2} | NULL | NULL |
| NULL | NULL | NULL | NULL | {BF5A2610-71E1-4228-9C44-08C84E6E1892} | NULL | NULL |
| NULL | NULL | NULL | NULL | {720A1E17-D981-4570-9357-27B36F0940F2} | NULL | NULL |
| NULL | NULL | NULL | NULL | {618B5528-7B7D-40D8-BDBE-10BBF51B7E26} | NULL | NULL |
| NULL | NULL | NULL | NULL | {BF5A2610-71E1-4228-9C44-08C84E6E1892} | NULL | NULL |

---

### routes

**Purpose**: Defines interstellar travel routes between star systems

**Row Count**: 45

**Description**: Routes represent navigable paths between star systems, potentially representing trade routes, exploration paths, or strategic connections. Each route has visual styling properties for map rendering and can be bidirectional.

#### Key Fields:
- **Navigation**: `round_trip` (1 if bidirectional), `count` (number of waypoints)
- **Visualization**: `red/green/blue` (RGB color values 0-1), `line_width`, `line_style`, `line_stipple`

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | id_string | TEXT |  | - |  |
| 2 | name | TEXT |  | - |  |
| 3 | route_type |  |  | - |  |
| 4 | count | INTEGER |  | 0 |  |
| 5 | round_trip | INTEGER |  | 0 |  |
| 6 | visible | INTEGER |  | 0 |  |
| 7 | red | float |  | 0 |  |
| 8 | green | float |  | 0 |  |
| 9 | blue | float |  | 0 |  |
| 10 | line_width | float |  | 0 |  |
| 11 | line_style | INTEGER |  | 0 |  |
| 12 | line_stipple | INTEGER |  | 0 |  |

#### Sample Data

| line_stipple | name | id_string | id | round_trip | route_type | count | red | visible | green | blue | line_style | line_width |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| NULL |  | {AF4F838C-B6A1-47A1-98C1-21088017B8B3} | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL | NULL |
| NULL |  | {F66C09DD-F320-4130-8A17-992401D529CE} | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL | NULL |
| NULL |  | {7982ED5F-BFDD-4D1A-9094-1A130D150F3E} | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL | NULL |
| NULL |  | {3E69BEE7-8527-488C-80C1-96232EAE2DBB} | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL | NULL |
| NULL |  | {577C47A0-C671-461E-9304-EEEC7BFF4556} | NULL | NULL |  | NULL | NULL | NULL | NULL | NULL | NULL | NULL |

---

### sector_info

**Purpose**: Global metadata for the entire stellar sector/database

**Row Count**: 1

**Description**: Contains configuration settings for the entire stellar sector including coordinate system parameters, display preferences, time tracking, and camera positioning for the 3D view. This is essentially the "header" information for the Astrosynthesis file.

#### Key Fields:
- **Spatial**: `size_x/y/z` (sector dimensions in light-years), `spherical` (1 if using spherical coordinates)
- **Time**: `epoch_year` (base year), `time` (current time offset), `date_format`
- **Units**: `unit_star_measure_name/abbrev/ratio` (custom measurement units)
- **Grid**: `grid_x/y/z` (grid dimensions), `grid_tick` (spacing), `grid_color`
- **Camera**: `origin_camera_x/y/z`, `origin_target_x/y/z` (default 3D view position)

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | id_string | TEXT |  | - |  |
| 2 | name | TEXT |  | - |  |
| 3 | notes | TEXT |  | - |  |
| 4 | gm_notes | TEXT |  | - |  |
| 5 | epoch_year | INTEGER |  | 0 |  |
| 6 | time | float |  | 0 |  |
| 7 | date_format | TEXT |  | - |  |
| 8 | unit_star_measure_name | TEXT |  | - |  |
| 9 | unit_star_measure_abbrev | TEXT |  | - |  |
| 10 | unit_star_measure_ratio | float |  | 0 |  |
| 11 | size_x | float |  | 0 |  |
| 12 | size_y | float |  | 0 |  |
| 13 | size_z | float |  | 0 |  |
| 14 | frequency | float |  | 0 |  |
| 15 | spherical | INTEGER |  | 0 |  |
| 16 | grid_style | INTEGER |  | 0 |  |
| 17 | grid_x | float |  | 0 |  |
| 18 | grid_y | float |  | 0 |  |
| 19 | grid_z | float |  | 0 |  |
| 20 | grid_tick | float |  | 0 |  |
| 21 | grid_tick_labels | INTEGER |  | 0 |  |
| 22 | grid_color | INTEGER |  | 0 |  |
| 23 | grids_enabled | INTEGER |  | 0 |  |
| 24 | background_color | INTEGER |  | 0 |  |
| 25 | base_font | TEXT |  | - |  |
| 26 | filter_style | INTEGER |  | 0 |  |
| 27 | filter_habitable | INTEGER |  | 0 |  |
| 28 | filter_population | INTEGER |  | 0 |  |
| 29 | filter_hide_unassigned_subsector | INTEGER |  | 0 |  |
| 30 | origin_camera_x | float |  | 0 |  |
| 31 | origin_camera_y | float |  | 0 |  |
| 32 | origin_camera_z | float |  | 0 |  |
| 33 | origin_target_x | float |  | 0 |  |
| 34 | origin_target_y | float |  | 0 |  |
| 35 | origin_target_z | float |  | 0 |  |

#### Sample Data

| frequency | notes | time | grid_tick_labels | unit_star_measure_abbrev | filter_style | filter_hide_unassigned_subsector | origin_camera_z | grid_style | filter_population | size_x | background_color | base_font | size_z | unit_star_measure_name | id | grids_enabled | origin_target_y | id_string | gm_notes | size_y | grid_x | origin_target_x | grid_y | epoch_year | filter_habitable | unit_star_measure_ratio | origin_target_z | grid_tick | spherical | grid_z | name | date_format | grid_color | origin_camera_x | origin_camera_y |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| NULL |  | NULL | NULL | ly | NULL | NULL | NULL | NULL | NULL | NULL | NULL | Verdana | NULL | Light Years | NULL | NULL | NULL | {76A3B4BD-7FEC-41C6-AF93-706856FF0DAE} |  | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | TotalSystem | dd mmm y h:n:s | NULL | NULL | NULL |

---

### sector_view_arrays

**Row Count**: 8

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | type_id | INTEGER |  | 0 |  |
| 2 | view_id | INTEGER |  | 0 |  |
| 3 | name | TEXT |  | - |  |
| 4 | visible | INTEGER |  | 0 |  |

#### Sample Data

| type_id | id | name | visible | view_id |
|---|---|---|---|---|
| NULL | NULL |  | NULL | NULL |
| NULL | NULL |  | NULL | NULL |
| NULL | NULL |  | NULL | NULL |
| NULL | NULL |  | NULL | NULL |
| NULL | NULL |  | NULL | NULL |

---

### sector_views

**Row Count**: 4

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | name | TEXT |  | - |  |
| 2 | camera_pos_x | float |  | 0 |  |
| 3 | camera_pos_y | float |  | 0 |  |
| 4 | camera_pos_z | float |  | 0 |  |
| 5 | target_pos_x | float |  | 0 |  |
| 6 | target_pos_y | float |  | 0 |  |
| 7 | target_pos_z | float |  | 0 |  |
| 8 | filter_style | INTEGER |  | 0 |  |
| 9 | filter_habitable | INTEGER |  | 0 |  |
| 10 | filter_population | INTEGER |  | 0 |  |
| 11 | filter_hide_unassigned_subsector | INTEGER |  | 0 |  |

#### Sample Data

| camera_pos_y | camera_pos_x | id | filter_population | filter_hide_unassigned_subsector | target_pos_z | filter_style | target_pos_y | name | filter_habitable | camera_pos_z | target_pos_x |
|---|---|---|---|---|---|---|---|---|---|---|---|
| NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | Amateru | NULL | NULL | NULL |
| NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | Wasomi | NULL | NULL | NULL |
| NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | Hoshima | NULL | NULL | NULL |
| NULL | NULL | NULL | NULL | NULL | NULL | NULL | NULL | Morazu | NULL | NULL | NULL |

---

### subsectors

**Row Count**: 0

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | id_string | TEXT |  | - |  |
| 2 | name | TEXT |  | - |  |
| 3 | color | INTEGER |  | 0 |  |
| 4 | show_label | INTEGER |  | 0 |  |
| 5 | shape | INTEGER |  | 0 |  |
| 6 | visible | INTEGER |  | 0 |  |
| 7 | center_x | float |  | 0 |  |
| 8 | center_y | float |  | 0 |  |
| 9 | center_z | float |  | 0 |  |
| 10 | size_x | float |  | 0 |  |
| 11 | size_y | float |  | 0 |  |
| 12 | size_z | float |  | 0 |  |
| 13 | show_grid | INTEGER |  | 0 |  |
| 14 | grid_x | float |  | 0 |  |
| 15 | grid_y | float |  | 0 |  |
| 16 | grid_z | float |  | 0 |  |
| 17 | rotate_x | float |  | 0 |  |
| 18 | rotate_y | float |  | 0 |  |
| 19 | rotate_z | float |  | 0 |  |
| 20 | grid_tick | float |  | 0 |  |
| 21 | tick_labels | INTEGER |  | 0 |  |

---

### surface_maps

**Row Count**: 0

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | body_id | INTEGER |  | - |  |
| 2 | id_string | TEXT |  | - |  |
| 3 | surface_map | BLOB |  | - |  |
| 4 | surface_map_image | BLOB |  | - |  |

---

### system_data_config

**Row Count**: 110

#### Columns

| # | Name | Type | Not Null | Default | Primary Key |
|---|------|------|----------|---------|-------------|
| 0 | id | INTEGER | ✓ | - | ✓ |
| 1 | body_id | INTEGER |  | - |  |
| 2 | type_id | INTEGER |  | - |  |
| 3 | sort_order | INTEGER |  | - |  |
| 4 | name | TEXT |  | - |  |
| 5 | source | TEXT |  | - |  |
| 6 | field | TEXT |  | - |  |
| 7 | hide_if_empty | INTEGER |  | 0 |  |

#### Sample Data

| id | source | field | name | sort_order | body_id | type_id | hide_if_empty |
|---|---|---|---|---|---|---|---|
| NULL |  |  | Not Assigned | NULL | NULL | NULL | NULL |
| NULL | spectral |  | Spectral Class | NULL | NULL | NULL | NULL |
| NULL | mass |  | Mass | NULL | NULL | NULL | NULL |
| NULL | solradius |  | Radius | NULL | NULL | NULL | NULL |
| NULL | luminosity |  | Luminosity | NULL | NULL | NULL | NULL |

---

## Relationships

### Primary Relationships (via ID columns)

```
bodies (star)
  ├── bodies (planets) via parent_id
  │   └── bodies (moons) via parent_id
  ├── atm_components via body_id
  ├── system_data_config via body_id
  ├── fwe_color_groups via body_id
  └── custom_fields via parent_id

routes
  └── route_waypoints via route_id
      └── bodies via body_id (optional reference)

sector_views
  └── sector_view_arrays via view_id

resources (standalone - binary data storage)
surface_maps → bodies via body_id
subsectors (standalone - spatial regions)
```

### Data Integrity Notes

1. **No Foreign Key Constraints**: The database relies on application logic for referential integrity
2. **Hierarchical Body Structure**: Determined by checking `system_id` and `parent_id` combinations
3. **Optional Relationships**: Many relationships are optional (nullable foreign keys)
4. **Orphaned Records**: The lack of constraints means orphaned records are possible

### Query Patterns

**Find all planets in a star system:**
```sql
SELECT * FROM bodies
WHERE system_id = ? AND parent_id = ? AND id != parent_id
```

**Get route with waypoints:**
```sql
SELECT r.*, rw.*, b.name as waypoint_name
FROM routes r
LEFT JOIN route_waypoints rw ON rw.route_id = r.id
LEFT JOIN bodies b ON b.id = rw.body_id
WHERE r.id = ?
ORDER BY rw.id
```

**Find habitable worlds:**
```sql
SELECT name, spectral, habitability, population, water, atmosphere
FROM bodies
WHERE habitability > 50
ORDER BY habitability DESC
```

---

*This schema documentation was automatically generated by SolarViewer*
