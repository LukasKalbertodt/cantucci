use math::*;
use super::{DistanceApprox, Shape};

#[derive(Clone)]
pub struct Mandelbulb {
    power: f32,
    max_iters: u64,
}

impl Mandelbulb {
    pub fn new(power: f32, max_iters: u64) -> Self {
        Mandelbulb {
            power: power,
            max_iters: max_iters,
        }
    }

    pub fn classic(max_iters: u64) -> Self {
        Self::new(8.0, max_iters)
    }
}

impl Shape for Mandelbulb {
    fn contains(&self, p: Point3<f32>) -> bool {
        const BAILOUT: f32 = 2.5;

        let mut z = p;

        for _ in 0..self.max_iters {
            // Short alias for the current radius
            let r = z.to_vec().magnitude();

            // If the radius is bigger than BAILOUT, this point will diverge
            if r > BAILOUT {
                return false;
            }

            // Convert to spherical coordinates
            let theta = (z.z / r).acos();
            let phi = f32::atan2(z.y, z.x);

            // Scale and rotate the point
            let zr = r.powf(self.power);
            let theta = theta * self.power;
            let phi = phi * self.power;

            // Convert back to cartesian coordinates and add p
            z = zr * Point3::new(
                theta.sin() * phi.cos(),
                phi.sin() * theta.sin(),
                theta.cos(),
            );
            z += p.to_vec();
        }

        // The point didn't diverge within `max_iters`, so we assume it's in
        // the set
        true
    }

    fn distance(&self, p: Point3<f32>) -> DistanceApprox {
        let mut z = p;
        let mut dr = 1.0;
        let mut r = 0.0;

        const BAILOUT: f32 = 2.5;

        for _ in 0..self.max_iters {
            r = z.to_vec().magnitude();
            if r > BAILOUT || (1.0 / r).is_infinite() {
                break;
            }

            // convert to polar coordinates
            let theta = (z.z / r).acos();
            let phi = f32::atan2(z.y, z.x);
            dr = r.powf(self.power - 1.0) * self.power * dr + 1.0;

            // scale and rotate the point
            let zr = r.powf(self.power);
            let theta = theta * self.power;
            let phi = phi * self.power;

            // convert back to cartesian coordinates
            z = zr * Point3::new(
                theta.sin() * phi.cos(),
                phi.sin() * theta.sin(),
                theta.cos(),
            );
            z += p.to_vec();
        }

        let ln_r = if r.ln().is_infinite() { 0.0 } else { r.ln() * r };
        let lower = 0.5 * ln_r / dr;

        DistanceApprox {
            min: lower,
            max: 4.0 * lower,
        }
    }
}
