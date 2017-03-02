use math::*;
use super::Shape;

/// Represents the 3D version of the classical mandelbulb described [here][1].
///
/// [1]: http://www.skytopia.com/project/fractal/mandelbulb.html
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

    impl_batch_methods!();
}

/// This operation rotates the point as triplex number. This is equivalent to
/// the squaring in the original 2D mandelbrot. First we convert the point
/// to spherical coordinates, then we rotate and convert them back.
fn rotate(p: Point3<f32>, power: f32) -> Point3<f32> {
    // Handle special case (general formula is not able to handle points on
    // the z axis).
    if p.x == 0.0 && p.y == 0.0 {
        let old_radius = (p - CENTER).magnitude();
        let theta = (p.z / old_radius).acos();

        // Scale and rotate the point
        let new_radius = old_radius.powf(power);
        let theta = theta * power;

        // Convert back to cartesian coordinates
        return new_radius * Point3::new(0.0, 0.0, theta.cos());
    }


    // For some integer powers there are formulas without trigonometric
    // functions. This improves performance.
    match power {
        8.0 => {
            let Point3 { x, y, z } = p;

            // Yes we actually need to do that, LLVM won't generate optimal
            // code here. LLVM transforms `x.powf(2)` into `x * x` but that's
            // all. It has probably to do with floating point precision, but
            // it's not that important for us.
            let x2 = x * x;
            let x4 = x2 * x2;
            let x6 = x2 * x4;
            let x8 = x4 * x4;

            let y2 = y * y;
            let y4 = y2 * y2;
            let y6 = y2 * y4;
            let y8 = y4 * y4;

            let z2 = z * z;
            let z4 = z2 * z2;
            let z6 = z2 * z4;
            let z8 = z4 * z4;

            let rxy2 = x2 + y2;
            let rxy4 = rxy2 * rxy2;
            let rxy6 = rxy2 * rxy4;
            let rxy8 = rxy4 * rxy4;

            let a = 1.0 + (
                z8
                - 28.0 * z6 * rxy2
                + 70.0 * z4 * rxy4
                - 28.0 * z2 * rxy6
            ) / rxy8;


            Point3 {
                x: a * (
                    x8
                    - 28.0 * x6 * y2
                    + 70.0 * x4 * y4
                    - 28.0 * x2 * y6
                    - y8
                ),
                y: 8.0 * a * x * y * (
                    x6
                    - 7.0 * x4 * y2
                    + 7.0 * x2 * y4
                    - y6
                ),
                z: 8.0 * z
                    * rxy2.sqrt()
                    * (z2 - rxy2)
                    * (z4 - 6.0 * z2 * rxy2 + rxy4),
            }
        }
        _ => {
            let old_radius = (p - CENTER).magnitude();

            // Convert to spherical coordinates
            let theta = (p.z / old_radius).acos();
            let phi = f32::atan2(p.y, p.x);

            // Scale and rotate the point
            let new_radius = old_radius.powf(power);
            let theta = theta * power;
            let phi = phi * power;

            // Convert back to cartesian coordinates
            new_radius * Point3::new(
                theta.sin() * phi.cos(),
                phi.sin() * theta.sin(),
                theta.cos(),
            )
        }
    }
}
