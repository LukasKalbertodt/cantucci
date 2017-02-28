use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{self, DepthTest, Surface, DrawParameters};
use glium::{VertexBuffer, IndexBuffer};
use glium::draw_parameters::BackfaceCullingMode;

use camera::Camera;
use env::Environment;
use errors::*;
use mesh::buffer::{MeshBuffer, Vertex};
use util::ToArr;
use super::renderer::Renderer;


pub struct MeshView {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u32>,
    // raw_buf: MeshBuffer,   // TODO: use or delete
}

impl MeshView {
    /// Creates all required non-global resources to draw the mesh stored in
    /// the `MeshBuffer`.
    pub fn from_raw_buf<F: Facade>(buf: MeshBuffer, facade: &F) -> Result<Self> {
        let vbuf = VertexBuffer::new(facade, buf.raw_vbuf())?;
        let ibuf = IndexBuffer::new(facade, PrimitiveType::TrianglesList, buf.raw_ibuf())?;

        Ok(MeshView {
            vbuf: vbuf,
            ibuf: ibuf,
            // raw_buf: buf,   // TODO: use or delete
        })
    }

    // TODO: use or delete
    // pub fn raw_buf(&self) -> &MeshBuffer {
    //     &self.raw_buf
    // }

    /// Draws the mesh.
    pub fn draw<S: Surface>(
        &self,
        surface: &mut S,
        camera: &Camera,
        env: &Environment,
        renderer: &Renderer,
    ) -> Result<()> {
        let uniforms = uniform! {
            view_matrix: camera.view_transform().to_arr(),
            proj_matrix: camera.proj_transform().to_arr(),
            light_dir: env.sun().light_dir().to_arr(),
        };

        // We want to activate the standard depth test.
        let params = DrawParameters {
            depth: glium::Depth {
                write: true,
                test: DepthTest::IfLess,
                .. Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            .. DrawParameters::default()
        };

        surface.draw(&self.vbuf, &self.ibuf, renderer.program(), &uniforms, &params)?;

        Ok(())
    }
}
