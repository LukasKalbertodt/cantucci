use camera::Projection;
use control::Orbit as OrbitControl;
use core::math::*;
use errors::*;
use event::{EventResponse, poll_events_with, QuitHandler};
use glium::backend::glutin_backend::GlutinFacade;
use mesh::FractalMesh;

const WINDOW_TITLE: &'static str = "Cantucci ◕ ◡ ◕";

pub struct App {
    facade: GlutinFacade,
    control: OrbitControl,
    mesh: FractalMesh,
}

impl App {
    /// Creates all needed resources, including the OpenGL context.
    pub fn init() -> Result<Self> {
        // Create OpenGL context
        let facade = try!(
            create_context().chain_err(|| "failed to create GL context")
        );

        let mesh = FractalMesh::new(&facade);

        let proj = Projection::new(
            Rad(1.0),
            0.01 .. 100.0,
            facade.get_framebuffer_dimensions(),
        );

        let orbit = OrbitControl::around(Point3::new(0.0, 0.0, 0.0), proj);

        Ok(App {
            facade: facade,
            control: orbit,
            mesh: mesh,
        })
    }

    /// Contains the main loop used to show stuff on the screen.
    pub fn run(&mut self) -> Result<()> {
        use glium::Surface;

        loop {
            self.control.update(0.02);

            let mut target = self.facade.draw();
            target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

            self.mesh.draw(&mut target, self.control.camera());

            target.finish().unwrap();


            // Poll window events
            let res = self.poll_events();
            if res == EventResponse::Quit {
                info!("Bye! :)");
                return Ok(());
            }
        }
    }

    fn poll_events(&mut self) -> EventResponse {
        use glium::glutin::Event;

        let mut new_res = None;

        let out = poll_events_with(&self.facade, vec![
            &mut self.control,
            &mut QuitHandler,
            &mut |e: &Event| {
                if let Event::Resized(w, h) = *e {
                    new_res = Some((w, h));
                    EventResponse::Continue
                } else {
                    EventResponse::NotHandled
                }
            },
        ]);

        if let Some((w, h)) = new_res {
            self.control.projection_mut().set_aspect_ratio(w, h);
            trace!("resolution changed to {}x{}px", w, h);
        }

        out
    }
}


/// Creates the OpenGL context and logs useful information about the
/// success or failure of said action.
fn create_context() -> Result<GlutinFacade> {
    use glium::glutin::{self, get_primary_monitor, GlRequest};
    use glium::{self, DisplayBuild};

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

    let context = try!(context);

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
