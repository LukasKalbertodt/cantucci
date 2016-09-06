use core::math::*;

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
