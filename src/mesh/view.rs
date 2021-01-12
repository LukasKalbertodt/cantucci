use wgpu::util::DeviceExt;

use crate::{
    camera::Camera,
    util::ToArr,
};
use super::Vertex;


pub struct MeshView {
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
}

impl MeshView {
    /// Creates all required non-global resources to draw the mesh stored in
    /// the `MeshBuffer`.
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape mesh vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape mesh index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        Self{ vbuf, ibuf }
    }

    // /// Draws the mesh.
    // pub fn draw<S: Surface>(
    //     &self,
    //     surface: &mut S,
    //     camera: &Camera,
    //     env: &Environment,
    //     renderer: &Renderer,
    // ) -> Result<()> {
    //     let uniforms = uniform! {
    //         view_matrix: camera.view_transform().to_arr(),
    //         proj_matrix: camera.proj_transform().to_arr(),
    //         light_dir: env.sun().light_dir().to_arr(),
    //     };

    //     // We want to activate the standard depth test.
    //     let params = DrawParameters {
    //         depth: glium::Depth {
    //             write: true,
    //             test: DepthTest::IfLess,
    //             .. Default::default()
    //         },
    //         backface_culling: BackfaceCullingMode::CullClockwise,
    //         .. DrawParameters::default()
    //     };

    //     surface.draw(&self.vbuf, &self.ibuf, renderer.program(), &uniforms, &params)?;

    //     Ok(())
    // }
}
