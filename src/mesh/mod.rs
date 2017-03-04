use glium::backend::Facade;
use glium::Surface;
use glium::glutin::{Event, MouseButton, ElementState, VirtualKeyCode};
use num_cpus;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use threadpool::ThreadPool;

use camera::Camera;
use core::math::*;
use core::Shape;
use env::Environment;
use errors::*;
use octree::{DebugView, Octree, SpanExt};
use event::{EventHandler, EventResponse};

pub mod buffer;
mod renderer;
mod view;

use self::buffer::{MeshBuffer, Timings};
use self::view::MeshView;
use self::renderer::Renderer;

/// Type to manage the graphical representation of the shape. It updates the
/// internal data depending on the camera position and resolution.
pub struct ShapeMesh {
    /// This octree holds the whole mesh.
    tree: Octree<MeshStatus, ()>,

    /// Holds global data (like the OpenGL program) to render the mesh.
    renderer: Renderer,

    /// The shape this mesh represents.
    shape: Arc<Shape>,

    /// Show the borders of the octree
    debug_octree: DebugView,

    // The following fields are simply to manage the generation of the mesh on
    // multiple threads.
    thread_pool: ThreadPool,
    new_meshes: Receiver<(Point3<f32>, (MeshBuffer, Timings))>,
    mesh_tx: Sender<(Point3<f32>, (MeshBuffer, Timings))>,
    active_jobs: u64,
    split_next_time: bool,
    show_debug: bool,

    // These are just for debugging/time measuring purposes
    batch_timings: Timings,
    finished_jobs: u64,
}

impl ShapeMesh {
    pub fn new<F: Facade>(facade: &F, shape: Arc<Shape>) -> Result<Self> {
        // Setup an empty tree and split the first two levels which results in
        // 8² = 64 children
        let mut tree = Octree::spanning(
            // TODO: let the shape tell us the bounding box
            Point3::new(-1.2, -1.2, -1.2) .. Point3::new(1.2, 1.2, 1.2)
        );
        let _ = tree.root_mut().split(None);
        for mut child in tree.root_mut().into_children().unwrap() {
            child.split(None);
        }

        // Prepare channels and thread pool to generate the mesh on all CPU
        // cores
        let (tx, rx) = channel();
        let num_threads = num_cpus::get();
        let pool = ThreadPool::new(num_threads);
        info!("Using {} threads to generate mesh", num_threads);

        let renderer = Renderer::new(facade, &*shape)?;
        let debug_octree = DebugView::new(facade)?;

        Ok(ShapeMesh {
            tree: tree,
            renderer: renderer,
            shape: shape,
            debug_octree: debug_octree,
            thread_pool: pool,
            new_meshes: rx,
            mesh_tx: tx,
            active_jobs: 0,
            split_next_time: false,
            show_debug: true,
            batch_timings: Timings::default(),
            finished_jobs: 0,
        })
    }

    pub fn shape(&self) -> &Shape {
        &*self.shape
    }

