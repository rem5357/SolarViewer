// Builder patterns for creating stellar systems and objects in StellarForge

use crate::stellar_forge::core::{Id, State, Vec3, Units};
use crate::stellar_forge::bodies::{StellarBody, BodyKind, SpatialParent};
use crate::stellar_forge::containers::{StarSystem, Galaxy, SystemType, PoliticalRegion};
use crate::stellar_forge::frames::Frame;
use crate::stellar_forge::motion::{MotionModel, OrbitalElements, FreeMotion};
use crate::stellar_forge::physical::{
    Physical, StarPhysical, PlanetPhysical, PlanetaryComposition,
    Atmosphere, StationPhysical, BeltPhysical,
};
use crate::stellar_forge::associations::Tag;
use rand::Rng;
use time::OffsetDateTime;

// System builder for creating complete star systems
pub struct SystemBuilder {
    name: String,
    galactic_coords: Option<crate::stellar_forge::coordinates::GalacticCoordinates>,
    legacy_position: Option<Vec3>,
    stars: Vec<StellarBody>,
    planets: Vec<PlanetBuilder>,
    belts: Vec<BeltBuilder>,
    stations: Vec<StationBuilder>,
    tags: Vec<Tag>,
}

impl SystemBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            galactic_coords: None,
            legacy_position: None,
            stars: Vec::new(),
            planets: Vec::new(),
            belts: Vec::new(),
            stations: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Set position using proper galactic coordinates
    pub fn at_galactic(mut self, l_deg: f64, b_deg: f64, distance_pc: f64) -> Self {
        self.galactic_coords = Some(
            crate::stellar_forge::coordinates::GalacticCoordinates::new(l_deg, b_deg, distance_pc)
        );
        self
    }

    /// Set position from Sun in parsecs (convenience method)
    pub fn at_position_pc(mut self, x_pc: f64, y_pc: f64, z_pc: f64) -> Self {
        // Convert cartesian position to galactic coordinates
        let cart = Vec3::new(
            x_pc * Units::PARSEC,
            y_pc * Units::PARSEC,
            z_pc * Units::PARSEC,
        );
        self.galactic_coords = Some(
            crate::stellar_forge::coordinates::GalacticCoordinates::from_cartesian(cart)
        );
        self
    }

    /// Legacy method for backward compatibility (assumes light-years)
    pub fn at_position(mut self, x_ly: f64, y_ly: f64, z_ly: f64) -> Self {
        let cart = Vec3::new(
            x_ly * Units::LIGHT_YEAR,
            y_ly * Units::LIGHT_YEAR,
            z_ly * Units::LIGHT_YEAR,
        );
        self.galactic_coords = Some(
            crate::stellar_forge::coordinates::GalacticCoordinates::from_cartesian(cart)
        );
        self.legacy_position = Some(cart);
        self
    }

    pub fn with_star(mut self, spectral_type: impl Into<String>) -> Self {
        let star = StellarBody::new_star(
            format!("{} A", self.name),
            SpatialParent::Frame(Id::new_v4()),
            spectral_type,
        );
        self.stars.push(star);
        self
    }

    pub fn with_binary_stars(
        mut self,
        primary_spectral: impl Into<String>,
        secondary_spectral: impl Into<String>,
        separation_au: f64,
    ) -> Self {
        let mut primary = StellarBody::new_star(
            format!("{} A", self.name),
            SpatialParent::Frame(Id::new_v4()),
            primary_spectral,
        );

        let mut secondary = StellarBody::new_star(
            format!("{} B", self.name),
            SpatialParent::Frame(Id::new_v4()),
            secondary_spectral,
        );

        // Set binary orbit
        let primary_mass = primary.mass_kg().unwrap_or(Units::SOLAR_MASS);
        let secondary_mass = secondary.mass_kg().unwrap_or(0.5 * Units::SOLAR_MASS);
        let total_mass = primary_mass + secondary_mass;

        // Barycenter calculations
        let r1 = separation_au * Units::AU * secondary_mass / total_mass;
        let r2 = separation_au * Units::AU * primary_mass / total_mass;

        primary.set_position(Vec3::new(-r1, 0.0, 0.0));
        secondary.set_position(Vec3::new(r2, 0.0, 0.0));

        // Set orbital motion for secondary around barycenter
        let mu = 6.67430e-11 * total_mass;  // G * M_total
        let orbital_period = 2.0 * std::f64::consts::PI
            * ((separation_au * Units::AU).powi(3) / mu).sqrt();

        let elements = OrbitalElements::new(
            separation_au * Units::AU,
            0.0,  // circular
            0.0,  // coplanar
            0.0,
            0.0,
            0.0,
            mu,
            OffsetDateTime::now_utc(),
        );

        secondary.set_orbital_motion(MotionModel::Keplerian(elements));

        self.stars.push(primary);
        self.stars.push(secondary);
        self
    }

    pub fn with_planet(mut self, builder: PlanetBuilder) -> Self {
        self.planets.push(builder);
        self
    }

    pub fn with_asteroid_belt(mut self, builder: BeltBuilder) -> Self {
        self.belts.push(builder);
        self
    }

    pub fn with_station(mut self, builder: StationBuilder) -> Self {
        self.stations.push(builder);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<Tag>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn build(self) -> StarSystem {
        // Use provided coordinates or default to Sol's position
        let coords = self.galactic_coords.unwrap_or_else(|| {
            crate::stellar_forge::coordinates::ReferencePositions::sol()
        });

        let mut system = if self.stars.len() == 1 {
            StarSystem::new_single_star(self.name, coords, self.stars.into_iter().next().unwrap())
        } else if self.stars.len() == 2 {
            let mut stars = self.stars.into_iter();
            StarSystem::new_binary(self.name, coords, stars.next().unwrap(), stars.next().unwrap())
        } else {
            let mut system = StarSystem {
                id: Id::new_v4(),
                name: self.name.clone(),
                galactic_coordinates: coords,
                system_type: SystemType::Multiple(self.stars.len() as u8),
                stars: self.stars,
                planets: Vec::new(),
                belts: Vec::new(),
                stations: Vec::new(),
                other_bodies: Vec::new(),
                barycenter: Vec3::zeros(),
                frame_id: Id::new_v4(),
                legacy_position: self.legacy_position,
            };
            system.update_barycenter();
            system
        };

        // Build and add planets
        for planet_builder in self.planets {
            if let Ok(planet) = planet_builder.build(&system) {
                system.add_planet(planet).ok();
            }
        }

        // Build and add belts
        for belt_builder in self.belts {
            if let Ok(belt) = belt_builder.build(&system) {
                system.add_asteroid_belt(belt).ok();
            }
        }

        // Build and add stations
        for station_builder in self.stations {
            if let Ok(station) = station_builder.build(&system) {
                system.add_station(station).ok();
            }
        }

        system
    }
}

