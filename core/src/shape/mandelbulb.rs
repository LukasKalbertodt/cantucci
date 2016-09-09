use math::*;

#[derive(Clone)]
pub struct Mandelbulb {
    power: f64,
    max_iters: u64,
}

impl Mandelbulb {
    pub fn new(power: f64, max_iters: u64) -> Self {
        Mandelbulb {
            power: power,
            max_iters: max_iters,
        }
    }

    pub fn classic(max_iters: u64) -> Self {
        Self::new(8.0, max_iters)
    }
}

impl super::Shape for Mandelbulb {
    fn contains(&self, p: Point3<f64>) -> bool {
        const BAILOUT: f64 = 2.5;

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
            let phi = f64::atan2(z.y, z.x);

            // Scale and rotate the point
            let zr = r.powf(self.power);
            let theta = theta * self.power;
            let phi = phi * self.power;

            // Convert back to cartesian coordinates and add p
            z = zr * Point3::new(
                theta.sin() * phi.cos(),
                phi.sin() * theta.sin(),
                theta.cos()
            );
            z += p.to_vec();
        }

        // The point didn't diverge within `max_iters`, so we assume it's in
        // the set
        true
    }
}
