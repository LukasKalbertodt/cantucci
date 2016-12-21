    use math::*;

mod sphere;
mod mandelbulb;

pub use self::sphere::Sphere;
pub use self::mandelbulb::Mandelbulb;

#[derive(Clone, Copy, Debug)]
pub struct DistanceApprox {
    pub min: f64,
    pub max: f64,
}

pub trait Shape: Send {
    /// Returns an estimate for the distance from `p` to the closest surface
    /// point of the shape.
    ///
    /// If `p` is inside the shape, the returned distance has to be negative.
    /// Either both or none of `min` and `max` have to be negative.
    fn distance(&self, p: Point3<f64>) -> DistanceApprox;

    /// Returns true iff the given point lies in the shape.
    fn contains(&self, p: Point3<f64>) -> bool;
}
