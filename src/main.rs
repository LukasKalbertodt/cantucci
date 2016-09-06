#![recursion_limit = "1024"]

extern crate core;
extern crate cgmath;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;


mod mesh;
mod errors;
mod camera;
mod control;
mod to_arr;

use glium::glutin::{self, Event, VirtualKeyCode, GlRequest};
use glium::backend::glutin_backend::GlutinFacade;
use mesh::FractalMesh;
use camera::{Camera, Projection};
use core::math::*;
use control::Orbit as OrbitControl;


fn main() {
    env_logger::init().unwrap();

    use glium::{DisplayBuild, Surface};
    let display = create_context().unwrap();

    println!("{:?}", display.get_opengl_version());
    println!("{:?}", display.get_supported_glsl_version());
    let mesh = FractalMesh::new(&display);

    let proj = Projection {
        fov: Rad(1.0),
        aspect_ratio: 1.0,
        near_plane: 0.01,
        far_plane: 100.0,
    };

    let mut orbit = OrbitControl::around(Point3::new(0.0, 0.0, 0.0), proj);

    loop {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        mesh.draw(&mut target, orbit.camera());

        target.finish().unwrap();

        for ev in display.poll_events() {
            orbit.handle_event(&ev);
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return,
                _ => ()
            }
        }
    }
}

struct App {
    facade: GlutinFacade,
}

impl App {
    pub fn new() {

    }


}



/// Creates the OpenGL context and logs useful information about the
/// success or failure of said action.
fn create_context() -> Result<GlutinFacade, ()> {
    use glium::glutin::get_primary_monitor;
    use glium::DisplayBuild;

    // Check resolution of monitor
    let monitor = get_primary_monitor();
    let (monitor_width, monitor_height) = monitor.get_dimensions();
    info!(
        "Starting on monitor '{}' ({}x{}px)",
        monitor.get_name().unwrap_or("???".into()),
        monitor_width,
        monitor_height,
    );

    // Create glium context
    let context = glutin::WindowBuilder::new()
        .with_dimensions(monitor_width / 2, monitor_height / 2)
        .with_title("Cantucci <3")
        .build_glium();

    match context {
        Err(e) => {
            error!("OpenGL context creation failed! Detailed error:");
            error!("{}", e);

            Err(())
        }
        Ok(context) => {
            // Print some information about the acquired OpenGL context
            info!("OpenGL context was successfully built");

            let glium::Version(api, major, minor) = *context.get_opengl_version();
            info!("Version of context: {} {}.{}",
                  if api == glium::Api::Gl { "OpenGL" } else { "OpenGL ES" },
                  major,
                  minor);

            let glium::Version(api, major, minor) = context.get_supported_glsl_version();
            info!("Supported GLSL version: {} {}.{}",
                  if api == glium::Api::Gl { "GLSL" } else { "GLSL ES" },
                  major,
                  minor);

            if let Some(mem) = context.get_free_video_memory() {
                let (mem, unit) = match () {
                    _ if mem < (1 << 12) => (mem, "B"),
                    _ if mem < (1 << 22) => (mem >> 10, "KB"),
                    _ if mem < (1 << 32) => (mem >> 20, "MB"),
                    _ => (mem >> 30, "GB"),
                };
                info!("Free GPU memory (estimated): {}{}", mem, unit);
            }

            Ok(context)
        }
    }
}
