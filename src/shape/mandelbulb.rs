use std::ops::Range;
use cgmath::{prelude::*, Point3};

use super::Shape;

/// Represents the 3D version of the classical mandelbulb described [here][1].
///
/// [1]: http://www.skytopia.com/project/fractal/mandelbulb.html
#[derive(Clone)]
pub struct Mandelbulb<const P: u8> {
    max_iters: u64,
    bailout: f32,
}

const CENTER: Point3<f32> = Point3 { x: 0.0, y: 0.0, z: 0.0 };

impl<const P: u8> Mandelbulb<P> {
    pub fn new(max_iters: u64, bailout: f32) -> Self {
        assert!(max_iters >= 1);

        Mandelbulb {
            max_iters,
            bailout,
        }
    }
}

impl Mandelbulb<8> {
    pub fn classic(max_iters: u64, bailout: f32) -> Self {
        Self::new(max_iters, bailout)
    }
}

impl<const P: u8> Shape for Mandelbulb<P> {
    fn contains(&self, p: Point3<f32>) -> bool {
        let mut z = p;

        for _ in 0..self.max_iters {
            // If the radius is bigger than BAILOUT, this point will diverge
            if (z - CENTER).magnitude() > self.bailout {
                return false;
            }

            z = rotate::<P>(z) + (p - CENTER);
        }

        // The point didn't diverge within `max_iters`, so we assume it's in
        // the set
        true
    }

    fn bounding_box(&self) -> Range<Point3<f32>> {
        // TODO: This value was found by experimenting... we should prove this
        // value
        Point3::new(-1.2, -1.2, -1.2) .. Point3::new(1.2, 1.2, 1.2)
    }

    fn min_distance_from(&self, p: Point3<f32>) -> f32 {
        let mut z = p;
        let mut dr = 1.0;
        let mut r = 0.0;

        for _ in 0..self.max_iters {
            r = (z - CENTER).magnitude();
            if r > self.bailout {
                break;
            }

            dr = r.powi(P as i32 - 1) * (P as f32) * dr + 1.0;
            z = rotate::<P>(z) + (p - CENTER);
        }

        let ln_r = r.ln() * r;
        0.5 * ln_r / dr
    }

    fn de_shader(&self) -> String {
        let s = include_str!("mandelbulb.frag")
            .replace("{BAILOUT}", &self.bailout.to_string())
            .replace("{MAX_ITERS}", &self.max_iters.to_string())
            .replace("{POWER}", &P.to_string());

        s
    }

    impl_batch_methods!();
}

/// This operation rotates the point as triplex number. This is equivalent to
/// the squaring in the original 2D mandelbrot. First we convert the point
/// to spherical coordinates, then we rotate and convert them back.
#[inline(always)]
fn rotate<const P: u8>(p: Point3<f32>) -> Point3<f32> {
    // Handle special case (general formula is not able to handle points on
    // the z axis).
    if p.x == 0.0 && p.y == 0.0 {
        let old_radius = (p - CENTER).magnitude();
        let theta = (p.z / old_radius).acos();

        // Scale and rotate the point
        let new_radius = old_radius.powi(P.into());
        let theta = theta * P as f32;

        // Convert back to cartesian coordinates
        return new_radius * Point3::new(0.0, 0.0, theta.cos());
    }


    // For some integer powers there are formulas without trigonometric
    // functions. This improves performance a lot (see #17).
    match P {
        8 => {
            let Point3 { x, y, z } = p;

            let x2 = x.powi(2);
            let x4 = x.powi(4);
            let x6 = x.powi(6);
            let x8 = x.powi(8);

            let y2 = y.powi(2);
            let y4 = y.powi(4);
            let y6 = y.powi(6);
            let y8 = y.powi(8);

            let z2 = z.powi(2);
            let z4 = z.powi(4);
            let z6 = z.powi(6);
            let z8 = z.powi(8);

            let rxy2 = x2 + y2;
            let rxy4 = rxy2.powi(2);
            let rxy6 = rxy2.powi(3);
            let rxy8 = rxy2.powi(4);

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
        power => {
            let old_radius = (p - CENTER).magnitude();

            // Convert to spherical coordinates
            let theta = (p.z / old_radius).acos();
            let phi = f32::atan2(p.y, p.x);

            // Scale and rotate the point
            let new_radius = old_radius.powi(power.into());
            let theta = theta * power as f32;
            let phi = phi * power as f32;

            // Convert back to cartesian coordinates
            new_radius * Point3::new(
                theta.sin() * phi.cos(),
                phi.sin() * theta.sin(),
                theta.cos(),
            )
        }
    }
}
