extern crate core;
#[macro_use]
extern crate glium;

use glium::glutin::{Event, VirtualKeyCode};

mod mesh;

use mesh::FractalMesh;


fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
    let mesh = FractalMesh::new(&display);

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        mesh.draw(&mut target);

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return,
                _ => ()
            }
        }
    }
}
