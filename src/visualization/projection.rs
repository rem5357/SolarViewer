/// Projection engine for converting 3D stellar coordinates to 2D
/// Uses orthographic projection with overlap resolution

#[derive(Debug, Clone)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct ProjectionEngine {
    width: u32,
    height: u32,
    margin: u32,
}

impl ProjectionEngine {
    pub fn new(width: u32, height: u32, margin: u32) -> Self {
        Self { width, height, margin }
    }

    /// Project 3D points to 2D using orthographic projection (drop Z)
    /// Scales to fit within the output dimensions
    pub fn project_orthographic(&self, points_3d: &[Point3D]) -> Vec<Point2D> {
        if points_3d.is_empty() {
            return Vec::new();
        }

        // Find bounding box in 3D space (using only X, Y)
        let mut min_x = points_3d[0].x;
        let mut max_x = points_3d[0].x;
        let mut min_y = points_3d[0].y;
        let mut max_y = points_3d[0].y;

        for point in points_3d {
            if point.x < min_x { min_x = point.x; }
            if point.x > max_x { max_x = point.x; }
            if point.y < min_y { min_y = point.y; }
            if point.y > max_y { max_y = point.y; }
        }

        let mut range_x = max_x - min_x;
        let mut range_y = max_y - min_y;

        // Add 10% padding
        range_x *= 1.1;
        range_y *= 1.1;

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;

        // Calculate scale to fit in available space
        let available_width = (self.width - 2 * self.margin) as f64;
        let available_height = (self.height - 2 * self.margin) as f64;

        let scale = if range_x > 0.0 && range_y > 0.0 {
            (available_width / range_x).min(available_height / range_y)
        } else {
            1.0
        };

        // Project each point
        points_3d
            .iter()
            .map(|p| {
                let px = (p.x - center_x) * scale + (self.width as f64) / 2.0;
                let py = (p.y - center_y) * scale + (self.height as f64) / 2.0;
                Point2D { x: px, y: py }
            })
            .collect()
    }

    /// Resolve overlapping stars by applying repulsive forces
    pub fn resolve_overlaps(&self, points_2d: &mut [Point2D], min_distance: f64) {
        let max_iterations = 50;

        for _ in 0..max_iterations {
            let mut moved = false;

            for i in 0..points_2d.len() {
                for j in (i + 1)..points_2d.len() {
                    let dx = points_2d[j].x - points_2d[i].x;
                    let dy = points_2d[j].y - points_2d[i].y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < min_distance && dist > 0.0 {
                        // Push stars apart
                        let force = (min_distance - dist) / 2.0;
                        let norm_x = dx / dist;
                        let norm_y = dy / dist;

                        points_2d[i].x -= norm_x * force;
                        points_2d[i].y -= norm_y * force;
                        points_2d[j].x += norm_x * force;
                        points_2d[j].y += norm_y * force;

                        moved = true;
                    }
                }
            }

            if !moved {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection() {
        let engine = ProjectionEngine::new(1000, 1000, 100);
        let points = vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 10.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 10.0, z: 5.0 },
        ];

        let projected = engine.project_orthographic(&points);
        assert_eq!(projected.len(), 3);
        assert!(projected[0].x > 0.0);
        assert!(projected[0].y > 0.0);
    }
}
