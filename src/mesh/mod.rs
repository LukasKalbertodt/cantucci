use cgmath::{prelude::*, Point3, Vector3};
use num_cpus;
use std::{array::IntoIter, sync::mpsc::{channel, Receiver, Sender}};
use std::sync::Arc;
use threadpool::ThreadPool;

use crate::{
    prelude::*,
    camera::Camera,
    octree::{Octree, SpanExt},
    shape::Shape,
    util::iter,
};

mod buffer;
mod view;

use self::buffer::{MeshBuffer, Timings};
use self::view::MeshView;

/// Type to manage the graphical representation of the shape. It updates the
/// internal data depending on the camera position and resolution.
pub struct ShapeMesh {
    /// This octree holds the whole mesh.
    tree: Octree<MeshStatus, ()>,
    pipeline: wgpu::RenderPipeline,

    /// The shape this mesh represents.
    shape: Arc<dyn Shape>,

    // The following fields are simply to manage the generation of the mesh on
    // multiple threads.
    thread_pool: ThreadPool,
    new_meshes: Receiver<(Point3<f32>, (MeshView, Timings))>,
    mesh_tx: Sender<(Point3<f32>, (MeshView, Timings))>,
    active_jobs: u64,

    // These are just for debugging/time measuring purposes
    batch_timings: Timings,
    finished_jobs: u64,
}

impl ShapeMesh {
    pub fn new(
        device: &wgpu::Device,
        out_format: wgpu::TextureFormat,
        shape: Arc<dyn Shape>,
    ) -> Result<Self> {
        // Setup an empty tree and split the first two levels which results in
        // 8² = 64 children
        let mut tree = Octree::spanning(shape.bounding_box());
        let _ = tree.root_mut().split(None);
        for mut child in IntoIter::new(tree.root_mut().into_children().unwrap()) {
            child.split(None);
        }

        // Prepare channels and thread pool to generate the mesh on all CPU
        // cores
        let (tx, rx) = channel();
        let num_threads = num_cpus::get();
        let pool = ThreadPool::new(num_threads);
        info!("Using {} threads to generate mesh", num_threads);

        let pipeline = view::create_pipeline(device, out_format);

        Ok(ShapeMesh {
            tree,
            pipeline,
            shape,
            thread_pool: pool,
            new_meshes: rx,
            mesh_tx: tx,
            active_jobs: 0,
            batch_timings: Timings::default(),
            finished_jobs: 0,
        })
    }

    /// Updates the mesh representing the shape. It increases resolution dynamically when
    /// camera is close to the objects surface.
    pub fn update(&mut self, device: Arc<wgpu::Device>, camera: &Camera) {
        /// Constant that corresponds to the amount of focus points used to determine which octree
        /// nodes are to be split (drawn in higher resolution). In the end FOCUS_POINTS² points
        /// are distributed over the near plane.
        const FOCUS_POINTS: u8 = 5;

        // Get focus points on the near plane. Through these points, distances from the camera to
        // nodes of the octree are calculated. When the distance is under a certain threshold, that
        // particular node is redrawn with higher resolution.
        let focii = self.get_focii(camera, FOCUS_POINTS);
        for focus in focii {
            if let Some(mut leaf) = self.tree.leaf_around_mut(focus) {
                if let Some(MeshStatus::Ready(_)) = leaf.leaf_data().unwrap() {
                    let dist = camera.position.distance(focus);
                    let span = leaf.span();
                    let threshold = 2.0 * (span.end.x - span.start.x).abs();
                    // If we are near enough to the surface, increase resolution.
                    if dist < threshold {
                        leaf.split(None);
                    }
                }
            }
        }

        let jobs_before = self.active_jobs;
        let finished_jobs_before = self.finished_jobs;

        // Collect generated meshes and prepare them for rendering.
        for (center, (view, timings)) in self.new_meshes.try_iter() {
            self.active_jobs -= 1;
            self.finished_jobs += 1;
            self.batch_timings = self.batch_timings + timings;

            *self.tree
                .leaf_around_mut(center)
                // we know that `center` is within the bound of the octree
                .unwrap()
                .leaf_data_mut()
                .unwrap() = Some(MeshStatus::Ready(view));
        }


        // TODO: Decide when to split nodes and when to regenerate regions
        // of space (see #9, #8)


        // Here we simply start a mesh generation job for each empty leaf node.
        let empty_leaves = self.tree.iter_mut()
            .filter_map(|n| n.into_leaf())
            .filter(|&(_, ref leaf_data)| leaf_data.is_none());
        for (span, leaf_data) in empty_leaves {
            const RESOLUTION: u32 = 64;

            // Prepare values to be moved into the closure.
            let tx = self.mesh_tx.clone();
            let shape = self.shape.clone();
            let device = device.clone();

            // Generate the raw buffers on another thread.
            self.thread_pool.execute(move || {
                let (buf, timings) = MeshBuffer::generate_for_box(&span, &*shape, RESOLUTION);
                let view = MeshView::new(&device, &buf.vertices, &buf.indices);

                // If the main thread hung up, it's fine: our thread will be
                // killed soon, too.
                let _ = tx.send((span.center(), (view, timings)));
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
            *leaf_data = Some(MeshStatus::Requested { old_view });
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
    }

    // Draws the whole shape by traversing the internal octree.
    pub(crate) fn draw(
        &self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera,
    ) {
        // Visit each node of the tree.
        // TODO: we might want visit the nodes in a different order (see #16)

        let it = self.tree.iter()
            .filter_map(|n| n.leaf_data().map(|data| (data, n.span())));
        for (leaf_data, _span) in it {
            match leaf_data {
                // If there is a view available, render it.
                &MeshStatus::Ready(ref view) |
                &MeshStatus::Requested { old_view: Some(ref view) } => {
                    view.draw(frame, device, queue, camera, &self.pipeline);
                }
                _ => (),
            }
        }
    }

    /// Returns points on the near plane distributed in a grid. These points are given
    /// in world coordinates. The number of points returned is focus_points².
    pub fn get_focii(&self, camera: &Camera, focus_points: u8) -> Vec<Point3<f32>> {
        const EPSILON: f32 = 0.000_001;
        const MAX_ITERS: u64 = 100;

        let (top_left, bottom_right) = camera.near_plane_bb();
        let (frustum_width, frustum_height) = camera.projection.near_plane_dimension();
        let size_horizontal = frustum_width / focus_points as f32;
        let size_vertical = frustum_height / focus_points as f32;
        let center_diff = (bottom_right - top_left) / (2.0 * focus_points as f32);

        let inv_view_trans = camera.inv_view_transform();

        iter::square(focus_points as u32)
            .map(|(x, y)| {
                let center = top_left + Vector3::new(
                    x as f32 * size_horizontal,
                    y as f32 * size_vertical,
                    0.0,
                ) + center_diff;

                Point3::from_homogeneous(
                    inv_view_trans * center.to_homogeneous()
                )
            })
            .filter_map(|p| {
                let mut pos = camera.position;
                let dir = (p - camera.position).normalize();

                for _ in 0..MAX_ITERS {
                    let distance = self.shape.min_distance_from(pos);
                    pos += dir * distance;
                    if distance < EPSILON {
                        return Some(pos);
                    }
                }
                None
            })
            .collect()
    }
}

enum MeshStatus {
    Requested {
        old_view: Option<MeshView>,
    },
    Ready(MeshView),
}


/// Per vertex data in the generated mesh.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    distance_from_surface: f32,
}

// `Vertex` is inhabited, allows any bitpattern, has no padding, all fields are
// `Pod`, and is `repr(C)`.
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
