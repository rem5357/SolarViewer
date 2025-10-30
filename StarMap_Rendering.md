# Star Map 2D Rendering - Visual Implementation Guide

## Overview

This skill document provides detailed specifications for rendering beautiful, professional-quality 2D star maps from 3D stellar coordinate data. The implementation uses Rust with modern graphics libraries to create both screen-optimized (dark) and print-optimized (light) versions at high resolution (5000x5000 pixels).

## Core Design Philosophy

**Procedural over Static:**
- Generate star symbols procedurally for infinite variety
- No pre-rendered assets to manage
- Perfect scaling at any resolution
- Consistent visual language

**Cartographic Clarity:**
- Function over decoration
- Clear visual hierarchy
- Readable at multiple scales
- Professional scientific aesthetic

**Dual-Purpose Output:**
- Dark background for screen viewing (deep space aesthetic)
- White background for printing (traditional cartography)
- Same layout, optimized styling for each medium

## Rust Crates and Dependencies

### Essential Graphics Libraries

```toml
[dependencies]
# Core 2D rendering
tiny-skia = "0.11"              # Lightweight 2D rendering (no system dependencies)
# OR
skia-safe = "0.75"              # Full-featured Skia bindings (more powerful, larger)

# Font rendering
fontdue = "0.9"                  # Pure Rust, fast font rasterization
# OR  
ab_glyph = "0.2"                # Alternative font rendering

# Image encoding
png = "0.17"                     # PNG encoding/decoding

# Math and geometry
nalgebra = "0.33"                # Linear algebra for transformations
euclid = "0.22"                  # 2D geometry primitives

# Color manipulation
palette = "0.7"                  # Color space conversions and manipulation

# Random for procedural generation
rand = "0.8"                     # Random number generation
rand_chacha = "0.3"              # Deterministic RNG with seed
```

### Recommended Combination

**For simplicity and reliability:**
```toml
tiny-skia = "0.11"
fontdue = "0.9"
png = "0.17"
nalgebra = "0.33"
palette = "0.7"
rand_chacha = "0.3"
```

This combination:
- Has no system dependencies (pure Rust)
- Works cross-platform without issues
- Sufficient for all rendering needs
- Good performance
- Easy to integrate

## High-Level Architecture

### Rendering Pipeline Structure

```rust
pub struct StarMapRenderer {
    width: u32,
    height: u32,
    pixmap: tiny_skia::Pixmap,
    theme: RenderTheme,
}

pub enum RenderTheme {
    Dark,   // Screen viewing - black background
    Light,  // Print - white background
}

pub struct Star {
    pub name: String,
    pub x: f32,        // 2D position on map
    pub y: f32,
    pub z: f32,        // Original 3D Z for depth encoding
    pub spectral_type: SpectralType,
    pub luminosity: f32,
}

pub struct Connection {
    pub from_star: usize,  // Index into star array
    pub to_star: usize,
    pub distance_ly: f32,  // Real 3D distance
}

pub enum SpectralType {
    O,  // Blue
    B,  // Blue-white
    A,  // White
    F,  // Yellow-white
    G,  // Yellow (like Sol)
    K,  // Orange
    M,  // Red
}
```

### Rendering Layers (Bottom to Top)

```rust
impl StarMapRenderer {
    pub fn render(&mut self, stars: &[Star], connections: &[Connection]) {
        // Layer 1: Background
        self.render_background();
        
        // Layer 2: Optional grid
        if self.config.show_grid {
            self.render_coordinate_grid();
        }
        
        // Layer 3: Connection lines
        self.render_connections(stars, connections);
        
        // Layer 4: Star symbols
        self.render_stars(stars);
        
        // Layer 5: Labels
        self.render_labels(stars, connections);
        
        // Layer 6: Legend and border
        self.render_legend();
        self.render_border();
    }
}
```

## Detailed Rendering Specifications

### 1. Background Layer

