use nalgebra::Vector2;

use crate::geom::*;

// An implicit surface
pub trait IsoLine {
    // A signed distance function (SDF) sample of this surface.
    // If sample returns a positive value (>0), this point is inside the surface.
    fn sample(&self, point: Vector2<f32>) -> f32;
    fn normal(&self, point: Vector2<f32>) -> Vector2<f32> {
        let epsilon = 0.01;
        Vector2::new(
            self.sample(point + (Vector2::x() * epsilon))
                - self.sample(point - Vector2::x() * epsilon),
            self.sample(point + (Vector2::y() * epsilon))
                - self.sample(point - Vector2::y() * epsilon),
        )
        .normalize()
    }
}

impl IsoLine for Circle {
    fn sample(&self, point: Vector2<f32>) -> f32 {
        let delta = point - self.center;
        (self.radius * self.radius) - delta.dot(&delta)
    }

    fn normal(&self, point: Vector2<f32>) -> Vector2<f32> {
        (point - self.center).normalize()
    }
}
