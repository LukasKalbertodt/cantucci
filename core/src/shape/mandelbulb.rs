use math::*;
use super::Shape;

// TODO: docs
#[derive(Clone)]
pub struct Mandelbulb {
    power: f32,
    max_iters: u64,
    bailout: f32,
}

const CENTER: Point3<f32> = Point3 { x: 0.0, y: 0.0, z: 0.0 };

impl Mandelbulb {
    pub fn new(power: f32, max_iters: u64, bailout: f32) -> Self {
        Mandelbulb {
            power: power,
            max_iters: max_iters,
            bailout: bailout,
        }
    }

    pub fn classic(max_iters: u64, bailout: f32) -> Self {
        Self::new(8.0, max_iters, bailout)
    }
}

impl Shape for Mandelbulb {
    fn contains(&self, p: Point3<f32>) -> bool {
        let mut z = p;

        for _ in 0..self.max_iters {
            // If the radius is bigger than BAILOUT, this point will diverge
            if (z - CENTER).magnitude() > self.bailout {
                return false;
            }

            z = rotate(z, self.power) + (p - CENTER);
        }

        // The point didn't diverge within `max_iters`, so we assume it's in
        // the set
        true
    }

    fn min_distance_from(&self, p: Point3<f32>) -> f32 {
        let mut z = p;
        let mut dr = 1.0;
        let mut r = 0.0;

        for _ in 0..self.max_iters {
            r = (z - CENTER).magnitude();
            if r > self.bailout || (1.0 / r).is_infinite() {
                break;
            }

            dr = r.powf(self.power - 1.0) * self.power * dr + 1.0;
            z = rotate(z, self.power) + (p - CENTER);
        }

        let ln_r = if r.ln().is_infinite() { 0.0 } else { r.ln() * r };
        0.5 * ln_r / dr
    }

    fn de_shader(&self) -> String {
        let s = include_str!("mandelbulb.frag")
            .replace("{BAILOUT}", &self.bailout.to_string())
            .replace("{MAX_ITERS}", &self.max_iters.to_string())
            .replace("{POWER}", &self.power.to_string());

        s
    }
}

fn rotate(z: Point3<f32>, power: f32) -> Point3<f32> {
    let old_radius = (z - CENTER).magnitude();

    // Convert to spherical coordinates
    let theta = (z.z / old_radius).acos();
    let phi = f32::atan2(z.y, z.x);

    // Scale and rotate the point
    let new_radius = old_radius.powf(power);
    let theta = theta * power;
    let phi = phi * power;

    // Convert back to cartesian coordinates and add p
    new_radius * Point3::new(
        theta.sin() * phi.cos(),
        phi.sin() * theta.sin(),
        theta.cos(),
    )
}
