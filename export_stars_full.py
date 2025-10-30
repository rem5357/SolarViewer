#!/usr/bin/env python3
"""
Export all stars to CSV, including both single-star systems and multi-star components.
"""
import sqlite3

conn = sqlite3.connect('TotalSystem.AstroDB')
c = conn.cursor()

# Create CSV output
with open('stars_complete.csv', 'w', encoding='utf-8') as f:
    # Write header
    f.write("Name,Spectral Type,Radius (Solar),Mass (Solar),Luminosity (Solar),Temperature (K),Star X,Star Y,Star Z,System Name,System X,System Y,System Z\n")

    # Get single-star systems
    c.execute("""
        SELECT id, name, spectral, radius, mass, luminosity, temp, x, y, z
        FROM bodies
        WHERE system_id = id AND parent_id = 0 AND spectral != '' AND spectral IS NOT NULL
        ORDER BY name
    """)

    single_stars = c.fetchall()
    for body_id, name, spectral, radius, mass, lum, temp, x, y, z in single_stars:
        # Escape quotes in names
        name_escaped = name.replace('"', '""')
        f.write(f'"{name_escaped}","{spectral}",{radius},{mass},{lum},{temp},{x},{y},{z},"",{x},{y},{z}\n')

    # Get multi-star component stars
    c.execute("""
        SELECT b.id, b.name, b.spectral, b.radius, b.mass, b.luminosity, b.temp,
                b.x, b.y, b.z, c.name, c.x, c.y, c.z
        FROM bodies b
        JOIN bodies c ON b.parent_id = c.id
        WHERE c.system_id = c.id AND c.parent_id = 0
        AND (c.spectral = '' OR c.spectral IS NULL)
        AND b.spectral != '' AND b.spectral IS NOT NULL
        AND b.parent_id = c.id
        ORDER BY c.name, b.name
    """)

    multi_stars = c.fetchall()
    for b_id, name, spectral, radius, mass, lum, temp, x, y, z, sys_name, sys_x, sys_y, sys_z in multi_stars:
        # Escape quotes in names
        name_escaped = name.replace('"', '""')
        sys_name_escaped = sys_name.replace('"', '""')
        f.write(f'"{name_escaped}","{spectral}",{radius},{mass},{lum},{temp},{x},{y},{z},"{sys_name_escaped}",{sys_x},{sys_y},{sys_z}\n')

print(f"Exported {len(single_stars)} single-star systems")
print(f"Exported {len(multi_stars)} multi-star components")
print(f"Total: {len(single_stars) + len(multi_stars)} stars")
print("\nCSV saved to: stars_complete.csv")

conn.close()
