// Example demonstrating StellarForge data structure usage

use solarviewer::stellar_forge::{
    builders::{SystemBuilder, PlanetBuilder, MoonBuilder, GalaxyBuilder, create_sol_like_system},
    containers::Galaxy,
    storage::{StellarForgeDataset, FileStorage},
    associations::{Tag, TagCategories},
    services::StellarForgeService,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("StellarForge Demo");
    println!("=================\n");

    // Example 1: Create a simple star system
    println!("1. Creating a simple star system...");
    let alpha_centauri = SystemBuilder::new("Alpha Centauri")
        .at_position(1.34, 0.0, 0.0)  // 1.34 parsecs from Sol
        .with_binary_stars("G2V", "K1V", 23.0)  // Binary stars at 23 AU separation
        .with_planet(
            PlanetBuilder::terrestrial("Proxima b", 0.05)
                .with_mass_and_radius(1.27, 1.08)
                .with_tag("potentially_habitable")
        )
        .with_tag("nearest_system")
        .build();

    println!("   Created: {} with {} stars and {} planets",
        alpha_centauri.name,
        alpha_centauri.stars.len(),
        alpha_centauri.planets.len()
    );

    // Example 2: Create Sol system
    println!("\n2. Creating Sol system...");
    let sol = create_sol_like_system()
        .at_position(0.0, 0.0, 0.0)
        .build();

    println!("   Created: {} with {} planets", sol.name, sol.planets.len());
    for planet in &sol.planets {
        println!("     - {} with {} moons", planet.name(), planet.child_count());
    }

    // Example 3: Create a custom system with multiple planet types
    println!("\n3. Creating a diverse custom system...");
    let custom_system = SystemBuilder::new("Kepler-452")
        .at_position(430.0, 0.0, 0.0)  // 430 parsecs away
        .with_star("G2V")
        .with_planet(
            PlanetBuilder::terrestrial("Kepler-452b", 1.05)
                .with_mass_and_radius(5.0, 1.63)
                .with_earth_like_atmosphere()
                .with_water(60.0)
                .with_tag("super_earth")
                .with_tag("habitable_zone")
        )
        .with_planet(
            PlanetBuilder::gas_giant("Kepler-452c", 5.2)
                .with_moon(MoonBuilder::new("Moon I"))
                .with_moon(MoonBuilder::new("Moon II").captured())
        )
        .build();

    println!("   Created: {}", custom_system.name);
    println!("   Habitable zone: {:?}", custom_system.habitable_zone());

    // Example 4: Create a small galaxy with multiple systems
    println!("\n4. Creating a small galaxy...");
    let galaxy = GalaxyBuilder::new("Demo Galaxy")
        .with_size(1000.0, 1000.0, 100.0)  // Small galaxy, 1000 ly across
        .with_system(
            SystemBuilder::new("System A")
                .at_position(100.0, 50.0, 0.0)
                .with_star("F5V")
        )
        .with_system(
            SystemBuilder::new("System B")
                .at_position(-100.0, -50.0, 10.0)
                .with_star("M2V")
        )
        .with_random_systems(10, 12345)  // Add 10 random systems
        .build();

    println!("   Created: {} with {} systems", galaxy.name, galaxy.star_systems.len());

    // Example 5: Use the service layer
    println!("\n5. Using StellarForge service...");
    let mut service = StellarForgeService::new("Service Galaxy");
    service.initialize()?;

    // Add our systems to the service
    service.add_system(sol)?;
    service.add_system(alpha_centauri)?;
    service.add_system(custom_system)?;

    // Query nearby systems
    let nearby = service.query_systems_in_range(
        nalgebra::Vector3::new(0.0, 0.0, 0.0),
        2.0  // Within 2 light-years
    );
    println!("   Systems within 2 ly of origin: {}", nearby.len());

    // Example 6: Save to file
    println!("\n6. Saving galaxy to file...");
    let dataset = StellarForgeDataset::new(galaxy);
    FileStorage::save_json(&dataset, "demo_galaxy.json")?;
    println!("   Saved to demo_galaxy.json");

    // Example 7: Tags and associations
    println!("\n7. Working with tags and associations...");
    let exploration_tags = TagCategories::exploration();
    let resource_tags = TagCategories::resources();

    println!("   Available exploration tags:");
    for tag in &exploration_tags {
        println!("     - {}", tag.as_str());
    }

    println!("   Available resource tags:");
    for tag in &resource_tags {
        println!("     - {}", tag.as_str());
    }

    println!("\nStellarForge demo complete!");

    Ok(())
}