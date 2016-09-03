use rayon::par_iter::IntoParallelIterator;
use rayon::par_iter::ParallelIterator;
use rayon::par_iter::ExactParallelIterator;

/// Represents a 2D-matrix of `Color`s. This is only used to save the final
/// result and show it on the screen. The data is saved contiguous line by
/// line:
///
///  [0] [1] [2]
///  [3] [4] [5]
///  [6] [7] [8]
///
pub struct PixelImage {
    width: usize,
    height: usize,
    data: Vec<Color>,
}

impl PixelImage {
    pub fn monochrome(width: usize, height: usize, color: Color) -> Self {
        PixelImage {
            width: width,
            height: height,
            data: vec![color; width * height],
        }
    }

    pub fn from_pixels<F>(width: usize, height: usize, func: F) -> Self
        where F: Fn(usize, usize) -> Color + Sync
    {
        let mut data = Vec::with_capacity(width * height);

        (0..width*height)
            .into_par_iter()
            .map(|i| func(i % width, i / width)).collect_into(&mut data);

        PixelImage {
            width: width,
            height: height,
            data: data,
        }
    }

    pub fn to_u32_buffer(&self) -> Vec<u32> {
        self.data.iter().map(|color| color.to_u32()).collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

macro_rules! gen_color_ctor {
    ($name:ident => $r:expr, $g:expr, $b:expr) => {
        pub fn $name() -> Self {
            Color {
                r: $r,
                g: $g,
                b: $b,
            }
        }
    }
}

impl Color {
    gen_color_ctor!(black =>   0,   0,   0);
    gen_color_ctor!(white => 255, 255, 255);
    gen_color_ctor!(red   => 255,   0,   0);
    gen_color_ctor!(green =>   0, 255,   0);
    gen_color_ctor!(blue  =>   0,   0, 255);

    pub fn greyscale(level: u8) -> Self {
        Color {
            r: level,
            g: level,
            b: level,
        }
    }

    pub fn to_u32(&self) -> u32 {
        (self.r as u32) << (2 * 8) |
        (self.g as u32) << (1 * 8) |
        (self.b as u32) << (0 * 8)
    }
}
