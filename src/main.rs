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
use camera::Projection;
use core::math::*;
use control::Orbit as OrbitControl;

const WINDOW_TITLE: &'static str = "Cantucci <3";

fn main() {
    env_logger::init().unwrap();

    let mut app = App::init();
    app.run();
}

struct App {
    facade: GlutinFacade,
    control: OrbitControl,
    mesh: FractalMesh,
}

impl App {
    pub fn init() -> Self {
        // TODO: proper error handling
        let facade = create_context().unwrap();

        let mesh = FractalMesh::new(&facade);

        let proj = Projection::new(
            Rad(1.0),
            0.01 .. 100.0,
            facade.get_framebuffer_dimensions(),
        );

        let orbit = OrbitControl::around(Point3::new(0.0, 0.0, 0.0), proj);

        App {
            facade: facade,
            control: orbit,
            mesh: mesh,
        }
    }

    pub fn run(&mut self) {
        use glium::Surface;

        loop {
            let mut target = self.facade.draw();
            target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

            self.mesh.draw(&mut target, self.control.camera());

            target.finish().unwrap();

            for ev in self.facade.poll_events() {
                self.control.handle_event(&ev);
                match ev {
                    Event::Closed => return,
                    Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return,
                    _ => ()
                }
            }
        }
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
        .with_title(WINDOW_TITLE)
        .with_gl(GlRequest::Latest)
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
            info!(
                "Version of context: {} {}.{}",
                if api == glium::Api::Gl { "OpenGL" } else { "OpenGL ES" },
                major,
                minor
            );

            let glium::Version(api, major, minor) = context.get_supported_glsl_version();
            info!(
                "Supported GLSL version: {} {}.{}",
                if api == glium::Api::Gl { "GLSL" } else { "GLSL ES" },
                major,
                minor
            );

            if let Some(mem) = context.get_free_video_memory().map(|mem| mem as u64) {
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
