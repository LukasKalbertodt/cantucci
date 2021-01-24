use std::ops::Range;
use cgmath::Point3;

#[macro_use]
mod util;
mod mandelbulb;
mod sphere;

#[cfg(test)]
mod bench;

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
pub trait Shape: Sync + Send + 'static {
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

    fn bounding_box(&self) -> Range<Point3<f32>>;

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


// Some points close to the surface of the mandelbulb which are used for
// benchmarking.
#[cfg(test)]
const BENCH_POINTS: [[f32; 3]; 20] = [
    [-0.73772496, -0.002343091, -0.7382717],
    [-0.7484558, -0.8255949, -0.0026540023],
    [-1.0951594, -0.0014639703, -0.0027306266],
    [-0.60622436, -0.16786861, 0.7227598],
    [-0.6000897, -0.5997089, 0.028461732],
    [-0.6077231, -0.8336551, -0.004541016],
    [-0.05153041, -0.5906257, -0.7647207],
    [-0.73772484, -0.0030531297, -0.7382715],
    [-1.09658, -0.032518614, 0.026089936],
    [-0.74845594, -0.8255949, -0.0033077204],
    [-0.0031473506, 0.59545904, 0.7711717],
    [0.59178185, -0.009300065, 0.70574695],
    [0.5934337, -0.0065053166, -0.8548532],
    [0.5906368, 0.5906708, 0.0002929632],
    [0.5909915, 0.6001409, -0.4285654],
    [-0.004541016, 0.5956404, 0.36293367],
    [-0.00073693885, 0.5916996, -0.8447121],
    [0.59545904, -0.004541016, 0.35817686],
    [0.59545904, -0.004541016, -0.3581769],
    [0.60028464, -0.36826742, 0.6579103],

];
