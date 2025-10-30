#!/usr/bin/env python3
import sqlite3

conn = sqlite3.connect('TotalSystem.AstroDB')
c = conn.cursor()

print("\n=== MULTI-STAR SYSTEM ANALYSIS ===\n")

# Find containers (system_id = id, no spectral type, but has child components)
c.execute("""
    SELECT id, name, x, y, z
    FROM bodies
    WHERE system_id = id AND parent_id = 0 AND (spectral = '' OR spectral IS NULL)
    AND (SELECT COUNT(*) FROM bodies b2
         WHERE b2.system_id = bodies.id AND b2.parent_id = 0 AND b2.id != bodies.id) > 0
    ORDER BY name
""")

multistar_systems = c.fetchall()
count = 0

for system in multistar_systems:
    sys_id, name, x, y, z = system

    # Count components
    c.execute("""
        SELECT COUNT(*) FROM bodies
        WHERE system_id = ? AND parent_id = 0 AND id != ?
    """, (sys_id, sys_id))
    comp_count = c.fetchone()[0]

    print(f"Multi-Star Container: {name} (ID: {sys_id})")
    print(f"  Position: ({x}, {y}, {z})")
    print(f"  Component Stars: {comp_count}")

    # Get component stars
    c.execute("""
        SELECT name, spectral, radius, mass, luminosity
        FROM bodies
        WHERE system_id = ? AND parent_id = 0 AND id != ?
        ORDER BY name
    """, (sys_id, sys_id))

    components = c.fetchall()
    for comp in components:
        comp_name, spectral, radius, mass, lum = comp
        print(f"    ├─ {comp_name}: {spectral} (R:{radius}, M:{mass}, L:{lum})")
    print()

    count += 1
    if count >= 15:
        print("... (showing first 15 multi-star systems)")
        break

# Summary statistics
c.execute("""
    SELECT COUNT(*) FROM bodies
    WHERE system_id = id AND parent_id = 0 AND (spectral = '' OR spectral IS NULL)
    AND (SELECT COUNT(*) FROM bodies b2
         WHERE b2.system_id = bodies.id AND b2.parent_id = 0 AND b2.id != bodies.id) > 0
""")
total_multistar = c.fetchone()[0]

c.execute("""
    SELECT COUNT(*) FROM bodies
    WHERE system_id = id AND parent_id = 0 AND spectral != '' AND spectral IS NOT NULL
""")
total_singlestar = c.fetchone()[0]

c.execute("""
    SELECT COUNT(*) FROM bodies
    WHERE system_id = id AND parent_id = 0 AND (spectral = '' OR spectral IS NULL)
    AND (SELECT COUNT(*) FROM bodies b2
         WHERE b2.system_id = bodies.id AND b2.parent_id = 0 AND b2.id != bodies.id) = 0
""")
total_empty = c.fetchone()[0]

print("\n=== SUMMARY ===")
print(f"Single-Star Systems: {total_singlestar}")
print(f"Multi-Star Containers: {total_multistar}")
print(f"Empty/Unclassified: {total_empty}")
print(f"Total: {total_singlestar + total_multistar + total_empty}")

conn.close()