**Dark Theme (Screen):**
```rust
fn render_background_dark(&mut self) {
    // Deep space gradient: darker at edges, slightly lighter in center
    let center_color = tiny_skia::Color::from_rgba8(0, 24, 48, 255);     // #001830
    let edge_color = tiny_skia::Color::from_rgba8(0, 8, 20, 255);        // #000814
    
    // Create radial gradient from center
    let gradient = create_radial_gradient(
        center_x: self.width as f32 / 2.0,
        center_y: self.height as f32 / 2.0,
        center_color,
        edge_color,
    );
    
    // Fill entire canvas
    self.pixmap.fill(edge_color);
    
    // Optional: Add subtle star field background
    self.render_background_stars(density: 0.0001);  // Very sparse, very dim
}

fn render_background_stars(&mut self, density: f32) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);  // Fixed seed for consistency
    let num_stars = (self.width * self.height) as f32 * density;
    
    for _ in 0..num_stars as u32 {
        let x = rng.gen_range(0..self.width) as f32;
        let y = rng.gen_range(0..self.height) as f32;
        let brightness = rng.gen_range(10..30);  // Very dim
        
        // Single pixel "star"
        let color = tiny_skia::Color::from_rgba8(
            brightness * 2, brightness * 2, brightness * 3, 255
        );
        
        self.draw_pixel(x as u32, y as u32, color);
    }
}
```

**Light Theme (Print):**
```rust
fn render_background_light(&mut self) {
    // Clean white or very light gray
    let bg_color = tiny_skia::Color::from_rgba8(255, 255, 255, 255);  // Pure white
    // OR slightly off-white for softer print appearance:
    // let bg_color = tiny_skia::Color::from_rgba8(248, 249, 250, 255);  // #F8F9FA
    
    self.pixmap.fill(bg_color);
}
```

### 2. Coordinate Grid (Optional)

```rust
fn render_coordinate_grid(&mut self) {
    let grid_spacing_px = 500.0;  // 500 pixels between grid lines
    
    let grid_color = match self.theme {
        RenderTheme::Dark => Color::from_rgba8(40, 50, 60, 255),    // Subtle blue-gray
        RenderTheme::Light => Color::from_rgba8(220, 220, 220, 255), // Light gray
    };
    
    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = 1.0;
    stroke.line_cap = tiny_skia::LineCap::Round;
    
    // Vertical lines
    let mut x = 0.0;
    while x <= self.width as f32 {
        self.draw_line(x, 0.0, x, self.height as f32, grid_color, &stroke);
        x += grid_spacing_px;
    }
    
    // Horizontal lines
    let mut y = 0.0;
    while y <= self.height as f32 {
        self.draw_line(0.0, y, self.width as f32, y, grid_color, &stroke);
        y += grid_spacing_px;
    }
}
```

### 3. Connection Lines

**Visual Hierarchy by Distance:**

```rust
struct LineStyle {
    stroke_width: f32,
    dash_pattern: Option<Vec<f32>>,
    opacity: u8,
    color: Color,
}

impl StarMapRenderer {
    fn get_line_style(&self, distance_ly: f32) -> LineStyle {
        match self.theme {
            RenderTheme::Dark => self.get_line_style_dark(distance_ly),
            RenderTheme::Light => self.get_line_style_light(distance_ly),
        }
    }
    
    fn get_line_style_dark(&self, distance_ly: f32) -> LineStyle {
        if distance_ly < 5.0 {
            // Close: Solid, bright, thick
            LineStyle {
                stroke_width: 3.0,
                dash_pattern: None,
                opacity: 255,
                color: Color::from_rgba8(120, 140, 160, 255),  // Light blue-gray
            }
        } else if distance_ly < 10.0 {
            // Moderate: Solid, medium
            LineStyle {
                stroke_width: 2.0,
                dash_pattern: None,
                opacity: 180,
                color: Color::from_rgba8(100, 120, 140, 180),
            }
        } else if distance_ly < 15.0 {
            // Far: Dashed
            LineStyle {
                stroke_width: 1.5,
                dash_pattern: Some(vec![5.0, 3.0]),
                opacity: 130,
                color: Color::from_rgba8(80, 100, 120, 130),
            }
        } else if distance_ly < 20.0 {
            // Very far: Dotted
            LineStyle {
                stroke_width: 1.5,
                dash_pattern: Some(vec![2.0, 3.0]),
                opacity: 80,
                color: Color::from_rgba8(60, 80, 100, 80),
            }
        } else {
            // Too far: Don't render by default
            return LineStyle::invisible();
        }
    }
    
    fn get_line_style_light(&self, distance_ly: f32) -> LineStyle {
        // Similar but with darker colors for white background
        if distance_ly < 5.0 {
            LineStyle {
                stroke_width: 3.0,
                dash_pattern: None,
                opacity: 255,
                color: Color::from_rgba8(60, 70, 80, 255),  // Dark gray-blue
            }
        } else if distance_ly < 10.0 {
            LineStyle {
                stroke_width: 2.0,
                dash_pattern: None,
                opacity: 220,
                color: Color::from_rgba8(80, 90, 100, 220),
            }
        } else if distance_ly < 15.0 {
            LineStyle {
                stroke_width: 1.5,
                dash_pattern: Some(vec![5.0, 3.0]),
                opacity: 180,
                color: Color::from_rgba8(100, 110, 120, 180),
            }
        } else if distance_ly < 20.0 {
            LineStyle {
                stroke_width: 1.5,
                dash_pattern: Some(vec![2.0, 3.0]),
                opacity: 140,
                color: Color::from_rgba8(120, 130, 140, 140),
            }
        } else {
            return LineStyle::invisible();
        }
    }
}
```

