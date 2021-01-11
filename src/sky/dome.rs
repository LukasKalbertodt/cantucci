use std::mem;

use wgpu::util::DeviceExt;

use crate::prelude::*;
use super::SKY_DISTANCE;


/// Represents a simple sky dome.
pub struct Dome {
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Dome {
    /// Creates all resources necessary to draw a sky dome.
    pub fn new(device: &wgpu::Device, out_format: wgpu::TextureFormat) -> Result<Self> {
        // Create vertex and index buffer
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dome vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let vs_module = device.create_shader_module(include_shader!("dome.vert"));
        let fs_module = device.create_shader_module(include_shader!("dome.frag"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Dome render pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: out_format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float3,
                            offset: 0,
                            shader_location: 0,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });


        Ok(Dome { vbuf, ibuf, pipeline })
    }

    /// Draws the sky dome.
    ///
    /// Currently, no depth test is active. Just draw the sky first, everything
    /// else will overdraw.
    pub fn draw(
        &self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // // We discard the translation transformation from the view matrix. This
        // // results in a "fixed" sky that moves with the camera.
        // let mut view_transform = camera.view_transform();
        // view_transform.w = Vector4::new(0.0, 0.0, 0.0, view_transform.w.w);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            // rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.ibuf.slice(..));
            rpass.set_vertex_buffer(0, self.vbuf.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
}

// `Vertex` is inhabited, allows any bitpattern, has no padding, all fields are
// `Pod`, and is `repr(C)`.
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

// We represent the sky by a diamond shaped mesh. The shaders will interpret the
// data they get accordingly to make it look like a perfect sky sphere.
const VERTICES: &[Vertex] = &[
    Vertex { pos: [-SKY_DISTANCE,           0.0,           0.0] },  // -x
    Vertex { pos: [          0.0, -SKY_DISTANCE,           0.0] },  // -y
    Vertex { pos: [ SKY_DISTANCE,           0.0,           0.0] },  // +x
    Vertex { pos: [          0.0,  SKY_DISTANCE,           0.0] },  // +y
    Vertex { pos: [          0.0,           0.0, -SKY_DISTANCE] },  // -z
    Vertex { pos: [          0.0,           0.0,  SKY_DISTANCE] },  // +z
];

const INDICES: &[u16] = &[
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
