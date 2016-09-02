extern crate cgmath;

mod types;

pub use types::{PixelImage, Color};

pub fn get_circle(width: usize, height: usize) -> PixelImage {
    PixelImage::from_pixels(width, height, |x, y| {
        let x = (x as f64) / (width as f64 / 4.0) - 2.0;
        let y = (y as f64) / (height as f64 / 4.0) - 2.0;

        let mut acc = (x, y);
        const MAX_ITERS: u64 = 255;
        let mut iter = 0;
        while acc.0 * acc.0 + acc.1 * acc.1 < 4.0 && iter < MAX_ITERS {
            let (a, b) = acc;
            acc = (a * a - b * b + x, 2.0 * a * b + y);
            iter += 1;
        }

        Color::greyscale((255.0 * (iter as f32 / MAX_ITERS as f32)) as u8)
    })
}
