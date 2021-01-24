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
            // TODO: this here should return the magnitude² as we need it ...
            r = z.magnitude();
            if r > self.bailout {
                break;
            }

            // ... here in the ^7 thingy.
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
        return rotate_on_z_axis::<P>(p);
    }


    // For some integer powers there are formulas without trigonometric
    // functions. This improves performance a lot (see #17).
    match P {
        // 8 => rotate_inner_p8_serial(p),
        8 => unsafe { rotate_inner_p8_simd(p) },
        _ => rotate_inner_px_generic::<P>(p),
    }
}

#[inline(never)]
#[cold]
fn rotate_on_z_axis<const P: u8>(p: Vec3) -> Vec3 {
    let old_radius = p.magnitude();
    let theta = (p.z() / old_radius).acos();

    // Scale and rotate the point
    let new_radius = old_radius.powi(P.into());
    let theta = theta * P as f32;

    // Convert back to cartesian coordinates
    Vec3::new(0.0, 0.0, new_radius * theta.cos())
}

fn rotate_inner_px_generic<const P: u8>(p: Vec3) -> Vec3 {
    let old_radius = p.magnitude();

    // Convert to spherical coordinates
    let theta = (p.z() / old_radius).acos();
    let phi = f32::atan2(p.y(), p.x());

    // Scale and rotate the point
    let new_radius = old_radius.powi(P.into());
    let theta = theta * P as f32;
    let phi = phi * P as f32;

    // Convert back to cartesian coordinates
    new_radius * Vec3::new(
        theta.sin() * phi.cos(),
        phi.sin() * theta.sin(),
        theta.cos(),
    )
}

