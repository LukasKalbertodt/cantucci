mod types;

pub use types::{PixelImage, Color};

pub fn get_circle(width: usize, height: usize) -> PixelImage {
    PixelImage::black(width, height)
}
