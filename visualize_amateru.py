#!/usr/bin/env python3
"""
2D Star Map Visualization - Amateru Region
Renders stars within 20 ly of Amateru as a 5000x5000 PNG
with lines between stars closer than 10 ly apart.
"""

import sqlite3
import math
from PIL import Image, ImageDraw, ImageFont

# Configuration
OUTPUT_WIDTH = 5000
OUTPUT_HEIGHT = 5000
BACKGROUND_COLOR = (0, 0, 0)  # Black
STAR_COLOR = (255, 255, 255)  # White
STAR_RADIUS = 80
LINE_COLOR = (100, 150, 255)  # Light blue
LINE_WIDTH = 3
TEXT_COLOR = (200, 220, 255)  # Light blue for text
CLOSE_DISTANCE_LY = 10.0  # Draw lines for stars closer than this
REGION_RADIUS_LY = 20.0  # Radius around Amateru

def get_stars_near_amateru(db_path):
    """Get Amateru and all stars within REGION_RADIUS_LY"""
    conn = sqlite3.connect(db_path)
    c = conn.cursor()

    # Get Amateru
    c.execute("""
        SELECT id, name, x, y, z, spectral
        FROM stars WHERE name = 'Amateru'
        LIMIT 1
    """)
    result = c.fetchone()
    if not result:
        # Try with different case
        c.execute("""
            SELECT id, name, x, y, z, spectral
            FROM (SELECT id, name, x, y, z, spectral FROM stars)
            WHERE LOWER(name) = 'amateru'
            LIMIT 1
        """)
        result = c.fetchone()

    if not result:
        print("Error: Could not find Amateru")
        print("Available stars starting with A:")
        c.execute("SELECT name FROM stars ORDER BY name LIMIT 20")
        for row in c.fetchall():
            print(f"  {row[0]}")
        conn.close()
        return None, []

    amateru_id, amateru_name, ax, ay, az, aspec = result
    print(f"Found Amateru: {amateru_name} at ({ax}, {ay}, {az})")
    print(f"  Spectral: {aspec}")

    # Get all stars and calculate distances
    c.execute("""
        SELECT id, name, x, y, z, spectral, radius_solar, mass_solar, luminosity_solar
        FROM stars
        ORDER BY name
    """)

    all_stars = c.fetchall()
    nearby_stars = [result]  # Include Amateru

    for star in all_stars:
        star_id, name, x, y, z, spec, radius, mass, lum = star
        if star_id == amateru_id:
            continue

        # Calculate 3D distance
        dist = math.sqrt((x - ax)**2 + (y - ay)**2 + (z - az)**2)
        if dist <= REGION_RADIUS_LY:
            nearby_stars.append(star)

    conn.close()

    print(f"\nFound {len(nearby_stars)} stars within {REGION_RADIUS_LY} ly of Amateru:")
    for star in nearby_stars[:10]:
        _, name, x, y, z, spec, _, _, _ = star
        dist = math.sqrt((x - ax)**2 + (y - ay)**2 + (z - az)**2)
        print(f"  {name:20} {spec:6} at distance {dist:6.2f} ly")
    if len(nearby_stars) > 10:
        print(f"  ... and {len(nearby_stars) - 10} more")

    return (amateru_id, amateru_name, ax, ay, az), nearby_stars

def calculate_distances(amateru, stars):
    """Calculate 3D distances between all pairs of stars"""
    ax, ay, az = amateru[2], amateru[3], amateru[4]
    distances = {}

    for i, star1 in enumerate(stars):
        for j, star2 in enumerate(stars):
            if i >= j:
                continue

            x1, y1, z1 = star1[2], star1[3], star1[4]
            x2, y2, z2 = star2[2], star2[3], star2[4]

            dist = math.sqrt((x2 - x1)**2 + (y2 - y1)**2 + (z2 - z1)**2)
            distances[(i, j)] = dist

    return distances

def project_to_2d(stars):
    """
    Project 3D star positions to 2D using simple orthographic projection.
    Find bounding box and scale to fit the output image.
    """
    if not stars:
        return []

    # Extract 2D coordinates (X and Y, drop Z)
    positions_2d = [(s[2], s[3]) for s in stars]

    # Calculate bounds
    min_x = min(p[0] for p in positions_2d)
    max_x = max(p[0] for p in positions_2d)
    min_y = min(p[1] for p in positions_2d)
    max_y = max(p[1] for p in positions_2d)

    range_x = max_x - min_x
    range_y = max_y - min_y

    # Add 10% padding
    padding = 0.1
    range_x *= (1 + padding)
    range_y *= (1 + padding)

    center_x = (min_x + max_x) / 2
    center_y = (min_y + max_y) / 2

    # Scale to output dimensions (leave margin for stars and labels)
    margin = 300
    available_width = OUTPUT_WIDTH - 2 * margin
    available_height = OUTPUT_HEIGHT - 2 * margin

    if range_x > 0 and range_y > 0:
        scale = min(available_width / range_x, available_height / range_y)
    else:
        scale = 1.0

    # Convert to pixel coordinates
    pixel_positions = []
    for pos in positions_2d:
        px = (pos[0] - center_x) * scale + OUTPUT_WIDTH / 2
        py = (pos[1] - center_y) * scale + OUTPUT_HEIGHT / 2
        pixel_positions.append((px, py))

    return pixel_positions

