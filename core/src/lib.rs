extern crate cgmath;

mod types;
mod math;
pub use types::{PixelImage, Color};
use math::*;


pub fn get_circle(width: usize, height: usize) -> PixelImage {
    PixelImage::from_pixels(width, height, |x, y| {
        // let x = (x as f64) / (width as f64 / 4.0) - 2.0;
        // let y = (y as f64) / (height as f64 / 4.0) - 2.0;

        // let mut acc = (x, y);
        // const MAX_ITERS: u64 = 255;
        // let mut iter = 0;
        // while acc.0 * acc.0 + acc.1 * acc.1 < 4.0 && iter < MAX_ITERS {
        //     let (a, b) = acc;
        //     acc = (a * a - b * b + x, 2.0 * a * b + y);
        //     iter += 1;
        // }

        // Color::greyscale((255.0 * (iter as f32 / MAX_ITERS as f32)) as u8)

        let origin_x = (x as f64) / (width as f64 / 4.0) - 2.0;
        let origin_z = (y as f64) / (height as f64 / 4.0) - 2.0;
        let origin = Point3::new(origin_x, -10.0, origin_z);
        let dir = Vector3::new(0.0, 1.0, 0.0);

        let surface_point = march_ray(origin, dir, |p| {
            let a = p.to_vec().magnitude() - 1.0;
            let b = (p - Point3f::new(1.0, 1.0, 1.0)).magnitude() - 1.0;
            if a < b { a } else { b }
        });

        match surface_point {
            Some(surface) => {
                Color::greyscale((255.0 * (surface.march_iters as f32 / 30.0)) as u8)
            }
            None => Color::white()
        }
    })
}


struct SurfacePoint {
    pos: Point3f,
    // normal: Point3f,
    march_iters: u32,
}

fn march_ray<E>(origin: Point3f, dir: Vector3f, mut de: E) -> Option<SurfacePoint>
    where E: FnMut(Point3f) -> DefaultFloat
{
    const MAX_ITERS: u32 = 30;
    const DISTANCE_THRESHOLD: f64 = 0.01;
    let dir = dir.normalize();

    let mut iter = 0;
    let mut p = origin;

    while iter < MAX_ITERS {
        let lower_bound = de(p);
        if lower_bound < DISTANCE_THRESHOLD {
            return Some(SurfacePoint {
                pos: p,
                march_iters: iter,
            });
        }

        p += dir * lower_bound;
        iter += 1;
    }

    None
}
