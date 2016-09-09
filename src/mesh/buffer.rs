use core::math::*;
use core::Shape;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{VertexBuffer, IndexBuffer};
use mesh::octree::Span;
use util::ToArr;

pub struct MeshBuffer {
    raw_vbuf: Vec<Vertex>,
    raw_ibuf: Vec<u32>,
    resolution: u32,
}

impl MeshBuffer {
    pub fn generate_for_box<S: Shape>(
        span: &Span,
        shape: &S,
        resolution: u32,
    ) -> Self {
        assert!(span.start.x < span.end.x);
        assert!(span.start.y < span.end.y);
        assert!(span.start.z < span.end.z);

        debug!("Starting to generate in {:?} @ {} res", span, resolution);

        let mut raw_vbuf = Vec::with_capacity(resolution.pow(3) as usize);

        for x in 0..resolution {
            for y in 0..resolution {
                for z in 0..resolution {
                    // Calculate the corresponding point in world space
                    let v = Vector3::new(x, y, z).cast::<f64>() / (resolution as f64);
                    let p = span.start + (span.end - span.start).mul_element_wise(v);

                    // "nice" coloring
                    let m = (p.to_vec().magnitude() as f32).powf(8.0);
                    if shape.contains(p) {
                        raw_vbuf.push(Vertex {
                            position: p.to_vec().cast::<f32>().to_arr(),
                            color: [m; 3],
                        });
                    }
                }
            }
        }

        // Fill index buffer
        let raw_ibuf = (0..raw_vbuf.len() as u32).collect();

        debug!(
            "Generated {} points in box ({:?}) @ {} res",
            raw_vbuf.len(),
            span,
            resolution,
        );

        MeshBuffer {
            raw_vbuf: raw_vbuf,
            raw_ibuf: raw_ibuf,
            resolution: resolution,
        }
    }

    pub fn resolution(&self) -> u32 {
        self.resolution
    }
}

pub struct MeshView {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u32>,
    raw_buf: MeshBuffer,
}

impl MeshView {
    pub fn from_raw_buf<F: Facade>(buf: MeshBuffer, facade: &F) -> Self {
        let vbuf = VertexBuffer::new(facade, &buf.raw_vbuf).unwrap();

        let ibuf = IndexBuffer::new(
            facade,
            PrimitiveType::Points,
            &buf.raw_ibuf
        ).unwrap();

        MeshView {
            vbuf: vbuf,
            ibuf: ibuf,
            raw_buf: buf,
        }
    }

    pub fn vbuf(&self) -> &VertexBuffer<Vertex> {
        &self.vbuf
    }

    pub fn ibuf(&self) -> &IndexBuffer<u32> {
        &self.ibuf
    }

    pub fn raw_buf(&self) -> &MeshBuffer {
        &self.raw_buf
    }
}


#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);
