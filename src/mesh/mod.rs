use glium::backend::Facade;
use glium::Surface;
use num_cpus;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use threadpool::ThreadPool;

use camera::Camera;
use core::math::*;
use core::Shape;
use env::Environment;
use errors::*;

mod buffer;
mod octree;
mod renderer;
mod view;

use self::octree::{Octree, SpanExt};
use self::buffer::MeshBuffer;
use self::view::MeshView;
use self::renderer::Renderer;

/// Type to manage the graphical representation of the shape. It updates the
/// internal data depending on the camera position and resolution.
pub struct FractalMesh<Sh> {
    /// This octree holds the whole mesh.
    tree: Octree<MeshStatus>,

    /// Holds global data (like the OpenGL program) to render the mesh.
    renderer: Renderer,

    /// The shape this mesh represents.
    shape: Sh,

    // The following fields are simply to manage the generation of the mesh on
    // multiple threads.
    thread_pool: ThreadPool,
    new_meshes: Receiver<(Point3<f32>, MeshBuffer)>,
    mesh_tx: Sender<(Point3<f32>, MeshBuffer)>,
    active_jobs: u64,
}

impl<Sh: Shape + Clone> FractalMesh<Sh> {
    pub fn new<F: Facade>(facade: &F, shape: Sh) -> Result<Self> {
        // Setup an empty tree and split the first two levels which results in
        // 8² = 64 children
        let mut tree = Octree::spanning(
            Point3::new(-1.2, -1.2, -1.2) .. Point3::new(1.2, 1.2, 1.2)
        );
        let _ = tree.root_mut().split();
        for mut child in tree.root_mut().into_children().unwrap() {
            child.split();
        }

        // Prepare channels and thread pool to generate the mesh on all CPU
        // cores
        let (tx, rx) = channel();
        let num_threads = num_cpus::get();
        let pool = ThreadPool::new(num_threads);
        info!("Using {} threads to generate mesh", num_threads);

        let renderer = Renderer::new(facade, &shape)?;

        Ok(FractalMesh {
            tree: tree,
            renderer: renderer,
            shape: shape,
            thread_pool: pool,
            new_meshes: rx,
            mesh_tx: tx,
            active_jobs: 0,
        })
    }

    pub fn shape(&self) -> &Sh {
        &self.shape
    }

    pub fn update<F: Facade>(&mut self, facade: &F, cam: &Camera) -> Result<()> {
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
                    let mesh_view = MeshView::from_raw_buf(buf, facade)?;
                    *self.tree
                        .leaf_around_mut(center)
                        .leaf_data_mut()
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

        fn desired_resolution(p: Point3<f32>, eye: Point3<f32>) -> ResolutionQuery {
            const PRECISION_MUTIPLIER: f32 = 150.0;
            const MAX_RES: f32 = 250.0;

            let desired = 1.0/(p - eye).magnitude() * PRECISION_MUTIPLIER;
            let desired = clamp(desired, 0.0, MAX_RES);

            ResolutionQuery {
                // min: (desired / 2.0) as u32,
                // desired: desired as u32,
                // max: (desired * 4.0) as u32,
                min: 0,
                desired: 100,
                max: 1_000_000,
            }
        }

        // TODO: iterate over all leaves
        let leaves = self.tree.root_mut()
            .into_children()
            .unwrap()
            .into_iter()
            .flat_map(|n| n.into_children().unwrap());
        for mut leaf in leaves {
            let desired_res = desired_resolution(leaf.span().center(), cam.position);

            // Decide whether or not to generate a new buffer for this leaf
            let generate = match *leaf.leaf_data_mut().unwrap() {
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
                    // let buf = MeshBuffer::generate_for_box(&span, &shape, 64);
                    let res = tx.send((span.center(), buf));

                    if res.is_err() {
                        debug!("main thread has hung up, my work was for nothing! :-(");
                    }
                });

                self.active_jobs += 1;

                let old_view = match leaf.leaf_data_mut().unwrap().take() {
                    Some(MeshStatus::Ready(view)) => Some(view),
                    _ => None,
                };
                *leaf.leaf_data_mut().unwrap() = Some(MeshStatus::Requested {
                    old_view: old_view,
                });
            }
        }

        if jobs_before != self.active_jobs {
            trace!("Currently active sample jobs: {}", self.active_jobs);
        }

        Ok(())
    }

    // Draws the whole shape by traversing the internal octree.
    pub fn draw<S: Surface>(
        &self,
        surface: &mut S,
        camera: &Camera,
        env: &Environment,
    ) -> Result<()> {
        // Visit each node of the tree
        // TODO: we might want visit the nodes in a different order (see #16)
        for leaf_data in self.tree.iter().filter_map(|e| e.leaf_data()) {
            match leaf_data {
                // If there is a view available, render it
                &MeshStatus::Ready(ref view) |
                &MeshStatus::Requested { old_view: Some(ref view) } => {
                    view.draw(surface, camera, env, &self.renderer)?;
                }
                _ => (),
            }
        }

        Ok(())
    }
}

enum MeshStatus {
    Requested {
        old_view: Option<MeshView>,
    },
    Ready(MeshView),
}
