extern crate core;
extern crate minifb;

use minifb::{Key, Scale, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut window = match minifb::Window::new("Test - ESC to exit", WIDTH, HEIGHT,
                                               WindowOptions::default()) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            return;
        }
    };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let image = core::get_circle(WIDTH, HEIGHT);
        let buffer = image.to_u32_buffer();

        window.update_with_buffer(&buffer);
    }
}