**Drawing Lines with Anti-aliasing:**

```rust
fn render_connections(&mut self, stars: &[Star], connections: &[Connection]) {
    for conn in connections {
        let from = &stars[conn.from_star];
        let to = &stars[conn.to_star];
        
        let style = self.get_line_style(conn.distance_ly);
        if style.is_invisible() {
            continue;
        }
        
        // Create stroke with style
        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = style.stroke_width;
        stroke.line_cap = tiny_skia::LineCap::Round;
        stroke.line_join = tiny_skia::LineJoin::Round;
        
        if let Some(dash) = style.dash_pattern {
            stroke.dash = tiny_skia::StrokeDash::new(dash, 0.0);
        }
        
        // Draw line
        let mut path = tiny_skia::PathBuilder::new();
        path.move_to(from.x, from.y);
        path.line_to(to.x, to.y);
        let path = path.finish().unwrap();
        
        let paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(style.color),
            anti_alias: true,  // CRITICAL for smooth lines
            ..Default::default()
        };
        
        self.pixmap.stroke_path(&path, &paint, &stroke, 
            tiny_skia::Transform::identity(), None);
    }
}
```

### 4. Star Symbol Rendering (Procedural)

**Color Mapping by Spectral Type:**

```rust
fn get_spectral_color(&self, spectral_type: SpectralType) -> (Color, Color) {
    // Returns (core_color, glow_color)
    match spectral_type {
        SpectralType::O => (
            Color::from_rgba8(155, 176, 255, 255),  // Bright blue core
            Color::from_rgba8(100, 130, 255, 0),    // Blue glow (transparent at edge)
        ),
        SpectralType::B => (
            Color::from_rgba8(170, 191, 255, 255),  // Blue-white
            Color::from_rgba8(120, 150, 255, 0),
        ),
        SpectralType::A => (
            Color::from_rgba8(202, 215, 255, 255),  // White
            Color::from_rgba8(180, 200, 255, 0),
        ),
        SpectralType::F => (
            Color::from_rgba8(248, 247, 255, 255),  // Yellow-white
            Color::from_rgba8(240, 230, 200, 0),
        ),
        SpectralType::G => (
            Color::from_rgba8(255, 244, 234, 255),  // Yellow (Sol-like)
            Color::from_rgba8(255, 220, 150, 0),
        ),
        SpectralType::K => (
            Color::from_rgba8(255, 210, 161, 255),  // Orange
            Color::from_rgba8(255, 180, 100, 0),
        ),
        SpectralType::M => (
            Color::from_rgba8(255, 204, 111, 255),  // Red-orange
            Color::from_rgba8(255, 150, 80, 0),
        ),
    }
}
```

**Procedural Star Rendering:**

