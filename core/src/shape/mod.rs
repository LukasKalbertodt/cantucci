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

    /// Returns the estimated distance from `p` to the closest surface point.
    /// If `p` is inside the shape, the same applies, but the returned distance
    /// has to be negative. `0.0` may be returned, this function mustn't
    /// return `-0.0`.
    fn distance(&self, p: Point3<f64>) -> DistanceApprox;
}
