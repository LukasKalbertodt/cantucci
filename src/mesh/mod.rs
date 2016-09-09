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
    tree: Octree<MeshView>,
    program: Program,
    shape: Sh,
    thread_pool: ThreadPool,
    new_meshes: Receiver<(Point3<f64>, MeshBuffer)>,
    mesh_tx: Sender<(Point3<f64>, MeshBuffer)>,
}

impl<Sh: Shape + 'static> FractalMesh<Sh> {
    pub fn new<F: Facade>(facade: &F, shape: Sh) -> Result<Self> {
        use util::gl::load_program;


        let mut tree = Octree::spanning(
            Point3::new(-1.0, -1.0, -1.0) .. Point3::new(1.0, 1.0, 1.0)
        );


        let (tx, rx) = channel();
        let num_threads = num_cpus::get();
        let pool = ThreadPool::new(num_threads);
        info!("Using {} threads to generate fractal", num_threads);

        let _ = tree.root_mut().split();
        for c in tree.root().children().unwrap() {
            // prepare values to be moved into the closure
            let tx = tx.clone();
            let span = c.span();
            let shape = shape.clone();

            pool.execute(move || {
                let buf = MeshBuffer::generate_for_box(&span, &shape, 80);
                let res = tx.send((span.center(), buf));

                if res.is_err() {
                    debug!("main thread has hung up, my work was for nothing!");
                }
            });
        }

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
        })
    }

    pub fn update<F: Facade>(&mut self, facade: &F) {
        loop {
            match self.new_meshes.try_recv() {
                Ok((center, buf)) => {
                    let mesh_view = MeshView::from_raw_buf(buf, facade);

                    *self.tree
                        .leaf_mut_around(center)
                        .leaf_data()
                        .unwrap() = Some(mesh_view);
                }
                Err(TryRecvError::Empty) => break,
                _ => panic!(),
            }
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
            if let Some(buf) = entry.leaf_data() {
                surface.draw(
                    buf.vbuf(),
                    buf.ibuf(),
                    &self.program,
                    &uniforms,
                    &params,
                ).expect("drawing on surface failed!");
            }
        }
    }
}