```rust
fn render_star(&mut self, star: &Star, seed: u64) {
    // Use star's unique ID/name as seed for consistent appearance
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    // Calculate size based on luminosity (scaled for visibility)
    let base_radius = 8.0 + (star.luminosity.log10() * 4.0).clamp(0.0, 12.0);
    
    // Slight random variation (±10%) for visual interest
    let radius = base_radius * rng.gen_range(0.9..1.1);
    
    let (core_color, glow_color) = self.get_spectral_color_for_theme(
        star.spectral_type, 
        self.theme
    );
    
    // Adjust colors for Z-depth (optional depth encoding)
    let depth_factor = (star.z / 20.0).clamp(-1.0, 1.0);
    let core_color = self.apply_depth_tint(core_color, depth_factor);
    
    // Layer 1: Outer glow (largest)
    self.draw_radial_glow(
        star.x, 
        star.y, 
        radius * 2.5,    // Glow extends 2.5x beyond core
        glow_color,
        0.3              // Maximum opacity at center
    );
    
    // Layer 2: Inner glow (medium)
    self.draw_radial_glow(
        star.x, 
        star.y, 
        radius * 1.8,
        core_color,
        0.6
    );
    
    // Layer 3: Core circle (solid)
    self.draw_filled_circle(
        star.x, 
        star.y, 
        radius,
        core_color
    );
    
    // Layer 4: Corona rays (spikes)
    let num_rays = rng.gen_range(4..9);  // 4-8 rays
    let ray_rotation = rng.gen_range(0.0..360.0);  // Random base rotation
    
    for i in 0..num_rays {
        let angle = (i as f32 / num_rays as f32) * 360.0 + ray_rotation;
        let ray_length = radius * rng.gen_range(1.5..2.2);
        self.draw_corona_ray(
            star.x, 
            star.y, 
            radius,
            ray_length,
            angle,
            core_color
        );
    }
    
    // Layer 5: Subtle center highlight
    self.draw_filled_circle(
        star.x,
        star.y,
        radius * 0.3,
        Color::from_rgba8(255, 255, 255, 180)  // Bright white center
    );
}

fn draw_radial_glow(&mut self, cx: f32, cy: f32, radius: f32, 
                    color: Color, max_opacity: f32) {
    // Create radial gradient
    let gradient = tiny_skia::RadialGradient::new(
        tiny_skia::Point::from_xy(cx, cy),  // Center
        tiny_skia::Point::from_xy(cx + radius, cy),  // Edge
        vec![
            tiny_skia::GradientStop::new(0.0, color.with_opacity(max_opacity)),
            tiny_skia::GradientStop::new(0.7, color.with_opacity(max_opacity * 0.3)),
            tiny_skia::GradientStop::new(1.0, color.with_opacity(0.0)),
        ],
        tiny_skia::SpreadMode::Pad,
        tiny_skia::Transform::identity(),
    ).unwrap();
    
    let shader = tiny_skia::Shader::RadialGradient(gradient);
    let paint = tiny_skia::Paint {
        shader,
        anti_alias: true,
        ..Default::default()
    };
    
    // Draw circle with gradient
    let mut path = tiny_skia::PathBuilder::new();
    path.push_circle(cx, cy, radius);
    let path = path.finish().unwrap();
    
    self.pixmap.fill_path(&path, &paint, 
        tiny_skia::FillRule::Winding,
        tiny_skia::Transform::identity(), 
        None);
}

fn draw_corona_ray(&mut self, cx: f32, cy: f32, 
                   inner_radius: f32, outer_length: f32, 
                   angle_degrees: f32, color: Color) {
    let angle = angle_degrees.to_radians();
    
    // Start point (at edge of core circle)
    let start_x = cx + inner_radius * angle.cos();
    let start_y = cy + inner_radius * angle.sin();
    
    // End point (ray extends outward)
    let end_x = cx + (inner_radius + outer_length) * angle.cos();
    let end_y = cy + (inner_radius + outer_length) * angle.sin();
    
    // Create gradient along ray (solid to transparent)
    let gradient = create_linear_gradient_along_line(
        start_x, start_y,
        end_x, end_y,
        color.with_opacity(0.8),
        color.with_opacity(0.0),
    );
    
    // Draw ray as wide line (tapered)
    let ray_width = inner_radius * 0.3;
    
    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = ray_width;
    stroke.line_cap = tiny_skia::LineCap::Round;
    
    let mut path = tiny_skia::PathBuilder::new();
    path.move_to(start_x, start_y);
    path.line_to(end_x, end_y);
    let path = path.finish().unwrap();
    
    let paint = tiny_skia::Paint {
        shader: gradient,
        anti_alias: true,
        ..Default::default()
    };
    
    self.pixmap.stroke_path(&path, &paint, &stroke,
        tiny_skia::Transform::identity(), None);
}

fn draw_filled_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Color) {
    let mut path = tiny_skia::PathBuilder::new();
    path.push_circle(cx, cy, radius);
    let path = path.finish().unwrap();
    
    let paint = tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        anti_alias: true,
        ..Default::default()
    };
    
    self.pixmap.fill_path(&path, &paint,
        tiny_skia::FillRule::Winding,
        tiny_skia::Transform::identity(),
        None);
}
```