// Planet builder
pub struct PlanetBuilder {
    name: String,
    orbital_radius_au: f64,
    eccentricity: f64,
    inclination_rad: f64,
    mass_earth: f64,
    radius_earth: f64,
    composition: PlanetaryComposition,
    atmosphere: Option<Atmosphere>,
    water_percent: Option<f64>,
    moons: Vec<MoonBuilder>,
    tags: Vec<Tag>,
}

impl PlanetBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            orbital_radius_au: 1.0,
            eccentricity: 0.0,
            inclination_rad: 0.0,
            mass_earth: 1.0,
            radius_earth: 1.0,
            composition: PlanetaryComposition::Terrestrial,
            atmosphere: None,
            water_percent: None,
            moons: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn terrestrial(name: impl Into<String>, orbital_radius_au: f64) -> Self {
        Self::new(name)
            .at_orbit(orbital_radius_au)
            .with_mass_and_radius(1.0, 1.0)
            .with_composition(PlanetaryComposition::Terrestrial)
    }

    pub fn gas_giant(name: impl Into<String>, orbital_radius_au: f64) -> Self {
        Self::new(name)
            .at_orbit(orbital_radius_au)
            .with_mass_and_radius(317.8, 11.2)  // Jupiter-like
            .with_composition(PlanetaryComposition::GasGiant)
    }

    pub fn ice_giant(name: impl Into<String>, orbital_radius_au: f64) -> Self {
        Self::new(name)
            .at_orbit(orbital_radius_au)
            .with_mass_and_radius(14.5, 4.0)  // Neptune-like
            .with_composition(PlanetaryComposition::IceGiant)
    }

    pub fn at_orbit(mut self, radius_au: f64) -> Self {
        self.orbital_radius_au = radius_au;
        self
    }

    pub fn with_eccentricity(mut self, e: f64) -> Self {
        self.eccentricity = e.clamp(0.0, 0.99);
        self
    }

    pub fn with_inclination_deg(mut self, i_deg: f64) -> Self {
        self.inclination_rad = i_deg.to_radians();
        self
    }

    pub fn with_mass_and_radius(mut self, mass_earth: f64, radius_earth: f64) -> Self {
        self.mass_earth = mass_earth;
        self.radius_earth = radius_earth;
        self
    }

    pub fn with_composition(mut self, composition: PlanetaryComposition) -> Self {
        self.composition = composition;
        self
    }

    pub fn with_atmosphere(mut self, atmosphere: Atmosphere) -> Self {
        self.atmosphere = Some(atmosphere);
        self
    }

    pub fn with_earth_like_atmosphere(mut self) -> Self {
        self.atmosphere = Some(Atmosphere::earth_like());
        self
    }

    pub fn with_water(mut self, percent: f64) -> Self {
        self.water_percent = Some(percent.clamp(0.0, 100.0));
        self
    }

    pub fn with_moon(mut self, moon: MoonBuilder) -> Self {
        self.moons.push(moon);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<Tag>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn build(self, system: &StarSystem) -> Result<StellarBody, String> {
        let mut planet = StellarBody::new_planet(
            self.name,
            SpatialParent::Body(system.id),
        );

        // Set physical properties
        let mut physical = PlanetPhysical::default();
        physical.mass_kg = self.mass_earth * Units::EARTH_MASS;
        physical.radius_m = self.radius_earth * Units::EARTH_RADIUS;
        physical.gravity_mps2 = 6.67430e-11 * physical.mass_kg / (physical.radius_m * physical.radius_m);
        physical.composition = self.composition;
        physical.atmosphere = self.atmosphere;
        physical.surface_water_percent = self.water_percent;

        planet.physical = Some(Physical::Planet(physical));

        // Set orbital motion
        let primary_mass = system.total_mass();
        let mu = 6.67430e-11 * primary_mass;

        let elements = OrbitalElements::new(
            self.orbital_radius_au * Units::AU,
            self.eccentricity,
            self.inclination_rad,
            0.0,  // RAAN
            0.0,  // Argument of periapsis
            0.0,  // Mean anomaly at epoch
            mu,
            OffsetDateTime::now_utc(),
        );

        planet.set_orbital_motion(MotionModel::Keplerian(elements));

        // Add moons
        for moon_builder in self.moons {
            if let Ok(moon) = moon_builder.build(&planet) {
                planet.add_child(moon).ok();
            }
        }

        // Add tags
        for tag in self.tags {
            planet.add_tag(tag);
        }

        Ok(planet)
    }
}

