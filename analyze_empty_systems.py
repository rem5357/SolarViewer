#!/usr/bin/env python3
import sqlite3

conn = sqlite3.connect('TotalSystem.AstroDB')
c = conn.cursor()

print("\n=== ANALYZING EMPTY STAR CONTAINERS ===\n")

# Find all empty containers (no spectral type, system_id = id)
c.execute("""
    SELECT id, name, x, y, z
    FROM bodies
    WHERE system_id = id AND parent_id = 0 AND (spectral = '' OR spectral IS NULL)
    ORDER BY name
    LIMIT 20
""")

empty_systems = c.fetchall()

for sys_id, name, x, y, z in empty_systems:
    print(f"\nEmpty Container: {name} (ID: {sys_id})")
    print(f"  Position: ({x}, {y}, {z})")

    # Check for ANY bodies that reference this system
    c.execute("""
        SELECT COUNT(*),
               COUNT(DISTINCT parent_id) as unique_parents,
               GROUP_CONCAT(DISTINCT body_type)
        FROM bodies
        WHERE system_id = ?
    """, (sys_id,))

    count, unique_parents, body_types = c.fetchone()
    print(f"  Total bodies in system: {count}")
    print(f"  Unique parent_id values: {unique_parents}")
    if body_types:
        print(f"  Body types: {body_types}")

    # List the bodies
    c.execute("""
        SELECT id, name, parent_id, body_type, spectral
        FROM bodies
        WHERE system_id = ?
        ORDER BY parent_id, name
    """, (sys_id,))

    bodies = c.fetchall()
    for body_id, body_name, parent_id, body_type, spectral in bodies:
        marker = "[STAR]" if spectral and spectral.strip() else ""
        print(f"    -> {body_name} (ID:{body_id}, Parent:{parent_id}, Type:{body_type}) {marker}")


# Summary of ALL system types
print("\n\n=== SYSTEM CLASSIFICATION ===\n")

c.execute("""
    SELECT
        CASE
            WHEN spectral != '' AND spectral IS NOT NULL THEN 'Single Star'
            WHEN (SELECT COUNT(*) FROM bodies b2 WHERE b2.system_id = bodies.id AND b2.id != bodies.id AND b2.spectral != '' AND b2.spectral IS NOT NULL) > 0
                THEN 'Multi-Star Container'
            WHEN (SELECT COUNT(*) FROM bodies b2 WHERE b2.system_id = bodies.id AND b2.id != bodies.id) > 0
                THEN 'Complex System'
            ELSE 'Empty/Unknown'
        END as system_type,
        COUNT(*) as count
    FROM bodies
    WHERE system_id = id AND parent_id = 0
    GROUP BY system_type
    ORDER BY count DESC
""")

results = c.fetchall()
total = 0
for sys_type, count in results:
    print(f"{sys_type}: {count}")
    total += count

print(f"\nTotal: {total}")

conn.close()