**Depth Encoding (Optional):**

```rust
fn apply_depth_tint(&self, base_color: Color, depth_factor: f32) -> Color {
    // depth_factor: -1.0 (far) to +1.0 (near)
    
    if depth_factor > 0.0 {
        // Nearer stars: slightly warmer (shift toward red/orange)
        let r = (base_color.red() as f32 + depth_factor * 20.0).clamp(0.0, 255.0);
        let g = base_color.green() as f32;
        let b = (base_color.blue() as f32 - depth_factor * 10.0).clamp(0.0, 255.0);
        Color::from_rgba8(r as u8, g as u8, b as u8, base_color.alpha())
    } else {
        // Farther stars: slightly cooler (shift toward blue)
        let r = (base_color.red() as f32 + depth_factor * 10.0).clamp(0.0, 255.0);
        let g = base_color.green() as f32;
        let b = (base_color.blue() as f32 - depth_factor * 20.0).clamp(0.0, 255.0);
        Color::from_rgba8(r as u8, g as u8, b as u8, base_color.alpha())
    }
}
```

### 5. Text Labels

**Label Rendering with Background:**

```rust
struct FontRenderer {
    font: fontdue::Font,
}

impl FontRenderer {
    fn new(font_bytes: &[u8]) -> Self {
        let font = fontdue::Font::from_bytes(
            font_bytes, 
            fontdue::FontSettings::default()
        ).expect("Failed to load font");
        
        Self { font }
    }
    
    fn render_label_with_background(
        &self,
        pixmap: &mut tiny_skia::Pixmap,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        text_color: Color,
        bg_color: Color,
        theme: RenderTheme,
    ) {
        // Rasterize text
        let metrics_and_bitmaps: Vec<_> = text
            .chars()
            .map(|c| self.font.rasterize(c, font_size))
            .collect();
        
        // Calculate total text width and height
        let total_width: f32 = metrics_and_bitmaps
            .iter()
            .map(|(m, _)| m.advance_width)
            .sum();
        let max_height = metrics_and_bitmaps
            .iter()
            .map(|(m, _)| m.height as f32)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        // Draw background box with padding
        let padding = 4.0;
        let bg_alpha = match theme {
            RenderTheme::Dark => 200,   // Semi-transparent on dark
            RenderTheme::Light => 230,  // More opaque on light
        };
        
        let bg_color_with_alpha = Color::from_rgba8(
            bg_color.red(),
            bg_color.green(),
            bg_color.blue(),
            bg_alpha,
        );
        
        self.draw_rounded_rect(
            pixmap,
            x - padding,
            y - max_height - padding,
            total_width + padding * 2.0,
            max_height + padding * 2.0,
            2.0,  // Corner radius
            bg_color_with_alpha,
        );
        
        // Draw text characters
        let mut cursor_x = x;
        for (metrics, bitmap) in metrics_and_bitmaps {
            self.draw_glyph_bitmap(
                pixmap,
                &bitmap,
                cursor_x,
                y - metrics.height as f32,
                metrics.width,
                metrics.height,
                text_color,
            );
            cursor_x += metrics.advance_width;
        }
    }
}
```

**Star Name Labels:**

