use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{self, DepthTest, Surface, DrawParameters};
use glium::{VertexBuffer, IndexBuffer, Program};

use camera::Camera;
use errors::*;
use super::Span;
use util::{gl, ToArr};


pub struct DebugView {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u8>,
    prog: Program,
}

impl DebugView {
    pub fn new<F: Facade>(facade: &F) -> Result<Self> {
        let raw_vbuf = [
            Vertex { pos: [0.0, 0.0, 0.0] },
            Vertex { pos: [1.0, 0.0, 0.0] },
            Vertex { pos: [1.0, 1.0, 0.0] },
            Vertex { pos: [0.0, 1.0, 0.0] },
            Vertex { pos: [0.0, 0.0, 1.0] },
            Vertex { pos: [1.0, 0.0, 1.0] },
            Vertex { pos: [1.0, 1.0, 1.0] },
            Vertex { pos: [0.0, 1.0, 1.0] },
        ];
        let raw_ibuf = [
            // bottom
            0, 1,
            1, 2,
            2, 3,
            3, 0,

            // top
            4, 5,
            5, 6,
            6, 7,
            7, 4,

            // sides
            0, 4,
            1, 5,
            2, 6,
            3, 7,
        ];

        let vbuf = VertexBuffer::new(facade, &raw_vbuf)?;
        let ibuf = IndexBuffer::new(facade, PrimitiveType::LinesList, &raw_ibuf)?;
        let prog = gl::load_program(facade, "octree-debug-view")?;


        Ok(DebugView {
            vbuf: vbuf,
            ibuf: ibuf,
            prog: prog,
        })
    }

    pub fn draw<S: Surface>(
        &self,
        surface: &mut S,
        camera: &Camera,
        span: Span,
    ) -> Result<()> {
        let uniforms = uniform! {
            view_matrix: camera.view_transform().to_arr(),
            proj_matrix: camera.proj_transform().to_arr(),
            cube_start: span.start.to_arr(),
            cube_end: span.end.to_arr(),
        };

        // We want to activate the standard depth test.
        let params = DrawParameters {
            depth: glium::Depth {
                write: true,
                test: DepthTest::IfLess,
                .. Default::default()
            },
            .. DrawParameters::default()
        };

        surface.draw(&self.vbuf, &self.ibuf, &self.prog, &uniforms, &params)?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pos: [f32; 3],
}

implement_vertex!(Vertex, pos);
