#![cfg(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1"
    )
)]

use std::{arch::x86_64::{
        __m128,
        _mm_set_ps,
        _mm_dp_ps,
        _mm_cvtss_f32,
        _mm_sqrt_ss,
        _mm_sub_ps,
        _mm_test_all_zeros,
        _mm_set_epi32,
        _mm_castps_si128,
        _mm_extract_ps,
        _mm_mul_ps,
        _mm_add_ps,
    }, ops::{Add, Mul, Range, Sub}};
use cgmath::Point3;

use super::Shape;

/// Represents the 3D version of the classical mandelbulb described [here][1].
///
/// [1]: http://www.skytopia.com/project/fractal/mandelbulb.html
#[derive(Clone)]
pub struct Mandelbulb<const P: u8> {
    max_iters: u64,
    bailout: f32,
}

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
    // fn contains(&self, p: Point3<f32>) -> bool {
    //     let mut z = p;

    //     for _ in 0..self.max_iters {
    //         // If the radius is bigger than BAILOUT, this point will diverge
    //         if (z - CENTER).magnitude() > self.bailout {
    //             return false;
    //         }

    //         z = rotate::<P>(z) + (p - CENTER);
    //     }

    //     // The point didn't diverge within `max_iters`, so we assume it's in
    //     // the set
    //     true
    // }

    fn bounding_box(&self) -> Range<Point3<f32>> {
        // TODO: This value was found by experimenting... we should prove this
        // value
        Point3::new(-1.2, -1.2, -1.2) .. Point3::new(1.2, 1.2, 1.2)
    }

    fn min_distance_from(&self, p: Point3<f32>) -> f32 {
        let p = Vec3::new(p.x, p.y, p.z);
        let mut z = p;
        let mut dr = 1.0;
        let mut r = 0.0;

        for _ in 0..self.max_iters {
            r = z.magnitude();
            if r > self.bailout {
                break;
            }

            dr = r.powi(P as i32 - 1) * (P as f32) * dr + 1.0;
            z = rotate::<P>(z) + p;
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
fn rotate<const P: u8>(p: Vec3) -> Vec3 {
    // Handle special case (general formula is not able to handle points on
    // the z axis).
    if p.is_on_z_axis() {
        let old_radius = p.magnitude();
        let theta = (p.z() / old_radius).acos();

        // Scale and rotate the point
        let new_radius = old_radius.powi(P.into());
        let theta = theta * P as f32;

        // Convert back to cartesian coordinates
        return Vec3::new(0.0, 0.0, new_radius * theta.cos());
    }


    // For some integer powers there are formulas without trigonometric
    // functions. This improves performance a lot (see #17).
    match P {
        8 => {
            let x = p.x();
            let y = p.y();
            let z = p.z();

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


            Vec3::new(
                a * (
                    x8
                    - 28.0 * x6 * y2
                    + 70.0 * x4 * y4
                    - 28.0 * x2 * y6
                    - y8
                ),
                8.0 * a * x * y * (
                    x6
                    - 7.0 * x4 * y2
                    + 7.0 * x2 * y4
                    - y6
                ),
                8.0 * z
                    * rxy2.sqrt()
                    * (z2 - rxy2)
                    * (z4 - 6.0 * z2 * rxy2 + rxy4),
            )
        }
        power => {
            let old_radius = p.magnitude();

            // Convert to spherical coordinates
            let theta = (p.z() / old_radius).acos();
            let phi = f32::atan2(p.y(), p.x());

            // Scale and rotate the point
            let new_radius = old_radius.powi(power.into());
            let theta = theta * power as f32;
            let phi = phi * power as f32;

            // Convert back to cartesian coordinates
            new_radius * Vec3::new(
                theta.sin() * phi.cos(),
                phi.sin() * theta.sin(),
                theta.cos(),
            )
        }
    }
}


/// A 3D vector stored in a 128bit SIMD register.
///
/// x is stored in the lowest bits, the highest 32 bits are unused.
#[derive(Debug, Clone, Copy)]
struct Vec3(__m128);

impl Vec3 {
    #[inline(always)]
    fn new(x: f32, y: f32, z: f32) -> Self {
        let r = unsafe { _mm_set_ps(0.0, z, y, x) };
        Self(r)
    }

    #[inline(always)]
    fn x(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.0) }
    }

    #[inline(always)]
    fn y(self) -> f32 {
        unsafe { f32::from_bits(_mm_extract_ps(self.0, 1) as u32) }
    }

    #[inline(always)]
    fn z(self) -> f32 {
        unsafe { f32::from_bits(_mm_extract_ps(self.0, 2) as u32) }
    }

    #[inline(always)]
    fn magnitude(self) -> f32 {
        unsafe {
            let len_squared = _mm_dp_ps(self.0, self.0, 0b0111_0001);
            let len = _mm_sqrt_ss(len_squared);
            _mm_cvtss_f32(len)
        }
    }

    #[inline(always)]
    fn is_on_z_axis(self) -> bool {
        unsafe {
            let mask = _mm_set_epi32(0, 0, 0x7FFF_FFFF, 0x7FFF_FFFF);
            _mm_test_all_zeros(_mm_castps_si128(self.0), mask) == 1
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(unsafe { _mm_add_ps(self.0, other.0) })
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(unsafe { _mm_sub_ps(self.0, other.0) })
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, other: f32) -> Self {
        Self(unsafe { _mm_mul_ps(self.0, Self::new(other, other, other).0) })
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        other * self
    }
}
