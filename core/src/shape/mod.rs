use math::*;

mod sphere;
mod mandelbulb;

pub use self::sphere::*;
pub use self::mandelbulb::*;

pub struct DistanceApprox {
    pub min: f64,
    pub max: f64,
}

pub trait Shape: Send {
    fn contains(&self, p: Point3<f64>) -> bool;
    fn distance(&self, p: Point3<f64>) -> DistanceApprox;
}