```rust
fn render_star_label(&mut self, star: &Star, font_renderer: &FontRenderer) {
    let label_offset_x = 15.0;  // Offset from star center
    let label_offset_y = -5.0;
    
    let (text_color, bg_color) = match self.theme {
        RenderTheme::Dark => (
            Color::from_rgba8(220, 230, 240, 255),  // Light text
            Color::from_rgba8(10, 20, 30, 200),     // Dark background
        ),
        RenderTheme::Light => (
            Color::from_rgba8(20, 30, 40, 255),     // Dark text
            Color::from_rgba8(240, 245, 250, 230),  // Light background
        ),
    };
    
    // Main star name
    font_renderer.render_label_with_background(
        &mut self.pixmap,
        &star.name,
        star.x + label_offset_x,
        star.y + label_offset_y,
        14.0,  // Font size
        text_color,
        bg_color,
        self.theme,
    );
    
    // Z-coordinate label (smaller, below name)
    let z_label = format!("[z: {:.1}]", star.z);
    font_renderer.render_label_with_background(
        &mut self.pixmap,
        &z_label,
        star.x + label_offset_x,
        star.y + label_offset_y + 18.0,
        10.0,  // Smaller font
        text_color.with_opacity(180),
        bg_color.with_opacity(150),
        self.theme,
    );
}
```

**Distance Labels on Connections:**

```rust
fn render_connection_label(
    &mut self, 
    from: &Star, 
    to: &Star, 
    distance_ly: f32,
    font_renderer: &FontRenderer
) {
    // Calculate midpoint
    let mid_x = (from.x + to.x) / 2.0;
    let mid_y = (from.y + to.y) / 2.0;
    
    // Format distance
    let label = format!("{:.1} LY", distance_ly);
    
    let (text_color, bg_color) = match self.theme {
        RenderTheme::Dark => (
            Color::from_rgba8(180, 200, 220, 255),
            Color::from_rgba8(20, 30, 40, 220),
        ),
        RenderTheme::Light => (
            Color::from_rgba8(60, 70, 80, 255),
            Color::from_rgba8(250, 252, 255, 240),
        ),
    };
    
    // Optional: rotate label to align with line
    // For simplicity, always horizontal
    font_renderer.render_label_with_background(
        &mut self.pixmap,
        &label,
        mid_x,
        mid_y - 8.0,  // Slightly above midpoint
        10.0,  // Small font for distances
        text_color,
        bg_color,
        self.theme,
    );
}
```

### 6. Border and Legend

**Border Frame:**

```rust
fn render_border(&mut self) {
    let border_width = 12.0;
    let corner_radius = 16.0;
    
    let border_color = match self.theme {
        RenderTheme::Dark => {
            // Metallic blue-gray
            Color::from_rgba8(60, 80, 120, 255)
        },
        RenderTheme::Light => {
            // Traditional black border
            Color::from_rgba8(0, 0, 0, 255)
        },
    };
    
    // Outer border (rounded rectangle)
    self.draw_rounded_rect_stroke(
        border_width / 2.0,
        border_width / 2.0,
        self.width as f32 - border_width,
        self.height as f32 - border_width,
        corner_radius,
        border_color,
        border_width,
    );
    
    // Optional: Inner shadow for depth (dark theme only)
    if matches!(self.theme, RenderTheme::Dark) {
        self.draw_inner_shadow(
            border_width,
            border_width,
            self.width as f32 - border_width * 2.0,
            self.height as f32 - border_width * 2.0,
            corner_radius - border_width / 2.0,
        );
    }
}

fn draw_rounded_rect_stroke(
    &mut self,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    corner_radius: f32,
    color: Color,
    stroke_width: f32,
) {
    let mut path = tiny_skia::PathBuilder::new();
    
    // Top-left corner
    path.move_to(x + corner_radius, y);
    path.line_to(x + width - corner_radius, y);
    path.quad_to(x + width, y, x + width, y + corner_radius);
    
    // Right side
    path.line_to(x + width, y + height - corner_radius);
    path.quad_to(x + width, y + height, x + width - corner_radius, y + height);
    
    // Bottom
    path.line_to(x + corner_radius, y + height);
    path.quad_to(x, y + height, x, y + height - corner_radius);
    
    // Left side
    path.line_to(x, y + corner_radius);
    path.quad_to(x, y, x + corner_radius, y);
    
    let path = path.finish().unwrap();
    
    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = stroke_width;
    stroke.line_cap = tiny_skia::LineCap::Round;
    stroke.line_join = tiny_skia::LineJoin::Round;
    
    let paint = tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        anti_alias: true,
        ..Default::default()
    };
    
    self.pixmap.stroke_path(&path, &paint, &stroke,
        tiny_skia::Transform::identity(), None);
}
```

