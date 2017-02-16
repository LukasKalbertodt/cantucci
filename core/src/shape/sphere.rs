use math::*;
use super::{DistanceApprox, Shape};

#[derive(Clone)]
pub struct Sphere {
    center: Point3<f32>,
    radius: f32,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32) -> Self {
        Sphere {
            center: center,
            radius: radius,
        }
    }
}

impl Shape for Sphere {
    // Overwrite default method for performance
    fn contains(&self, p: Point3<f32>) -> bool {
        (self.center - p).magnitude2() <= (self.radius * self.radius)
    }

    fn distance(&self, p: Point3<f32>) -> DistanceApprox {
        let d = (self.center - p).magnitude() - self.radius;
        DistanceApprox {
            min: d,
            max: d,
        }
    }
}
