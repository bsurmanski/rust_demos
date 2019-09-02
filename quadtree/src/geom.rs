use nalgebra::Vector2;

#[derive(Debug)]
pub struct Circle {
    pub center: Vector2<f32>,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: Vector2<f32>, radius: f32) -> Circle {
        Circle { center, radius }
    }
}

pub struct Line {
    pub points: [Vector2<f32>; 2],
}

impl Line {
    pub fn new(p1: Vector2<f32>, p2: Vector2<f32>) -> Line {
        Line { points: [p1, p2] }
    }

    pub fn lengthsq(&self) -> f32 {
        return self.points[0].dot(&self.points[1])
    }

    pub fn length(&self) -> f32 {
        return f32::sqrt(self.lengthsq())
    }
}

