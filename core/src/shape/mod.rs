use math::*;

mod sphere;
mod mandelbulb;

pub use self::sphere::*;
pub use self::mandelbulb::*;

pub trait Shape: Send + Clone {
    fn contains(&self, p: Point3<f64>) -> bool;
}
