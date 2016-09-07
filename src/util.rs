#![allow(dead_code)]

use core::math::*;
use std::ops::Sub;

pub trait ToArr {
    type Output;

    fn to_arr(&self) -> Self::Output;
}

impl<T: BaseNum> ToArr for Matrix4<T> {
    type Output = [[T; 4]; 4];

    fn to_arr(&self) -> Self::Output {
        (*self).into()
    }
}

pub fn lerp<V, F>(a: V, b: V, t: F) -> V
    where V: Lerp<F>,
          F: LerpFactor
{
    assert!(t >= F::zero() && t <= F::one());
    a.lerp(b, t)
}

pub trait LerpFactor: Zero + One + PartialOrd + Sub<Output=Self> {}
impl LerpFactor for f32 {}
impl LerpFactor for f64 {}

pub trait Lerp<F: LerpFactor> {
    fn lerp(self, other: Self, t: F) -> Self;
}

macro_rules! impl_lerp {
    ($self_type:ty, $factor:ty) => {
        impl Lerp<$factor> for $self_type {
            fn lerp(self, other: Self, t: $factor) -> Self {
                self * (1.0 - t) + other * t
            }
        }
    }
}

impl_lerp!(Vector3<f64>, f64);
impl_lerp!(f64, f64);
impl_lerp!(Rad<f64>, f64);

impl Lerp<f64> for Point3<f64> {
    fn lerp(self, other: Self, t: f64) -> Self {
        self * (1.0 - t) + other.to_vec() * t
    }
}


// impl<V, F> Lerp<F> for V
//     where F: LerpFactor,
//           V: Mul<F>,
//           V::Output: Add<Output=V>
// {
//     fn lerp(self, other: Self, t: F) -> Self {
//         self * (F::one() - t) + other * t
//     }
// }

// impl<T, F> Lerp<F> for Point3<T>
//     where F: LerpFactor,
//           Point3<T>: Mul<F>,
//           <Point3<T> as Mul<F>>::Output: Add<Output=Point3<T>>
// {
//     fn lerp(self, other: Self, t: F) -> Self {
//         self * (F::one() - t) + other * t
//     }
// }

// impl<F> Lerp<F> for Point3<f64>
//     where F: LerpFactor,
//           Point3<f64>: Mul<F>,
//           <Vector3<f64> as Mul<F>>::Output: Add<Output=Vector3<f64>>
// {
//     fn lerp(self, other: Self, t: F) -> Self {
//         self * (F::one() - t) + other * t
//     }
// }

// impl<V, F> Lerp<F> for V
//     where V: VectorSpace<Scalar = F>,
//           F: LerpFactor + BaseNum
// {
//     fn lerp(self, other: Self, t: F) -> Self {
//         self * (F::one() - t) + other * t
//     }
// }

// impl<T: BaseNum, F: LerpFactor> Lerp<F> for T {
//     fn lerp(self, other: Self, t: F) -> Self {
//         self * (F::one() - t) + other * t
//     }
// }

pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    assert!(min < max);

    match () {
        () if val < min => min,
        () if val > max => max,
        _ => val,
    }
}
