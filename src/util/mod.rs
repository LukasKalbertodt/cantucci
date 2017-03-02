use core::math::*;

pub mod gl;
pub mod iter;
pub mod grid;
pub mod time;


pub trait ToArr {
    type Output;

    fn to_arr(&self) -> Self::Output;
}

macro_rules! to_arr_impl_gen_into_type {
    ($ty:ident, $out:ty) => {
        impl<T: BaseNum> ToArr for $ty <T> {
            type Output = $out;

            fn to_arr(&self) -> Self::Output {
                (*self).into()
            }
        }
    }
}

to_arr_impl_gen_into_type!(Matrix4, [[T; 4]; 4]);
to_arr_impl_gen_into_type!(Point3, [T; 3]);
to_arr_impl_gen_into_type!(Vector3, [T; 3]);
