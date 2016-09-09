use camera::Camera;
use core::math::*;
use core::Shape;
use errors::*;
use glium::backend::Facade;
use glium::{self, DepthTest, Program, Surface, DrawParameters};
use util::ToArr;

mod octree;
mod buffer;

use self::octree::Octree;
use self::buffer::MeshBuffer;

/// Type to manage the graphical representation of the fractal. It updates the
/// internal data depending on the camera position and resolution.
pub struct FractalMesh<Sh> {
    buffer: Octree<MeshBuffer>,
    program: Program,
    shape: Sh,
}

impl<Sh: Shape> FractalMesh<Sh> {
    pub fn new<F: Facade>(facade: &F, shape: Sh) -> Result<Self> {
        use util::gl::load_program;

        let buf = MeshBuffer::generate_for_box(
            facade,
            Point3::new(-1.0, -1.0, -1.0) .. Point3::new(1.0, 1.0, 1.0),
            &shape,
            100,
        );
        let mut tree = Octree::spanning(
            Point3::new(-1.0, -1.0, -1.0) .. Point3::new(1.0, 1.0, 1.0)
        );

        *tree.iter_mut().next().unwrap() = Some(buf);

        let prog = try!(
            load_program(facade, "point-cloud-mandelbulb")
                .chain_err(|| "loading program for fractal mesh failed")
        );

        Ok(FractalMesh {
            buffer: tree,
            program: prog,
            shape: shape,
        })
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

        for entry in self.buffer.iter() {
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
