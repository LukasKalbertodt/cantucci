extern crate core;
extern crate minifb;

use minifb::{Key, Scale, WindowOptions, Window};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    let mut window = Window::new("Cantucci", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Unable to create window");

    window.update();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (width, height) = (WIDTH, HEIGHT);
        let image = core::get_circle(width, height);
        let buffer = image.to_u32_buffer();

        window.update_with_buffer(&buffer);
    }
}
