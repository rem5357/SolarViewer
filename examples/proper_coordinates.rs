// Example demonstrating proper astronomical coordinates in StellarForge

use solarviewer::stellar_forge::{
    coordinates::{
        GalacticCoordinates, EquatorialCoordinates,
        CoordinateTransform, CoordinateFormatter, ReferencePositions,
    },
    builders::SystemBuilder,
};

fn main() {
    println!("Proper Astronomical Coordinates Demo");
    println!("=====================================\n");

    // Example 1: Known star positions in proper galactic coordinates
    println!("1. Reference Star Positions (IAU Galactic Coordinates):");
    println!("   Sol: {}",
        CoordinateFormatter::format_galactic(&ReferencePositions::sol())
    );
    println!("   Alpha Centauri: {}",
        CoordinateFormatter::format_galactic(&ReferencePositions::alpha_centauri())
    );
    println!("   Polaris: {}",
        CoordinateFormatter::format_galactic(&ReferencePositions::polaris())
    );
    println!("   Vega: {}",
        CoordinateFormatter::format_galactic(&ReferencePositions::vega())
    );
    println!("   Galactic Center (Sgr A*): {}",
        CoordinateFormatter::format_galactic(&ReferencePositions::sgr_a_star())
    );

    // Example 2: Creating systems with proper coordinates
    println!("\n2. Creating Systems with Proper Coordinates:");

    // Using galactic coordinates directly
    let kepler_452 = SystemBuilder::new("Kepler-452")
        .at_galactic(290.7, 47.5, 430.0)  // l=290.7°, b=47.5°, d=430 pc
        .with_star("G2V")
        .build();

    println!("   Kepler-452: {}",
        CoordinateFormatter::format_galactic(&kepler_452.galactic_coordinates)
    );

    // Using cartesian parsecs from Sun
    let proxima = SystemBuilder::new("Proxima Centauri")
        .at_position_pc(-0.47, -0.36, -1.15)  // Cartesian position in parsecs
        .with_star("M5.5V")
        .build();

    println!("   Proxima Centauri: {}",
        CoordinateFormatter::format_galactic(&proxima.galactic_coordinates)
    );

    // Example 3: Coordinate transformations
    println!("\n3. Coordinate Transformations:");

    let vega_gal = ReferencePositions::vega();
    println!("   Vega in Galactic: {}",
        CoordinateFormatter::format_galactic(&vega_gal)
    );

    // Convert to equatorial
    let vega_eq = CoordinateTransform::galactic_to_equatorial(vega_gal);
    println!("   Vega in Equatorial: {}",
        CoordinateFormatter::format_equatorial(&vega_eq)
    );

    // Show cartesian
    let vega_cart = vega_gal.to_cartesian();
    println!("   Vega in Cartesian: {}",
        CoordinateFormatter::format_cartesian(vega_cart, "pc")
    );

    // Example 4: Converting from Astrosynthesis coordinates
    println!("\n4. Converting from Astrosynthesis (non-standard) to IAU Galactic:");

    // Simulate Astrosynthesis coordinates (in light-years)
    let astro_x = 10.5;   // "right" on the map
    let astro_y = -3.2;   // "up" on the map
    let astro_z = 1.7;    // "out" of the screen

    println!("   Astrosynthesis input: X={}, Y={}, Z={} ly", astro_x, astro_y, astro_z);

    let galactic = CoordinateTransform::astrosynthesis_to_galactic(astro_x, astro_y, astro_z);
    println!("   Converted to IAU Galactic: {}",
        CoordinateFormatter::format_galactic(&galactic)
    );

    // Create a system from Astrosynthesis coordinates
    use solarviewer::stellar_forge::containers::StarSystem;
    use solarviewer::stellar_forge::bodies::StellarBody;

    let imported_system = StarSystem::from_astrosynthesis(
        "Imported System",
        astro_x, astro_y, astro_z,
        StellarBody::new_star("Star",
            solarviewer::stellar_forge::bodies::SpatialParent::Frame(uuid::Uuid::new_v4()),
            "G5V"
        )
    );

    println!("   System '{}' created at: {}",
        imported_system.name,
        CoordinateFormatter::format_galactic(&imported_system.galactic_coordinates)
    );

    // Example 5: Distance calculations
    println!("\n5. Distance Calculations:");

    let sol_pos = ReferencePositions::sol().to_cartesian();
    let alpha_cen_pos = ReferencePositions::alpha_centauri().to_cartesian();

    let distance_m = (alpha_cen_pos - sol_pos).norm();
    let distance_pc = distance_m / solarviewer::stellar_forge::core::Units::PARSEC;
    let distance_ly = distance_m / solarviewer::stellar_forge::core::Units::LIGHT_YEAR;

    println!("   Distance from Sol to Alpha Centauri:");
    println!("     {:.2} pc", distance_pc);
    println!("     {:.2} ly", distance_ly);

    // Example 6: Local Standard of Rest correction
    println!("\n6. Local Standard of Rest (LSR) Corrections:");

    let heliocentric_vel = nalgebra::Vector3::new(20000.0, 230000.0, 7000.0); // m/s
    println!("   Heliocentric velocity: {:.1} km/s",
        CoordinateFormatter::format_cartesian(heliocentric_vel / 1000.0, "km/s")
    );

    let lsr_vel = CoordinateTransform::apply_lsr_correction(heliocentric_vel);
    println!("   LSR-corrected velocity: {:.1} km/s",
        CoordinateFormatter::format_cartesian(lsr_vel / 1000.0, "km/s")
    );

    println!("\n   Solar motion relative to LSR:");
    println!("     U = 11.1 km/s (toward galactic center)");
    println!("     V = 12.24 km/s (in rotation direction)");
    println!("     W = 7.25 km/s (toward NGP)");

    // Example 7: North Galactic Pole
    println!("\n7. Important Reference Directions:");

    let ngp = CoordinateTransform::north_galactic_pole_equatorial();
    println!("   North Galactic Pole (NGP): {}",
        CoordinateFormatter::format_equatorial(&ngp)
    );

    let gc = CoordinateTransform::galactic_center_position();
    println!("   Galactic Center: {}",
        CoordinateFormatter::format_galactic(&gc)
    );

    println!("\n✨ Key Points:");
    println!("   - IAU Galactic: X toward GC, Y toward rotation, Z toward NGP");
    println!("   - Origin is at the Sun (Sol)");
    println!("   - Compatible with real astronomical databases");
    println!("   - Astrosynthesis coordinates are automatically converted");
    println!("   - All standard transformations are supported");
}