extern crate cgmath;

use std::ops::Sub;
use std::fmt;

use cgmath::prelude::*;
use cgmath::{
    Vector3,
    Point3,
    Rad,
};


pub(crate) fn lerp<V, F>(a: V, b: V, t: F) -> V
where
    V: Lerp<F>,
    F: LerpFactor + fmt::Debug,
{
    assert!(t >= F::zero() && t <= F::one(), "factor was {:?}", t);
    a.lerp(b, t)
}

pub trait LerpFactor: Zero + One + PartialOrd + Sub<Output=Self> {}
impl LerpFactor for f64 {}
impl LerpFactor for f32 {}

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

impl_lerp!(Vector3<f32>, f32);
impl_lerp!(f32, f32);
impl_lerp!(Rad<f32>, f32);

impl Lerp<f32> for Point3<f32> {
    fn lerp(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other.to_vec() * t
    }
}