// Moon builder
pub struct MoonBuilder {
    name: String,
    orbital_radius_km: f64,
    mass_lunar: f64,
    radius_lunar: f64,
    is_captured: bool,
    retrograde: bool,
}

impl MoonBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            orbital_radius_km: 384400.0,  // Moon's distance
            mass_lunar: 1.0,
            radius_lunar: 1.0,
            is_captured: false,
            retrograde: false,
        }
    }

    pub fn at_orbit_km(mut self, radius_km: f64) -> Self {
        self.orbital_radius_km = radius_km;
        self
    }

    pub fn with_mass_and_radius(mut self, mass_lunar: f64, radius_lunar: f64) -> Self {
        self.mass_lunar = mass_lunar;
        self.radius_lunar = radius_lunar;
        self
    }

    pub fn captured(mut self) -> Self {
        self.is_captured = true;
        self
    }

    pub fn retrograde(mut self) -> Self {
        self.retrograde = true;
        self
    }

    pub fn build(self, planet: &StellarBody) -> Result<StellarBody, String> {
        let mut moon = StellarBody::new_moon(
            self.name,
            SpatialParent::Body(planet.id),
        );

        // Set physical properties
        let mut physical = PlanetPhysical::default();
        physical.mass_kg = self.mass_lunar * 7.342e22;  // Lunar mass
        physical.radius_m = self.radius_lunar * 1.737e6;  // Lunar radius
        physical.gravity_mps2 = 6.67430e-11 * physical.mass_kg / (physical.radius_m * physical.radius_m);

        moon.physical = Some(Physical::Moon(physical));

        // Set orbital motion
        let planet_mass = planet.mass_kg().unwrap_or(Units::EARTH_MASS);
        let mu = 6.67430e-11 * planet_mass;

        let inclination = if self.retrograde {
            std::f64::consts::PI * 0.9  // Highly inclined retrograde
        } else if self.is_captured {
            rand::thread_rng().gen_range(0.0..std::f64::consts::PI / 3.0)
        } else {
            0.0  // Equatorial
        };

        let elements = OrbitalElements::new(
            self.orbital_radius_km * 1000.0,
            if self.is_captured { 0.2 } else { 0.0 },
            inclination,
            0.0,
            0.0,
            0.0,
            mu,
            OffsetDateTime::now_utc(),
        );

        moon.set_orbital_motion(MotionModel::Keplerian(elements));

        Ok(moon)
    }
}

