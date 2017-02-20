use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{DrawParameters, IndexBuffer, Program, Surface, VertexBuffer};

use camera::Camera;
use core::math::*;
use errors::*;
use super::SKY_DISTANCE;
use util::{self, ToArr};


/// Represents a simple sky dome.
pub struct Sky {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u8>,
    program: Program,
}

impl Sky {
    /// Creates all resources necessary to draw a sky.
    pub fn new<F: Facade>(facade: &F) -> Result<Self> {
        // We represent the sky by a diamond shaped mesh.
        let raw_vbuf = [
            Vertex { pos: [-SKY_DISTANCE,           0.0,           0.0] },  // -x
            Vertex { pos: [          0.0, -SKY_DISTANCE,           0.0] },  // -y
            Vertex { pos: [ SKY_DISTANCE,           0.0,           0.0] },  // +x
            Vertex { pos: [          0.0,  SKY_DISTANCE,           0.0] },  // +y
            Vertex { pos: [          0.0,           0.0, -SKY_DISTANCE] },  // -z
            Vertex { pos: [          0.0,           0.0,  SKY_DISTANCE] },  // +z
        ];

        let raw_ibuf = [
            // Top triangles
            0, 1, 5,  // -x -y +z
            1, 2, 5,  // +x -y +z
            2, 3, 5,  // +x +y +z
            3, 0, 5,  // -x +y +z

            // Bottom triangles
            0, 1, 4,  // -x -y -z
            1, 2, 4,  // +x -y -z
            2, 3, 4,  // +x +y -z
            3, 0, 4,  // -x +y -z
        ];

        let vbuf = VertexBuffer::new(facade, &raw_vbuf)?;
        let ibuf = IndexBuffer::new(
            facade,
            PrimitiveType::TrianglesList,
            &raw_ibuf,
        )?;
        let program = util::gl::load_program(facade, "sky")?;

        Ok(Sky {
            vbuf: vbuf,
            ibuf: ibuf,
            program: program,
        })
    }

    /// Draws the sky.
    ///
    /// Currently, no depth test is active. Just draw the sky first, everything
    /// else will overdraw.
    pub fn draw<S: Surface>(&self, surface: &mut S, camera: &Camera) -> Result<()> {
        // We discard the translation transformation from the view matrix. This
        // results in a "fixed" sky that moves with the camera.
        let mut view_transform = camera.view_transform();
        view_transform.w = Vector4::new(0.0, 0.0, 0.0, view_transform.w.w);

        let uniforms = uniform! {
            view_matrix: view_transform.to_arr(),
            proj_matrix: camera.proj_transform().to_arr(),
        };

        surface.draw(
            &self.vbuf,
            &self.ibuf,
            &self.program,
            &uniforms,
            &DrawParameters::default(),
        )?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
}

implement_vertex!(Vertex, pos);
