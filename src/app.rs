// use glium::backend::glutin_backend::GlutinFacade;
// use std::sync::Arc;

// use camera::Projection;
// use control::Fly as FlyControl;
// use control::Orbit as OrbitControl;
// use control::{CamControl, KeySwitcher};
// use math::*;
// use shape::{Shape, Mandelbulb, Sphere};
// use env::Environment;
// use errors::*;
// use event::{EventResponse, poll_events_with, QuitHandler};
// use mesh::ShapeMesh;

use std::{
    rc::Rc,
    time::{Duration, Instant},
};


use cgmath::{Point3, Rad};
use winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    camera::Projection,
    control::{CamControl, Fly as FlyControl, KeySwitcher, Orbit as OrbitControl},
    event::{EventHandler, EventResponse, QuitHandler},
    prelude::*,
};

const WINDOW_TITLE: &'static str = "Cantucci ◕ ◡ ◕";

pub(crate) async fn run() -> Result<()> {
    let event_loop = EventLoop::new();
    debug!("Created event loop");

    let window = Window::new(&event_loop).context("failed to open window")?;
    window.set_title(WINDOW_TITLE);
    // TODO: maybe chose initial dimension of the window
    debug!("Created window");

    let mut app = App::new(Rc::new(window)).await?;

    info!("Initialized app");
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => app.draw(),
            Event::LoopDestroyed => info!("Bye :-)"),

            // Explicitly list all the events we don't handle (currently)
            Event::NewEvents(_)
            | Event::UserEvent(_)
            | Event::Suspended
            | Event::Resumed
            | Event::MainEventsCleared
            | Event::RedrawEventsCleared => {}

            // Forward window and device events to handlers.
            event => {
                let resp = app.handle_event(&event);
                if resp == EventResponse::Quit {
                    *control_flow = ControlFlow::Exit;
                }
            }
        }
    });
}

struct App {
    window: Rc<Window>,
    device: wgpu::Device,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    queue: wgpu::Queue,

    control: KeySwitcher<OrbitControl, FlyControl>,
    fps_timer: FpsTimer,
}

impl App {
    async fn new(window: Rc<Window>) -> Result<Self> {
        info!("Initializing wgpu...");

        // Create an instance, just a temporary object to get access to other
        // objects.
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        debug!("Created wgpu instance");

        // Create the surface, something we can draw on (or well, we will create
        // a swapchain from). The surface call is `unsafe` because we must
        // ensure `window` is a valid raw window handle to create a surface on.
        // Let's just assume it is.
        let surface = unsafe { instance.create_surface(&*window) };
        debug!("Created wgpu surface");

        // The adapter is a physical device. The variable is only temporary and
        // only used to create a "logical device" (the `device`).
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .context("Failed to find an appropiate adapter")?;

        debug!("Created wgpu adapter: {:#?}", adapter.get_info());
        trace!("Adapter features: {:#?}", adapter.features());
        trace!("Adapter limits: {:#?}", adapter.limits());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .context("Failed to create device")?;

        debug!("Created wgpu device (including a queue)");
        trace!("Device features: {:#?}", device.features());
        trace!("Device limits: {:#?}", device.limits());

        let desc = swap_chain_description(window.inner_size());
        let swap_chain = device.create_swap_chain(&surface, &desc);
        debug!("Created swapchain with dimensions {}x{}", desc.width, desc.height);

        info!("Finished wgpu intialization");

        // Initialize our projection parameters.
        let proj = Projection::new(Rad(1.0), 0.000_04..10.0, window.inner_size().into());

        let orbit = OrbitControl::around(Point3::new(0.0, 0.0, 0.0), proj);
        let fly = FlyControl::new(orbit.camera().clone(), window.clone());
        let switcher = KeySwitcher::new(orbit, fly, VirtualKeyCode::F);

        Ok(Self {
            window,
            device,
            surface,
            swap_chain,
            queue,

            control: switcher,
            fps_timer: FpsTimer::new(),
        })
    }

    fn recreate_swap_chain(&mut self, new_size: PhysicalSize<u32>) {
        let desc = swap_chain_description(new_size);
        self.swap_chain = self.device.create_swap_chain(&self.surface, &desc);
    }

    fn draw(&mut self) {
        self.fps_timer.register_frame();
        if let Some(fps) = self.fps_timer.report_fps() {
            self.window.set_title(&format!("{} ({:.1} fps)", WINDOW_TITLE, fps))
        }
        self.window.request_redraw();
    }
}

impl EventHandler for App {
    fn handle_event(&mut self, e: &Event<()>) -> EventResponse {
        if let Event::WindowEvent {
            event: WindowEvent::Resized(new_size),
            ..
        } = e
        {
            self.recreate_swap_chain(*new_size);
            debug!("Window dimension changed to {:?}", new_size);
            return EventResponse::Break;
        }

        crate::event::handle_with(e, &mut [&mut QuitHandler, &mut self.control])
    }
}

fn swap_chain_description(size: PhysicalSize<u32>) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    }
}


/// How often the FPS are reported. Longer times lead to more delay and more
/// smoothing.
const REPORT_INTERVAL: Duration = Duration::from_millis(250);

pub(crate) struct FpsTimer {
    last_report: Instant,
    frames_since_last_report: u32,
}

impl FpsTimer {
    fn new() -> Self {
        Self {
            last_report: Instant::now(),
            frames_since_last_report: 0,
        }
    }

    fn register_frame(&mut self) {
    self.frames_since_last_report += 1;
}