// Asteroid belt builder
pub struct BeltBuilder {
    name: String,
    inner_radius_au: f64,
    outer_radius_au: f64,
    total_mass_earth: f64,
    composition: String,
}

impl BeltBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            inner_radius_au: 2.0,
            outer_radius_au: 3.5,
            total_mass_earth: 0.0005,  // Asteroid belt mass
            composition: "Rocky/Metallic".to_string(),
        }
    }

    pub fn between(mut self, inner_au: f64, outer_au: f64) -> Self {
        self.inner_radius_au = inner_au;
        self.outer_radius_au = outer_au;
        self
    }

    pub fn with_mass_earth(mut self, mass: f64) -> Self {
        self.total_mass_earth = mass;
        self
    }

    pub fn with_composition(mut self, comp: impl Into<String>) -> Self {
        self.composition = comp.into();
        self
    }

    pub fn build(self, system: &StarSystem) -> Result<StellarBody, String> {
        let mut belt = StellarBody::new_belt(
            self.name,
            SpatialParent::Body(system.id),
        );

        let physical = BeltPhysical {
            inner_radius_m: self.inner_radius_au * Units::AU,
            outer_radius_m: self.outer_radius_au * Units::AU,
            thickness_m: Some(0.1 * Units::AU),  // Typical thickness
            total_mass_kg: Some(self.total_mass_earth * Units::EARTH_MASS),
            particle_density: Some(1000.0),  // particles per million cubic km
            average_particle_size_m: Some(1000.0),  // 1 km average
            composition: self.composition,
        };

        belt.physical = Some(Physical::Belt(physical));

        // Belt orbits at average radius
        let avg_radius = (self.inner_radius_au + self.outer_radius_au) / 2.0 * Units::AU;
        belt.set_position(Vec3::new(avg_radius, 0.0, 0.0));

        Ok(belt)
    }
}

// Station builder
pub struct StationBuilder {
    name: String,
    station_type: crate::stellar_forge::physical::StationType,
    population: Option<u64>,
    docking_ports: u32,
    orbit_body_id: Option<Id>,
    position: Option<Vec3>,
}

impl StationBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            station_type: crate::stellar_forge::physical::StationType::Trading,
            population: None,
            docking_ports: 4,
            orbit_body_id: None,
            position: None,
        }
    }

    pub fn of_type(mut self, station_type: crate::stellar_forge::physical::StationType) -> Self {
        self.station_type = station_type;
        self
    }

    pub fn with_population(mut self, pop: u64) -> Self {
        self.population = Some(pop);
        self
    }

    pub fn with_docking_ports(mut self, ports: u32) -> Self {
        self.docking_ports = ports;
        self
    }

    pub fn orbiting(mut self, body_id: Id) -> Self {
        self.orbit_body_id = Some(body_id);
        self
    }

    pub fn at_position(mut self, pos: Vec3) -> Self {
        self.position = Some(pos);
        self
    }

    pub fn build(self, system: &StarSystem) -> Result<StellarBody, String> {
        let mut station = StellarBody::new_station(
            self.name,
            SpatialParent::Body(system.id),
        );

        let mut physical = StationPhysical::default();
        physical.station_type = self.station_type;
        physical.docking_ports = self.docking_ports;
        physical.current_population = self.population;

        station.physical = Some(Physical::Station(physical));

        // Set position or orbit
        if let Some(pos) = self.position {
            station.set_position(pos);
        } else if let Some(_body_id) = self.orbit_body_id {
            // Set up orbital motion around specified body
            // This would require looking up the body and setting appropriate motion
        }

        Ok(station)
    }
}

