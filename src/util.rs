#![allow(dead_code)]

use core::math::*;
use std::ops::{Add, Mul};

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


pub fn lerp<T>(a: T, b: T, t: f64) -> <<T as Mul<f64>>::Output as Add>::Output
    where T: Mul<f64>,
          T::Output: Add
{
    assert!(t >= 0.0 && t <= 1.0);
    a* (1.0 - t) + b * t
}

pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    assert!(min < max);

    match () {
        () if val < min => min,
        () if val > max => max,
        _ => val,
    }
}
