extern crate cgmath;

use cgmath::Point2;

mod types;

pub use types::{PixelImage, Color};

pub fn get_circle(width: usize, height: usize) -> PixelImage {
    let out = PixelImage::monochrome(width, height, Color::white());

    let radius = (std::cmp::min(width, height) as f32) * 0.7 / 2.0;
    let center = Point2::new((width as f32) / 2.0, (height as f32) / 2.0);

    PixelImage::from_pixels(width, height, |x, y| {
        let x = x as f32 - center.x;
        let y = y as f32 - center.y;

        if ((x * x + y * y) as f32) < radius * radius {
            Color::red()
        } else {
            Color::white()
        }
    })
}