fn rotate_inner_p8_scalar(p: Vec3) -> Vec3 {
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

#[inline(never)]
unsafe fn rotate_inner_p8_simd(p: Vec3) -> Vec3 {
    use core::arch::x86_64::*;

    let p = p.0;

    // We first calculate a bunch of powers of x, y, z and (x² + y²). The last
    // value we define as "w²". To be precise, we need:
    //
    //       x  x²  x⁴  x⁶  x⁸
    //       y  y²  y⁴  y⁶  y⁸
    //       z  z²  z⁴  z⁶  z⁸
    //       -  w²  w⁴  w⁶  w⁸
    // var = p  p2  p4  p6  p8
    //
    //
    // We will calculate higher powers by multiplying. (x, y, z) is stored in
    // `p`. The subsequent powers will be stored in p2, p4, p6 and p8. The
    // highest component of these SIMD vectors will be wⁿ (except in p).

    // w² = x² + y² (In every position)
    let w2_everywhere = _mm_dp_ps(p, p, 0b0011_1111);

    // First simply multiply p with itself to get x², y² and z². Then we set the
    // highest component of `p2` to w² by bitwise or. In the line above we made
    // sure that the dest mask of `_mm_dp_ps` writes to the highest component.
    // All other are 0.
    let p2 = _mm_mul_ps(p, p);
    let p2 = _mm_blend_ps(p2, w2_everywhere, 0b1000);

    // Time to create the other powers.
    let p4 = _mm_mul_ps(p2, p2);
    let p6 = _mm_mul_ps(p4, p2);
    let p8 = _mm_mul_ps(p4, p4);

    // For later caculations it is beneficial to have all xs, ys, zs and ws in
    // one vector. So what we do here is basically a 4x4 matrix transpose. We do
    // this with the powers 2, 4, 6, 8. The original `p` is not involved in
    // this.
    let (xs, ys, zs, ws) = {
        let x2_x4_y2_y4 = _mm_unpacklo_ps(p2, p4);
        let x6_x8_y6_y8 = _mm_unpacklo_ps(p6, p8);
        let z2_z4_w2_w4 = _mm_unpackhi_ps(p2, p4);
        let z6_z8_w6_w8 = _mm_unpackhi_ps(p6, p8);

        (
            _mm_movelh_ps(x2_x4_y2_y4, x6_x8_y6_y8),
            _mm_movehl_ps(x6_x8_y6_y8, x2_x4_y2_y4),
            _mm_movelh_ps(z2_z4_w2_w4, z6_z8_w6_w8),
            _mm_movehl_ps(z6_z8_w6_w8, z2_z4_w2_w4),
        )
    };

    // Some constants we need.
    let ones = _mm_set1_ps(1.0); // Four 1.0
    let const_1_n28_70_n28 = _mm_set_ps(1.0, -28.0, 70.0, -28.0);

    // Calculate temporary value `a`. The value is stored in the lowest two
    // components of the register, the highest two registers are 1.0. The value
    // of a is calculated like this:
    //
    //     a = 1.0 + sum(
    //         + 1.0 * z8 * (1.0 / w8)       // [96..127]
    //         -28.0 * z6 * (w2  / w8)       // [64..95]
    //         +70.0 * z4 * (w4  / w8)       // [32..63]
    //         -28.0 * z2 * (w6  / w8)       // [0..31]
    //     )
    //
    // To do this efficiently, we calculate `temp` by preparing three vectors,
    // multiplying two of them and then calculating the dot product of that
    // result with the third.
    let a = {
        let tmp = _mm_dp_ps(
            _mm_mul_ps(const_1_n28_70_n28, zs),
            _mm_div_ps(ones, ws),
            0b1111_0011,
        );

        _mm_add_ps(ones, tmp)
    };


    // Some values for the calculations below.
    let c1_y2_y4_y6 = {
        let y2_y2_y4_y6 = _mm_permute_ps(ys, 0b00_00_01_10);
        _mm_blend_ps(y2_y2_y4_y6, ones, 0b1000)
    };

    // `xtmp` is a scalar caculated as:
    //
    //     xtmp = -y8 +
    //         + 1 * x8 * 1    +
    //         -28 * x6 * y2   +
    //         +70 * x4 * y4   +
    //         -28 * x2 * y6
    //
    let xtmp = {
        let tmp = _mm_dp_ps(
            _mm_mul_ps(const_1_n28_70_n28, xs),
            c1_y2_y4_y6,
            0b1111_0001,
        );


        let y8 = f32::from_bits(_mm_extract_ps(ys, 3) as u32);
        _mm_cvtss_f32(tmp) - y8
    };

    // `ytmp` is a scalar caculated as:
    //
    //     ytmp = x * y * (
    //         +1 * x6 * 1     +
    //         -7 * x4 * y2    +
    //         +7 * x2 * y4    +
    //         -1 * 1  * y6
    //     )
    let ytmp = {
        let x6_x4_x2_x2 = _mm_permute_ps(xs, 0b10_01_00_00);
        let x6_x4_x2_c1 = _mm_blend_ps(x6_x4_x2_x2, ones, 0b0001);

        let tmp = _mm_dp_ps(
            _mm_mul_ps(c1_y2_y4_y6, x6_x4_x2_c1),
            _mm_set_ps(1.0, -7.0, 7.0, -1.0),
            0b1111_0010,
        );

        // Multiply with y. Since `tmp` has 0 in every position except the
        // second, the non second elements in the other factor don't matter.
        let tmp = _mm_mul_ps(tmp, p);
        let tmp = f32::from_bits(_mm_extract_ps(tmp, 1) as u32);
        let x = _mm_cvtss_f32(p);

        x * tmp
    };

    // `ztmp` is a scalar caculated as:
    //
    //     ztmp = z
    //         * w
    //         * (z2 - w²)
    //         * (z4 - 6.0 * z2 * w² + w⁴),
    let ztmp = {
        let z = f32::from_bits(_mm_extract_ps(p, 2) as u32);
        let z2 = _mm_cvtss_f32(zs);
        let z4 = f32::from_bits(_mm_extract_ps(zs, 1) as u32);
        let w2 = _mm_cvtss_f32(ws);
        let w4 = f32::from_bits(_mm_extract_ps(ws, 1) as u32);

        z * w2.sqrt()
            * (z2 - w2)
            * (z4 - 6.0 * z2 * w2 + w4)
    };

    // Caculate the final result. It is caculated as:
    //
    //     z = 8 * 1 * ztmp
    //     y = 8 * a * ytmp
    //     x = 1 * a * xtmp
    let xyztmp = _mm_set_ps(0.0, ztmp, ytmp, xtmp);
    Vec3(
        _mm_mul_ps(
            _mm_mul_ps(xyztmp, a),
            _mm_set_ps(0.0, 8.0, 8.0, 1.0),
        )
    )
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

#[cfg(test)]
mod bench {
    use test::{Bencher, black_box};
    use super::Vec3;
    use super::super::BENCH_POINTS;

    #[bench]
    fn rotate_inner_generic(b: &mut Bencher) {
        b.iter(|| {
            for &[x, y, z] in &BENCH_POINTS {
                black_box(super::rotate_inner_px_generic::<8>(Vec3::new(x, y, z)));
            }
        });
    }

    #[bench]
    fn rotate_inner_p8_scalar(b: &mut Bencher) {
        b.iter(|| {
            for &[x, y, z] in &BENCH_POINTS {
                black_box(super::rotate_inner_p8_scalar(Vec3::new(x, y, z)));
            }
        });
    }

    #[bench]
    fn rotate_inner_p8_simd(b: &mut Bencher) {
        b.iter(|| {
            for &[x, y, z] in &BENCH_POINTS {
                black_box(unsafe { super::rotate_inner_p8_simd(Vec3::new(x, y, z)) });
            }
        });
    }
}
