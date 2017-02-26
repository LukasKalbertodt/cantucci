use math::*;
use super::Shape;

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

    fn min_distance_from(&self, p: Point3<f32>) -> f32 {
        (self.center - p).magnitude() - self.radius
    }

    fn max_distance_from(&self, p: Point3<f32>) -> Option<f32> {
        Some(self.min_distance_from(p))
    }

    fn de_shader(&self) -> String {
        let s = include_str!("shape.frag")
            .replace("{X}", &self.center.x.to_string())
            .replace("{Y}", &self.center.y.to_string())
            .replace("{Z}", &self.center.z.to_string())
            .replace("{RADIUS}", &self.radius.to_string());

        s
    }
}
