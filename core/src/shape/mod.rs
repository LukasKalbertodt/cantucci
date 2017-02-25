use math::*;

mod sphere;
mod mandelbulb;

pub use self::sphere::Sphere;
pub use self::mandelbulb::Mandelbulb;

#[derive(Clone, Copy, Debug)]
pub struct DistanceApprox {
    pub min: f32,
    pub max: f32,
}

pub trait Shape: Send + 'static {
    /// Returns an estimate for the distance from `p` to the closest surface
    /// point of the shape.
    ///
    /// If `p` is inside the shape, the returned distance has to be negative.
    /// Either both or none of `min` and `max` have to be negative.
    fn distance(&self, p: Point3<f32>) -> DistanceApprox;

    /// Returns true iff the given point lies in the shape.
    fn contains(&self, p: Point3<f32>) -> bool;

    /// Returns a string containing the GLSL definition of the distance
    /// estimator.
    ///
    /// The GLSL function needs to have this signature:
    ///
    /// ```
    /// float shape_de(vec3 point)
    /// ```
    fn de_shader(&self) -> String;
}
