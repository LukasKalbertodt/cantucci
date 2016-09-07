extern crate cgmath;
extern crate rayon;

mod types;
pub mod shape;
pub mod math;

pub use types::{PixelImage, Color};
pub use shape::Shape;
// use math::*;
// use std::time::Instant;


pub fn get_circle(width: usize, height: usize, _angle: f64) -> PixelImage {

    let out = PixelImage::from_pixels(width, height, |_, _| {

        Color::white()

    //     const WINDOW: f64 = 3.0;
    //     let origin_x = (x as f64) / (width as f64 / WINDOW) - (WINDOW / 2.0);
    //     let origin_z = (y as f64) / (height as f64 / WINDOW) - (WINDOW / 2.0);
    //     // let origin = Point3::new(origin_x, -2.0, origin_z - 1.0);
    //     // let dir = Vector3::new(0.0, 1.0, 0.2);

    //     let rot = Basis3::from_angle_y(Rad(angle));

    //     let origin = rot.rotate_point(Point3::new(origin_x, origin_z, -1.0));
    //     let dir = Vector3::new(angle.sin(), 0.2, angle.cos());

    //     let surface_point = march_ray(origin, dir, |p| {
    //         let mut p = p;
    //         std::mem::swap(&mut p.y, &mut p.z);
    //         let mut z = p;
    //         let mut dr = 1.0;
    //         let mut r = 0.0;

    //         const MAX_ITERS: u32 = 10;
    //         const BAILOUT: f64 = 2.5;
    //         const POWER: f64 = 8.0;

    //         for i in 0..MAX_ITERS {

    //             r = z.to_vec().magnitude();
    //             if r > BAILOUT {
    //                 break;
    //             }

    //             // convert to polar coordinates
    //             let theta = (z.z / r).acos();
    //             // let theta = f64::atan2((z.x * z.x + z.y * z.y).sqrt(), z.z);
    //             let phi = f64::atan2(z.y, z.x);
    //             dr = r.powf(POWER - 1.0) * POWER * dr + 1.0;

    //             // scale and rotate the point
    //             let zr = r.powf(POWER);
    //             let theta = theta * POWER;
    //             let phi = phi * POWER;

    //             // convert back to cartesian coordinates
    //             z = zr * Point3f::new(
    //                 theta.sin() * phi.cos(),
    //                 phi.sin() * theta.sin(),
    //                 theta.cos()
    //             );
    //             z += p.to_vec();
    //         }

    //         0.5 * r.ln() * r / dr

    //     });

    //     match surface_point {
    //         Some(surface) => {
    //             let c = Color::greyscale(
    //                 (200.0 - (200.0 *
    //                     (surface.march_iters as f32 / MAX_MARCH_ITERS as f32)
    //                 )) as u8
    //             );

    //             // println!("{:?}", origin);

    //             // let c = Color {
    //             //     r: (surface.pos.y * 128.0).abs() as u8,
    //             //     g: (surface.pos.y * 128.0).abs() as u8,
    //             //     b: (surface.pos.y * 128.0).abs() as u8,
    //             // };
    //             // println!("{:?}", c);

    //             c
    //         }
    //         None => Color::white()
    //     }
    });

    out
}

// const MAX_MARCH_ITERS: u32 = 400;

// struct SurfacePoint {
//     pos: Point3f,
//     // normal: Point3f,
//     march_iters: u32,
// }

// fn march_ray<E>(origin: Point3f, dir: Vector3f, mut de: E) -> Option<SurfacePoint>
//     where E: FnMut(Point3f) -> DefaultFloat
// {
//     const DISTANCE_THRESHOLD: f64 = 0.001;
//     const BAILOUT: f64 = 2.5;
//     let dir = dir.normalize();

//     let mut iter = 0;
//     let mut p = origin;

//     // println!("---------------");
//     while iter < MAX_MARCH_ITERS {
//         let lower_bound = de(p);
//         // println!("{}", lower_bound);
//         if lower_bound < DISTANCE_THRESHOLD {
//             return Some(SurfacePoint {
//                 pos: p,
//                 march_iters: iter,
//             });
//         }
//         if lower_bound > BAILOUT {
//             break;
//         }

//         p += dir * lower_bound;
//         iter += 1;
//     }

//     None
// }