**Legend:**

```rust
fn render_legend(&mut self, font_renderer: &FontRenderer) {
    let legend_x = self.width as f32 - 300.0;
    let legend_y = 50.0;
    let legend_width = 250.0;
    let legend_height = 400.0;
    
    // Semi-transparent background
    let bg_alpha = match self.theme {
        RenderTheme::Dark => 200,
        RenderTheme::Light => 240,
    };
    
    let bg_color = match self.theme {
        RenderTheme::Dark => Color::from_rgba8(10, 20, 30, bg_alpha),
        RenderTheme::Light => Color::from_rgba8(245, 248, 252, bg_alpha),
    };
    
    self.draw_rounded_rect(
        legend_x,
        legend_y,
        legend_width,
        legend_height,
        8.0,
        bg_color,
    );
    
    // Title
    let title_color = match self.theme {
        RenderTheme::Dark => Color::from_rgba8(220, 230, 240, 255),
        RenderTheme::Light => Color::from_rgba8(20, 30, 40, 255),
    };
    
    font_renderer.render_text(
        &mut self.pixmap,
        "LEGEND",
        legend_x + 20.0,
        legend_y + 30.0,
        16.0,
        title_color,
    );
    
    // Spectral types
    let mut y_offset = legend_y + 60.0;
    let spectral_types = [
        (SpectralType::O, "O - Blue"),
        (SpectralType::B, "B - Blue-White"),
        (SpectralType::A, "A - White"),
        (SpectralType::F, "F - Yellow-White"),
        (SpectralType::G, "G - Yellow"),
        (SpectralType::K, "K - Orange"),
        (SpectralType::M, "M - Red"),
    ];
    
    for (spec_type, label) in spectral_types {
        let (color, _) = self.get_spectral_color(spec_type);
        
        // Draw small example star
        self.draw_filled_circle(
            legend_x + 30.0,
            y_offset,
            6.0,
            color,
        );
        
        // Draw label
        font_renderer.render_text(
            &mut self.pixmap,
            label,
            legend_x + 50.0,
            y_offset + 5.0,
            12.0,
            title_color,
        );
        
        y_offset += 25.0;
    }
    
    // Distance line styles
    y_offset += 20.0;
    font_renderer.render_text(
        &mut self.pixmap,
        "DISTANCE",
        legend_x + 20.0,
        y_offset,
        14.0,
        title_color,
    );
    
    y_offset += 30.0;
    
    let line_examples = [
        (2.5, "0-5 LY: Close"),
        (7.5, "5-10 LY: Moderate"),
        (12.5, "10-15 LY: Far"),
        (17.5, "15-20 LY: Very Far"),
    ];
    
    for (distance, label) in line_examples {
        let style = self.get_line_style(distance);
        
        // Draw example line
        self.draw_line_with_style(
            legend_x + 25.0,
            y_offset,
            legend_x + 25.0 + 50.0,
            y_offset,
            style,
        );
        
        // Draw label
        font_renderer.render_text(
            &mut self.pixmap,
            label,
            legend_x + 90.0,
            y_offset + 5.0,
            11.0,
            title_color,
        );
        
        y_offset += 25.0;
    }
}
```

## Complete Rendering Example

