use math::*;
use super::{DistanceApprox, Shape};

#[derive(Clone)]
pub struct Sphere {
    center: Point3<f64>,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Point3<f64>, radius: f64) -> Self {
        Sphere {
            center: center,
            radius: radius,
        }
    }
}

impl Shape for Sphere {
    fn contains(&self, p: Point3<f64>) -> bool {
        (self.center - p).magnitude2() <= (self.radius * self.radius)
    }

    fn distance(&self, p: Point3<f64>) -> DistanceApprox {
        let d = (self.center - p).magnitude() - self.radius;
        DistanceApprox {
            min: d,
            max: d,
        }
    }
}
