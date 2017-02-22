use glium::backend::Facade;
use glium::Surface;
use num_cpus;
use std::sync::mpsc::{channel, Receiver, Sender};
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

    /// Updates the mesh representing the shape.
    pub fn update<F: Facade>(&mut self, facade: &F, _: &Camera) -> Result<()> {
        let jobs_before = self.active_jobs;

        // Collect generated meshes and prepare them for rendering.
        for (center, buf) in self.new_meshes.try_iter() {
            self.active_jobs -= 1;

            // Create OpenGL view from raw buffer and save it in the
            // tree.
            // We haven't yet measured how expensive this is. If it gets too
            // slow, we might want to limit the number of `from_raw_buf()`
            // calls we can do in one `update()` call in order to avoid
            // too high frame times.
            let mesh_view = MeshView::from_raw_buf(buf, facade)?;
            *self.tree
                .leaf_around_mut(center)
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
            const RESOLUTION: u32 = 100;

            // Prepare values to be moved into the closure
            let tx = self.mesh_tx.clone();
            let shape = self.shape.clone();

            // Generate the raw buffers on another thread
            self.thread_pool.execute(move || {
                let buf = MeshBuffer::generate_for_box(&span, &shape, RESOLUTION);
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