// Galaxy builder for creating entire galaxies
pub struct GalaxyBuilder {
    name: String,
    size_ly: Vec3,
    sectors: Vec<SectorBuilder>,
    systems: Vec<SystemBuilder>,
}

impl GalaxyBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size_ly: Vec3::new(100000.0, 100000.0, 1000.0),  // Milky Way-like
            sectors: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn with_size(mut self, x_ly: f64, y_ly: f64, z_ly: f64) -> Self {
        self.size_ly = Vec3::new(x_ly, y_ly, z_ly);
        self
    }

    pub fn with_sector(mut self, sector: SectorBuilder) -> Self {
        self.sectors.push(sector);
        self
    }

    pub fn with_system(mut self, system: SystemBuilder) -> Self {
        self.systems.push(system);
        self
    }

    pub fn with_random_systems(mut self, count: usize, seed: u64) -> Self {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        for i in 0..count {
            let x = rng.gen_range(-self.size_ly.x / 2.0..self.size_ly.x / 2.0);
            let y = rng.gen_range(-self.size_ly.y / 2.0..self.size_ly.y / 2.0);
            let z = rng.gen_range(-self.size_ly.z / 2.0..self.size_ly.z / 2.0);

            let spectral_types = ["M5V", "M0V", "K5V", "K0V", "G5V", "G0V", "F5V", "F0V", "A5V", "A0V"];
            let spectral = spectral_types[rng.gen_range(0..spectral_types.len())];

            let system = SystemBuilder::new(format!("System-{}", i))
                .at_position(x, y, z)
                .with_star(spectral);

            self.systems.push(system);
        }

        self
    }

    pub fn build(self) -> Galaxy {
        let mut galaxy = Galaxy::new(self.name);
        galaxy.metadata.size_ly = self.size_ly;

        // Build and add all systems
        for system_builder in self.systems {
            let system = system_builder.build();
            galaxy.add_star_system(system).ok();
        }

        galaxy
    }
}

// Sector builder
pub struct SectorBuilder {
    name: String,
    min_bound: Vec3,
    max_bound: Vec3,
}

impl SectorBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            min_bound: Vec3::zeros(),
            max_bound: Vec3::zeros(),
        }
    }

    pub fn with_bounds(mut self, min: Vec3, max: Vec3) -> Self {
        self.min_bound = min;
        self.max_bound = max;
        self
    }
}

// Helper functions for common system configurations
pub fn create_sol_like_system() -> SystemBuilder {
    SystemBuilder::new("Sol")
        .with_star("G2V")
        .with_planet(
            PlanetBuilder::terrestrial("Mercury", 0.387)
                .with_mass_and_radius(0.055, 0.383)
        )
        .with_planet(
            PlanetBuilder::terrestrial("Venus", 0.723)
                .with_mass_and_radius(0.815, 0.949)
                .with_water(0.0)
        )
        .with_planet(
            PlanetBuilder::terrestrial("Earth", 1.0)
                .with_earth_like_atmosphere()
                .with_water(71.0)
                .with_moon(MoonBuilder::new("Moon"))
        )
        .with_planet(
            PlanetBuilder::terrestrial("Mars", 1.524)
                .with_mass_and_radius(0.107, 0.532)
                .with_moon(MoonBuilder::new("Phobos").at_orbit_km(9377.0))
                .with_moon(MoonBuilder::new("Deimos").at_orbit_km(23460.0))
        )
        .with_asteroid_belt(
            BeltBuilder::new("Main Belt")
                .between(2.0, 3.5)
        )
        .with_planet(
            PlanetBuilder::gas_giant("Jupiter", 5.2)
                .with_moon(MoonBuilder::new("Io").at_orbit_km(421800.0))
                .with_moon(MoonBuilder::new("Europa").at_orbit_km(671100.0))
                .with_moon(MoonBuilder::new("Ganymede").at_orbit_km(1070400.0))
                .with_moon(MoonBuilder::new("Callisto").at_orbit_km(1882700.0))
        )
        .with_planet(
            PlanetBuilder::gas_giant("Saturn", 9.5)
                .with_mass_and_radius(95.2, 9.45)
                .with_moon(MoonBuilder::new("Titan").at_orbit_km(1221700.0))
        )
        .with_planet(
            PlanetBuilder::ice_giant("Uranus", 19.2)
                .with_inclination_deg(97.8)  // Tilted axis
        )
        .with_planet(
            PlanetBuilder::ice_giant("Neptune", 30.0)
        )
}