    /// Updates the mesh representing the shape.
    pub fn update<F: Facade>(&mut self, facade: &F, camarero: &Camera) -> Result<()> {
        if self.split_next_time {
            self.split_next_time = false;
            let node = self.get_focus(camarero)
                .and_then(|focus| self.tree.leaf_around_mut(focus));
            if let Some(mut node) = node {
                node.split(None);
            }
        }
        let jobs_before = self.active_jobs;

        // Collect generated meshes and prepare them for rendering.
        for (center, (buf, timings)) in self.new_meshes.try_iter() {
            self.active_jobs -= 1;
            self.finished_jobs += 1;
            self.batch_timings = self.batch_timings + timings;

            // Create OpenGL view from raw buffer and save it in the
            // tree.
            // We haven't yet measured how expensive this is. If it gets too
            // slow, we might want to limit the number of `from_raw_buf()`
            // calls we can do in one `update()` call in order to avoid
            // too long delays within frames.
            let mesh_view = MeshView::from_raw_buf(buf, facade)?;
            *self.tree
                .leaf_around_mut(center)
                // we know that `center` is within the bound of the octree
                .unwrap()
                .leaf_data_mut()
                .unwrap() = Some(MeshStatus::Ready(mesh_view));
        }


        // TODO: Decide when to split nodes and when to regenerate regions
        // of space (see #9, #8)


        // Here we simply start a mesh generation job for each empty leaf node
        let empty_leaves = self.tree.iter_mut()
            .filter_map(|n| n.into_leaf())
            .filter(|&(_, ref leaf_data)| leaf_data.is_none());
        for (span, leaf_data) in empty_leaves {
            const RESOLUTION: u32 = 64;

            // Prepare values to be moved into the closure
            let tx = self.mesh_tx.clone();
            let shape = self.shape.clone();

            // Generate the raw buffers on another thread
            self.thread_pool.execute(move || {
                let buf = MeshBuffer::generate_for_box(&span, &*shape, RESOLUTION);
                tx.send((span.center(), buf))
                    .expect("main thread has hung up, my work was for nothing! :-(");
            });

            self.active_jobs += 1;

            // If there has been an old view, we want to preserve it and
            // continue to render it until the new one is available. This
            // doesn't make a lot of sense right now, but might be helpful
            // later. Or it might not.
            let old_view = match leaf_data.take() {
                Some(MeshStatus::Ready(view)) => Some(view),
                _ => None,
            };
            *leaf_data = Some(MeshStatus::Requested {
                old_view: old_view,
            });
        }

        if jobs_before != self.active_jobs {
            trace!("Currently active sample jobs: {}", self.active_jobs);

            const PRINT_EVERY_FINISHED_JOBS: u64 = 64;
            if self.finished_jobs % PRINT_EVERY_FINISHED_JOBS == 0 && self.finished_jobs > 0 {
                debug!(
                    "Finished {} new jobs in: {}",
                    PRINT_EVERY_FINISHED_JOBS,
                    self.batch_timings,
                );
                // TODO: reset timings
                self.batch_timings = Timings::default();
            }
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

        let focus_point = self.get_focus(camera);

        let it = self.tree.iter()
            .filter_map(|n| n.leaf_data().map(|data| (data, n.span())));
        for (leaf_data, span) in it {
            match leaf_data {
                // If there is a view available, render it
                &MeshStatus::Ready(ref view) |
                &MeshStatus::Requested { old_view: Some(ref view) } => {
                    view.draw(surface, camera, env, &self.renderer)?;
                    if self.show_debug {
                        let highlight = focus_point
                            .map(|fp| span.contains(fp))
                            .unwrap_or(false);
                        self.debug_octree.draw(surface, camera, span, highlight)?;
                    }
                }
                _ => (),
            }
        }


        Ok(())
    }

    /// Returns the point on the shape's surface the camera is currently looking at
    pub fn get_focus(&self, camera: &Camera) -> Option<Point3<f32>> {
        const EPSILON: f32 = 0.000_001;
        const MAX_ITERS: u64 = 100;

        let mut pos = camera.position;
        for _ in 0..MAX_ITERS {
            let distance = self.shape.min_distance_from(pos);
            pos += camera.direction() * distance;
            if distance < EPSILON {
                break;
            }
        }
        Some(pos)
    }
}

impl EventHandler for ShapeMesh {

    fn handle_event(&mut self, e: &Event) -> EventResponse {
        match *e {
            Event::MouseInput(ElementState::Released, MouseButton::Left) => {
                self.split_next_time = true;
                EventResponse::Continue
            },
            Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::G)) => {
                self.show_debug = !self.show_debug;
                EventResponse::Continue
            },
            _ => { EventResponse::NotHandled }
        }
    }

}

enum MeshStatus {
    Requested {
        old_view: Option<MeshView>,
    },
    Ready(MeshView),
}
