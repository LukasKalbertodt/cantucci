use camera::Camera;
use core::math::*;
use core::Shape;
use errors::*;
use glium::backend::Facade;
use glium::{self, DepthTest, Program, Surface, DrawParameters};
use num_cpus;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use threadpool::ThreadPool;
use util::ToArr;

mod octree;
mod buffer;

use self::octree::{Octree, SpanExt};
use self::buffer::{MeshBuffer, MeshView};

/// Type to manage the graphical representation of the fractal. It updates the
/// internal data depending on the camera position and resolution.
pub struct FractalMesh<Sh> {
    tree: Octree<MeshStatus>,
    program: Program,
    shape: Sh,
    thread_pool: ThreadPool,
    new_meshes: Receiver<(Point3<f64>, MeshBuffer)>,
    mesh_tx: Sender<(Point3<f64>, MeshBuffer)>,
    active_jobs: u64,
}

impl<Sh: Shape + 'static> FractalMesh<Sh> {
    pub fn new<F: Facade>(facade: &F, shape: Sh) -> Result<Self> {
        use util::gl::load_program;

        // Setup empty tree and split first level to have 8 children
        let mut tree = Octree::spanning(
            Point3::new(-1.0, -1.0, -1.0) .. Point3::new(1.0, 1.0, 1.0)
        );
        let _ = tree.root_mut().split();

        // Prepare thread pool and channels to communicate
        let (tx, rx) = channel();
        let num_threads = num_cpus::get();
        let pool = ThreadPool::new(num_threads);
        info!("Using {} threads to generate fractal", num_threads);

        // Load Shader program
        let prog = try!(
            load_program(facade, "point-cloud-mandelbulb")
                .chain_err(|| "loading program for fractal mesh failed")
        );

        Ok(FractalMesh {
            tree: tree,
            program: prog,
            shape: shape,
            thread_pool: pool,
            new_meshes: rx,
            mesh_tx: tx,
            active_jobs: 0,
        })
    }

    pub fn update<F: Facade>(&mut self, facade: &F, cam: &Camera) {
        let jobs_before = self.active_jobs;

        // Collect generated meshes and prepare them for rendering
        loop {
            // TODO: we might want to measure the time, to avoid blocking
            // in this main thread for too long

            // If the channel can yield one item
            match self.new_meshes.try_recv() {
                Ok((center, buf)) => {
                    self.active_jobs -= 1;

                    // Create OpenGL view from raw buffer and save it in the
                    // tree
                    let mesh_view = MeshView::from_raw_buf(buf, facade);
                    *self.tree
                        .leaf_mut_around(center)
                        .leaf_data()
                        .unwrap() = Some(MeshStatus::Ready(mesh_view));
                }
                Err(TryRecvError::Empty) => {
                    // That's fine, we check again next time
                    break
                }
                Err(TryRecvError::Disconnected) => {
                    // It's stupid to say that, but this can't happen...
                    // This only happens if all `Sender`s were destroyed and
                    // since we have one `Sender` saved in this struct: when
                    // this cases occurs, something is *way* kaputt!
                    panic!("All senders have hung up...");
                }
            }
        }

        // Check camera position, calculate required precision and start new
        // tasks if necessary

        #[derive(Clone, Copy, Debug)]
        struct ResolutionQuery {
            min: u32,
            desired: u32,
            max: u32,
        }

        fn desired_resolution(p: Point3<f64>, eye: Point3<f64>) -> ResolutionQuery {
            const PRECISION_MUTIPLIER: f64 = 70.0;
            const MAX_RES: f64 = 100.0;

            let desired = 1.0/(p - eye).magnitude() * PRECISION_MUTIPLIER;
            let desired = clamp(desired, 0.0, MAX_RES);

            ResolutionQuery {
                min: (desired / 2.0) as u32,
                desired: desired as u32,
                max: (desired * 4.0) as u32,
            }
        }

        // TODO: iterate over all leaves
        for mut leaf in self.tree.root_mut().into_children().unwrap() {
            let desired_res = desired_resolution(leaf.span().center(), cam.position);

            // Decide whether or not to generate a new buffer for this leaf
            let generate = match *leaf.leaf_data().unwrap() {
                Some(MeshStatus::Ready(ref view)) => {
                    let leaf_res = view.raw_buf().resolution();
                    leaf_res < desired_res.min || leaf_res > desired_res.max
                }
                Some(MeshStatus::Requested { .. }) => false,
                None => desired_res.desired > 0,
            };

            if generate {
                // prepare values to be moved into the closure
                let tx = self.mesh_tx.clone();
                let span = leaf.span();
                let shape = self.shape.clone();

                // Generate the raw buffers on another thread
                self.thread_pool.execute(move || {
                    let buf = MeshBuffer::generate_for_box(&span, &shape, desired_res.desired);
                    let res = tx.send((span.center(), buf));

                    if res.is_err() {
                        debug!("main thread has hung up, my work was for nothing! :-(");
                    }
                });

                self.active_jobs += 1;

                let old_view = match leaf.leaf_data().unwrap().take() {
                    Some(MeshStatus::Ready(view)) => Some(view),
                    _ => None,
                };
                *leaf.leaf_data().unwrap() = Some(MeshStatus::Requested {
                    old_view: old_view,
                });
            }
        }

        if jobs_before != self.active_jobs {
            trace!("Currently active sample jobs: {}", self.active_jobs);
        }
    }

    pub fn draw<S: Surface>(&mut self, surface: &mut S, camera: &Camera) {
        let uniforms = uniform! {
            view_matrix: camera.view_transform().to_arr(),
            proj_matrix: camera.proj_transform().to_arr(),
        };

        let params = DrawParameters {
            point_size: Some(2.0),
            depth: glium::Depth {
                write: true,
                test: DepthTest::IfLess,
                .. Default::default()
            },
            // backface_culling: ::glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            .. DrawParameters::default()
        };

        for entry in &self.tree {
            match entry.leaf_data() {
                Some(&MeshStatus::Ready(ref view)) |
                Some(&MeshStatus::Requested { old_view: Some(ref view) }) => {
                    surface.draw(
                        view.vbuf(),
                        view.ibuf(),
                        &self.program,
                        &uniforms,
                        &params,
                    ).expect("drawing on surface failed!");
                }
                _ => (),
            }
        }
    }
}

enum MeshStatus {
    Requested {
        old_view: Option<MeshView>,
    },
    Ready(MeshView),
}