```rust
use tiny_skia::*;
use rand::{SeedableRng, Rng};
use rand_chacha::ChaCha8Rng;

pub struct StarMap {
    pub stars: Vec<Star>,
    pub connections: Vec<Connection>,
}

impl StarMap {
    pub fn render_to_files(
        &self,
        output_dir: &str,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Load font (embed or read from file)
        let font_data = include_bytes!("../assets/fonts/Roboto-Regular.ttf");
        let font_renderer = FontRenderer::new(font_data);
        
        // Render dark version (screen)
        let mut renderer_dark = StarMapRenderer::new(width, height, RenderTheme::Dark);
        renderer_dark.render(&self.stars, &self.connections, &font_renderer);
        renderer_dark.save_png(&format!("{}/starmap_dark.png", output_dir))?;
        
        // Render light version (print)
        let mut renderer_light = StarMapRenderer::new(width, height, RenderTheme::Light);
        renderer_light.render(&self.stars, &self.connections, &font_renderer);
        renderer_light.save_png(&format!("{}/starmap_light.png", output_dir))?;
        
        Ok(())
    }
}

impl StarMapRenderer {
    pub fn new(width: u32, height: u32, theme: RenderTheme) -> Self {
        let pixmap = Pixmap::new(width, height).unwrap();
        
        Self {
            width,
            height,
            pixmap,
            theme,
        }
    }
    
    pub fn save_png(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.pixmap.save_png(path)?;
        Ok(())
    }
}
```

## Usage Example

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load star data from Astrosynthesis SQLite
    let stars = load_stars_from_astrodb("sector.AstroDB")?;
    
    // Project 3D to 2D (using algorithm from StarMap2D_Visualization.md)
    let stars_2d = project_to_2d(&stars)?;
    
    // Calculate connections (within 15 LY)
    let connections = calculate_connections(&stars_2d, 15.0);
    
    // Create map
    let map = StarMap {
        stars: stars_2d,
        connections,
    };
    
    // Render both versions at 5000x5000
    map.render_to_files("./output", 5000, 5000)?;
    
    println!("Star maps rendered successfully!");
    println!("  - output/starmap_dark.png (screen viewing)");
    println!("  - output/starmap_light.png (printing)");
    
    Ok(())
}
```

## Optimization Tips

### Performance

1. **Spatial Partitioning**: Use KD-tree or grid to avoid rendering off-screen elements
2. **Level of Detail**: Simplify star rendering when many stars visible
3. **Parallel Rendering**: Use Rayon to render stars in parallel
4. **Caching**: Cache rendered star symbols at common sizes

### Quality

1. **Anti-aliasing**: Always enable for smooth curves and diagonals
2. **Gamma Correction**: Apply proper gamma for accurate colors
3. **Subpixel Positioning**: Use float coordinates, not integers
4. **Blending**: Use proper alpha blending for layered effects

### File Size

1. **PNG Compression**: Use appropriate compression level (6-9)
2. **Indexed Color**: Consider for print version (fewer colors needed)
3. **Strip Metadata**: Remove unnecessary metadata from PNG

## Advanced Features

### Interactive Elements (Future)

If rendering to SVG instead of PNG:
- Add `id` attributes to stars for JavaScript interaction
- Include metadata in custom attributes
- Enable CSS-based styling
- Support zoom/pan without re-rendering

### Animations

For animated versions:
- Render star "twinkle" (vary brightness slightly)
- Animate connections pulsing
- Rotate corona rays slowly
- Depth-based parallax on pan

### Multi-Resolution

Generate multiple resolutions:
- 5000x5000 - Print quality
- 2000x2000 - Screen/web
- 500x500 - Thumbnail

## Testing and Validation

### Visual Quality Checks

1. **Zoom Test**: View at 100%, 50%, 25% - should look good at all scales
2. **Print Test**: Actually print on paper to verify colors/contrast
3. **Color Blindness**: Test with color blindness simulators
4. **Edge Cases**: Test with very dense regions, single star, etc.

### Performance Benchmarks

Target performance for 5000x5000 output:
- 100 stars: < 1 second
- 500 stars: < 5 seconds
- 1000 stars: < 15 seconds

Profile with `cargo flamegraph` to identify bottlenecks.

## Conclusion

This specification provides a complete implementation guide for rendering beautiful, professional-quality 2D star maps in Rust. The procedural approach ensures infinite visual variety while maintaining cartographic clarity. The dual-theme system optimizes for both screen viewing and print output.

Key principles:
- **Procedural over static** for flexibility
- **Clear visual hierarchy** for usability
- **Anti-aliasing everywhere** for quality
- **Theme-aware rendering** for versatility

The result: Star maps that are both scientifically accurate (real distances annotated) and aesthetically beautiful (space-opera visual appeal).