def resolve_overlaps(pixel_positions, min_distance=150):
    """
    Resolve overlapping stars by applying simple repulsive forces.
    Iteratively push stars apart if they're too close.
    """
    positions = [list(p) for p in pixel_positions]

    max_iterations = 50
    for iteration in range(max_iterations):
        moved = False

        for i in range(len(positions)):
            for j in range(i + 1, len(positions)):
                dx = positions[j][0] - positions[i][0]
                dy = positions[j][1] - positions[i][1]
                dist = math.sqrt(dx**2 + dy**2)

                if dist < min_distance and dist > 0:
                    # Push stars apart
                    force = (min_distance - dist) / 2
                    norm_x = dx / dist
                    norm_y = dy / dist

                    positions[i][0] -= norm_x * force
                    positions[i][1] -= norm_y * force
                    positions[j][0] += norm_x * force
                    positions[j][1] += norm_y * force

                    moved = True

        if not moved:
            break

    return [(float(p[0]), float(p[1])) for p in positions]

def render_map(amateru, nearby_stars, distances, pixel_positions):
    """Render the star map to PNG"""
    # Create image
    img = Image.new('RGB', (OUTPUT_WIDTH, OUTPUT_HEIGHT), BACKGROUND_COLOR)
    draw = ImageDraw.Draw(img)

    ax_id = amateru[0]

    # Draw lines between nearby stars (< 10 ly)
    for (i, j), dist_ly in distances.items():
        if dist_ly < CLOSE_DISTANCE_LY:
            x1, y1 = pixel_positions[i]
            x2, y2 = pixel_positions[j]

            # Vary line width based on distance
            if dist_ly < 5:
                width = 5
            elif dist_ly < 8:
                width = 4
            else:
                width = 3

            draw.line([(x1, y1), (x2, y2)], fill=LINE_COLOR, width=width)

    # Draw stars
    for i, star in enumerate(nearby_stars):
        star_id, name, x, y, z, spec, radius, mass, lum = star
        px, py = pixel_positions[i]

        # Vary size by luminosity (cube root to make differences visible)
        size_factor = (lum ** (1/3)) if lum > 0 else 1.0
        size_factor = min(size_factor, 3.0)  # Cap at 3x
        radius = STAR_RADIUS * size_factor

        # Highlight Amateru
        if star_id == ax_id:
            # Draw larger circle with different color for Amateru
            draw.ellipse(
                [px - radius - 10, py - radius - 10,
                 px + radius + 10, py + radius + 10],
                fill=(255, 200, 0), outline=(255, 255, 0), width=5
            )

        # Draw star
        draw.ellipse(
            [px - radius, py - radius, px + radius, py + radius],
            fill=STAR_COLOR, outline=TEXT_COLOR, width=2
        )

    # Try to load a font, fall back to default
    try:
        font = ImageFont.truetype("arial.ttf", 40)
        small_font = ImageFont.truetype("arial.ttf", 30)
    except:
        font = ImageFont.load_default()
        small_font = font

    # Draw star names
    for i, star in enumerate(nearby_stars):
        star_id, name, x, y, z, spec, radius, mass, lum = star
        px, py = pixel_positions[i]

        # Offset text below star
        text_y = py + STAR_RADIUS + 50

        # Draw name and spectral type
        label = f"{name}\n{spec}"

        # For Amateru, highlight it
        if star_id == ax_id:
            text_color = (255, 200, 0)
            draw.text((px - 100, text_y), name, fill=text_color, font=font)
            draw.text((px - 80, text_y + 50), spec, fill=text_color, font=small_font)
        else:
            draw.text((px - 100, text_y), name, fill=TEXT_COLOR, font=small_font)
            draw.text((px - 80, text_y + 40), spec, fill=TEXT_COLOR, font=small_font)

    # Draw title and info
    title = f"Stars Near Amateru (within {REGION_RADIUS_LY} ly)"
    draw.text((50, 50), title, fill=(255, 255, 255), font=font)

    info = f"{len(nearby_stars)} stars | Lines: {CLOSE_DISTANCE_LY} ly | Scale: ~1 ly per pixel"
    draw.text((50, 120), info, fill=(150, 150, 150), font=small_font)

    # Save
    output_path = "amateru_map.png"
    img.save(output_path)
    print(f"\nMap saved to: {output_path}")
    print(f"Image size: {OUTPUT_WIDTH}x{OUTPUT_HEIGHT}")
    print(f"Stars rendered: {len(nearby_stars)}")
    print(f"Connections drawn: {len([(d for d in distances.values() if d < CLOSE_DISTANCE_LY)])}")

def main():
    db_path = "TotalSystem.AstroDB"

    print(f"Visualizing stars near Amateru from {db_path}")
    print(f"Output: 5000x5000 PNG with black background\n")

    # Get data
    amateru, nearby_stars = get_stars_near_amateru(db_path)
    if not amateru:
        return

    # Calculate distances
    distances = calculate_distances(amateru, nearby_stars)

    # Project to 2D
    print(f"\nProjecting {len(nearby_stars)} stars to 2D...")
    pixel_positions = project_to_2d(nearby_stars)

    # Resolve overlaps
    print("Resolving star overlaps...")
    pixel_positions = resolve_overlaps(pixel_positions, min_distance=150)

    # Render
    print("Rendering map...")
    render_map(amateru, nearby_stars, distances, pixel_positions)

if __name__ == "__main__":
    main()
