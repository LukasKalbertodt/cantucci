use math::*;

#[macro_use]
mod util;
mod mandelbulb;
mod sphere;

pub use self::mandelbulb::Mandelbulb;
pub use self::sphere::Sphere;

/// Describes a 3D object that can be rendered by this application.
///
/// Unlike in standard real time 3D graphics, the object is not represented
/// with a triangle mesh, but via a small set of functions (actually, only one
/// function is important). It's usually called distance field or distance
/// function as the only way to get information about our object is by querying
/// the so called distance estimator (DE). This function returns an
/// approximation of the distance from a given point to the surface of the
/// mesh. See `min_distance_from()` for more information.
pub trait Shape: Send + 'static {
    /// Returns a lower bound of the distance from `p` to the closest surface
    /// point of the shape.
    ///
    /// If `p` is inside the shape, the returned distance has to be negative.
    /// The value must converge towards the real distance when approaching the
    /// shape. That means that there has to be a constant c such that for every
    /// point p the real distance is <= c * min_distance_from(p):
    ///
    /// ∃ c ∈ ℝ  ∀ p ∈ ℝ³ distance_from(p) <= c * min_distance_from(p)
    ///
    /// This also implies that min_distance_from(p) can only return 0 iff p
    /// lies on the shape's surface.
    fn min_distance_from(&self, p: Point3<f32>) -> f32;

    // TODO: this method is hacky...
    /// Returns a string containing the GLSL definition of the distance
    /// estimator.
    ///
    /// The GLSL function needs to have this signature:
    ///
    /// ```
    /// float shape_de(vec3 point)
    /// ```
    fn de_shader(&self) -> String;

    /// Returns an upper bound of the distance from `p` to the closest surface
    /// point of the shape, or `None` if no such estimate can be made.
    ///
    /// Whether or not this function returns `None` might only depend on the
    /// implementer (the `self` parameter) and *not* on `p`! So if this
    /// function returns `None` once, the calling code can assume that this
    /// shape can never return an upper bound.
    ///
    /// Similar to `min_distance_from()` this upper bound must converge to the
    /// real value as we approach the surface.
    fn max_distance_from(&self, _p: Point3<f32>) -> Option<f32> {
        None
    }

    /// Combines `min_distance_from()` and `max_distance_from()`: returns a
    /// tuple of the lower and upper bound (in that order).
    ///
    /// This method is here for optimization purposes only. If you are
    /// interested in the lower *and* upper bound, you should call this method
    /// as shapes may implement it more efficiently than calling
    /// `min_distance_from()` and `max_distance_from()` independently.
    fn bounded_distance_from(&self, p: Point3<f32>) -> (f32, Option<f32>) {
        (self.min_distance_from(p), self.max_distance_from(p))
    }

    /// Returns true iff the given point lies in the shape.
    fn contains(&self, p: Point3<f32>) -> bool {
        self.min_distance_from(p) < 0.0
    }

    /// Calls `min_distance_from()` for each given point and returns the
    /// results as vector. This is for use through a trait object to reduce
    /// the virtual call overhead.
    ///
    /// This method and the other two `batch_` methods should be implemented
    /// by using the `impl_batch_methods` macro, if it's not possible to
    /// improve performance by writing a custom implementation.
    fn batch_min_distance_from(&self, points: &[Point3<f32>]) -> Vec<f32>;

    /// Calls `max_distance_from()` for each given point and returns the
    /// results as vector. See `batch_min_distance_from()` for more
    /// information.
    ///
    /// This will panic when `max_distance_from()` returns `None`.
    fn batch_max_distance_from(&self, points: &[Point3<f32>]) -> Vec<f32>;

    /// Calls `bounded_distance_from()` for each given point and returns the
    /// results as vector. See `batch_min_distance_from()` for more
    /// information.
    ///
    /// This will panic when `max_distance_from()` returns `None`.
    fn batch_bounded_distance_from(&self, points: &[Point3<f32>]) -> Vec<(f32, f32)>;

}
