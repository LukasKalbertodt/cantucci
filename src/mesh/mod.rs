use glium::backend::Facade;
use glium::Surface;
use glium::glutin::{Event, ElementState, VirtualKeyCode};
use num_cpus;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use threadpool::ThreadPool;

use camera::Camera;
use math::*;
use shape::Shape;
use env::Environment;
use errors::*;
use octree::{DebugView, Octree, SpanExt};
use event::{EventHandler, EventResponse};

mod buffer;
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
    show_debug: bool,

    // These are just for debugging/time measuring purposes
    batch_timings: Timings,
    finished_jobs: u64,
}

impl ShapeMesh {
    pub fn new<F: Facade>(facade: &F, shape: Arc<Shape>) -> Result<Self> {
        // Setup an empty tree and split the first two levels which results in
        // 8² = 64 children
        let mut tree = Octree::spanning(shape.bounding_box());
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
            show_debug: true,
            batch_timings: Timings::default(),
            finished_jobs: 0,
        })
    }

    pub fn shape(&self) -> &Shape {
        &*self.shape
    }

    /// Updates the mesh representing the shape.
    pub fn update<F: Facade>(&mut self, facade: &F, camera: &Camera) -> Result<()> {
        // increase resolution dynamically if there is a cube responding to the focii
        // determines how many focus points are used to determine which nodes are to be split
        const FOCUS_POINTS: u8 = 5;
        let focii = self.get_focii(camera, FOCUS_POINTS);
        for focus in focii {
            if let Some(mut leaf) = self.tree.leaf_around_mut(focus) {
                let do_split = if let &Some(MeshStatus::Ready(_)) = leaf.leaf_data().unwrap() {
                    true
                } else {
                    false
                };
                if do_split {
                    let dist = camera.position.distance(focus);
                    let span = leaf.span();
                    let threshold = 2.0 * (span.end.x - span.start.x).abs();
                    // if we are near enough to the surface, increase resolution
                    if dist < threshold {
                        leaf.split(None);
                    }
                }
            }
        }

        let jobs_before = self.active_jobs;
        let finished_jobs_before = self.finished_jobs;

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
        }

        const PRINT_EVERY_FINISHED_JOBS: u64 = 64;
        if self.finished_jobs % PRINT_EVERY_FINISHED_JOBS == 0
            && self.finished_jobs > 0
            && finished_jobs_before != self.finished_jobs {
            debug!(
                "Finished {} new jobs in: {}",
                PRINT_EVERY_FINISHED_JOBS,
                self.batch_timings,
            );
            self.batch_timings = Timings::default();
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

    /// Returns equally distributed points on the near_plane. The number of points
    /// returned is focus_points².
    pub fn get_focii(&self, camera: &Camera, focus_points: u8) -> Vec<Point3<f32>> {
        // sets the amount of focus points that may cause a split event
        const EPSILON: f32 = 0.000_001;
        const MAX_ITERS: u64 = 100;

        let bb = camera.get_near_plane_bb();
        let frustum_width = bb.1.x - bb.0.x;
        let frustum_height = bb.1.y - bb.0.y;
        let size_horizontal = frustum_width/focus_points as f32;
        let size_vertical = frustum_height/focus_points as f32;
        let center_diff = (bb.1 - bb.0)/(2.0*focus_points as f32);

        let inv_view_trans = camera.view_transform().invert().unwrap();

        let mut vec = Vec::new();
        for y in 0..focus_points {
            for x in 0..focus_points {
                let center = bb.0 + Vector3::new(
                    x as f32 * size_horizontal,
                    y as f32 * size_vertical,
                    0.0,
                ) + center_diff;

                vec.push(
                    Point3::from_homogeneous(
                        inv_view_trans * center.to_homogeneous()
                    )
                );
            }
        }

        let mut ret = Vec::new();
        for point in vec {
            let mut pos = camera.position;
            let dir = (point - camera.position).normalize();

            for _ in 0..MAX_ITERS {
                let distance = self.shape.min_distance_from(pos);
                pos += dir * distance;
                if distance < EPSILON {
                    ret.push(pos);
                    break;
                }
            }
        }
        ret
    }

    // Wrapper method for just getting the focus along the regular camera direction vector
    pub fn get_focus(&self, camera: &Camera) -> Option<Point3<f32>> {
        for point in Self::get_focii(self, camera, 1) {
            return Some(point);
        }
        None
    }
}

impl EventHandler for ShapeMesh {

    fn handle_event(&mut self, e: &Event) -> EventResponse {
        match *e {
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
