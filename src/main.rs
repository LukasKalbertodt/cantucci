extern crate core;
extern crate minifb;

use minifb::{Key, WindowOptions, Scale, Window};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

fn main() {
    let win_opt = WindowOptions {
        scale: Scale::X2,
        .. WindowOptions::default()
    };

    let mut window = Window::new("Cantucci", WIDTH, HEIGHT, win_opt)
        .expect("Unable to create window");

    window.update();

    let mut angle = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (width, height) = (WIDTH, HEIGHT);
        let image = core::get_circle(width, height, angle);
        let buffer = image.to_u32_buffer();

        window.update_with_buffer(&buffer);
        println!("wup");

        angle += 0.1;
    }
}
