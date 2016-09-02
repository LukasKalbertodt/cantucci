
///
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
    pub fn black(width: usize, height: usize) -> Self {
        PixelImage {
            width: width,
            height: height,
            data: vec![Color::black(); width * height],
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

impl Color {
    pub fn black() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0,
        }
    }

    pub fn to_u32(&self) -> u32 {
        (self.r as u32) << (2 * 8) |
        (self.g as u32) << (1 * 8) |
        (self.b as u32) << (0 * 8)
    }
}