    /// Returns `Some(fps)` every `REPORT_INTERVAL`.
    pub(crate) fn report_fps(&mut self) -> Option<f64> {
        let elapsed = self.last_report.elapsed();
        if elapsed >= REPORT_INTERVAL {
            let fps = self.frames_since_last_report as f64 / elapsed.as_secs_f64();
            self.last_report = Instant::now();
            self.frames_since_last_report = 0;

            Some(fps)
        } else {
            None
        }
    }
}


// pub struct App {
//     facade: GlutinFacade,
//     mesh: ShapeMesh,
//     env: Environment,
//     print_fps: bool,
// }

// impl App {
//     /// Creates all needed resources, including the OpenGL context.
//     pub fn init() -> Result<Self> {
//         // Create OpenGL context
//         let facade = create_context()
//             .chain_err(|| "failed to create GL context")?;

//         let shape = if ::std::env::args().len() > 1 {
//             Arc::new(Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0)) as Arc<dyn Shape>
//         } else {
//             Arc::new(Mandelbulb::classic(6, 2.5)) as Arc<dyn Shape>
//         };
//         let mesh = ShapeMesh::new(&facade, shape)?;

//         let env = Environment::new(&facade)?;

//         Ok(App {
//             facade: facade,
//             control: Box::new(switcher),
//             mesh: mesh,
//             env: env,
//             print_fps: false,
//         })
//     }

//     /// Contains the main loop used to show stuff on the screen.
//     pub fn run(&mut self) -> Result<()> {
//         use glium::Surface;
//         use std::time::{Duration, Instant};
//         use std::io::{self, Write};

//         // Code for printing FPS and frame time
//         const PRINT_FPS_EVERY_MS: u64 = 200;
//         let mut next_fps_print_in = Duration::from_millis(PRINT_FPS_EVERY_MS);
//         let mut frame_count = 0;
//         let mut last_time = Instant::now();

//         loop {
//             // FPS calculations
//             let before_frame = Instant::now();

//             // Approximate time since last iteration and update all components
//             let delta = Instant::now() - last_time;
//             let delta_sec = (delta.subsec_nanos() / 1000) as f32 / 1_000_000.0;

//             self.control.update(delta_sec, self.mesh.shape());
//             self.env.update(delta_sec);
//             self.mesh.update(&self.facade, &self.control.camera())?;

//             last_time = Instant::now();

//             // Clear and start drawing on the default framebuffer
//             let mut target = self.facade.draw();
//             target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

//             self.env.sky().draw(&mut target, &self.control.camera())?;
//             self.env.sun().draw(&mut target, &self.control.camera())?;
//             self.mesh.draw(&mut target, &self.control.camera(), &self.env)?;

//             target.finish().unwrap();

//             // Poll window events
//             let res = self.poll_events();
//             if res == EventResponse::Quit {
//                 info!("Bye! :)");
//                 return Ok(());
//             }

//             // Print FPS
//             if self.print_fps {
//                 frame_count += 1;
//                 let delta = Instant::now() - before_frame;

//                 if delta >= next_fps_print_in {
//                     let over_time = delta - next_fps_print_in;
//                     let since_last = over_time + Duration::from_millis(PRINT_FPS_EVERY_MS);
//                     let since_last = (since_last.subsec_nanos() / 1000) as f32 / 1000.0;

//                     let avg_delta = since_last / (frame_count as f32);

//                     print!("\rδ {:.3}ms ({:.3} FPS)", avg_delta, 1000.0 / avg_delta);
//                     io::stdout().flush().expect("flushing stdout failed...");

//                     // Reset values
//                     frame_count = 0;
//                     next_fps_print_in = Duration::from_millis(PRINT_FPS_EVERY_MS);
//                 } else {
//                     next_fps_print_in -= delta;
//                 }
//             }
//         }
//     }

//     fn poll_events(&mut self) -> EventResponse {
//         use glium::glutin::Event;
//         use glium::glutin::ElementState::*;
//         use glium::glutin::VirtualKeyCode as Vkc;

//         let mut new_res = None;
//         let print_fps = &mut self.print_fps;

//         let out = poll_events_with(&self.facade, &mut [
//             self.control.as_event_handler(),
//             &mut self.mesh,
//             &mut QuitHandler,
//             &mut |e: &Event| {
//                 if let Event::Resized(w, h) = *e {
//                     new_res = Some((w, h));
//                     EventResponse::Continue
//                 } else {
//                     EventResponse::NotHandled
//                 }
//             },
//             &mut |e: &Event| {
//                 if let Event::KeyboardInput(Pressed, _, Some(Vkc::V)) = *e {
//                     *print_fps = !*print_fps;
//                     EventResponse::Break
//                 } else {
//                     EventResponse::NotHandled
//                 }
//             },
//         ]);

//         out
//     }
// }

// /// Creates the OpenGL context and logs useful information about the
// /// success or failure of said action.
// fn create_context() -> Result<GlutinFacade> {
//     use glium::glutin::{self, get_primary_monitor, GlRequest};
//     use glium::DisplayBuild;

//     // Check resolution of monitor
//     let monitor = get_primary_monitor();
//     let (monitor_width, monitor_height) = monitor.get_dimensions();

//     // Create glium context
//     let context = glutin::WindowBuilder::new()
//         .with_dimensions(monitor_width / 2, monitor_height / 2)
//         .with_title(WINDOW_TITLE)
//         .with_gl(GlRequest::Latest)
//         .build_glium()?;

//     // Print some information about the acquired OpenGL context
//     info!("OpenGL context was successfully built");

//     Ok(context)
// }
