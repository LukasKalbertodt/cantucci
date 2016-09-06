mod octree;
mod buffer;

use glium::{self, DepthTest, Program, Surface, DrawParameters};
use glium::backend::Facade;
use camera::Camera;
use util::ToArr;
use core::math::*;
use self::octree::Octree;
use self::buffer::MeshBuffer;

/// Type to manage the graphical representation of the fractal. It updates the
/// internal data depending on the camera position and resolution.
pub struct FractalMesh {
    buffer: Octree<MeshBuffer>,
    program: Program,
}

impl FractalMesh {
    pub fn new<F: Facade>(facade: &F) -> Self {
        let buf = MeshBuffer::generate_for_cube(
            facade,
            Point3::new(-1.0, -1.0, -1.0) .. Point3::new(1.0, 1.0, 1.0),
        );
        let tree = Octree::Leaf(buf);


        // Create program
        let vertex_shader_src = r#"
            #version 400
            uniform dmat4 view_matrix;
            uniform dmat4 proj_matrix;

            out float z;
            out vec3 ocolor;

            in vec3 position;
            in vec3 color;
            void main() {
                z = position.z;
                ocolor = color;

                gl_Position = vec4(
                    proj_matrix *
                    view_matrix *
                    vec4(position, 1.0)
                );
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            out vec4 color;
            in vec3 ocolor;
            in float z;
            void main() {
                color = vec4(ocolor, 1.0);
            }
        "#;

        let program = Program::from_source(
            facade,
            vertex_shader_src,
            fragment_shader_src,
            None
        ).unwrap();


        FractalMesh {
            buffer: tree,
            program: program,
        }
    }

    pub fn draw<S: Surface>(&self, surface: &mut S, camera: &Camera) {
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

        match self.buffer {
            Octree::Leaf(ref buf) => {
                surface.draw(
                    buf.vbuf(),
                    buf.ibuf(),
                    &self.program,
                    &uniforms,
                    &params,
                ).expect("drawing on surface failed!");
            }
            _ => unimplemented!(),
        }
    }
}
