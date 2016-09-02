extern crate core;
extern crate minifb;

use minifb::{Key, WindowOptions, Scale, Window};

const WIDTH: usize = 400;
const HEIGHT: usize = 400;

fn main() {
    let win_opt = WindowOptions {
        scale: Scale::X2,
        .. WindowOptions::default()
    };

    let mut window = Window::new("Cantucci", WIDTH, HEIGHT, win_opt)
        .expect("Unable to create window");

    window.update();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (width, height) = (WIDTH, HEIGHT);
        let image = core::get_circle(width, height);
        let buffer = image.to_u32_buffer();

        window.update_with_buffer(&buffer);
        println!("wup");
    }
}
