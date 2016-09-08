use core::math::*;
use core::Shape;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{VertexBuffer, IndexBuffer};
use std::ops::Range;
use util::ToArr;

pub struct MeshBuffer {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u32>,
}

impl MeshBuffer {
    pub fn generate_for_box<F, S>(
        facade: &F,
        boxr: Range<Point3<f64>>,
        shape: &S,
        resolution: u32
    ) -> Self
        where F: Facade,
              S: Shape
    {
        assert!(boxr.start.x < boxr.end.x);
        assert!(boxr.start.y < boxr.end.y);
        assert!(boxr.start.z < boxr.end.z);

        let mut raw_vbuf = Vec::with_capacity(resolution.pow(3) as usize);

        for x in 0..resolution {
            for y in 0..resolution {
                for z in 0..resolution {
                    // Calculate the corresponding point in world space
                    let v = Vector3::new(x, y, z).cast::<f64>() / (resolution as f64);
                    let p = boxr.start + (boxr.end - boxr.start).mul_element_wise(v);

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



        let vbuf = VertexBuffer::new(facade, &raw_vbuf).unwrap();
        debug!("Generated {} points in box ({:?})", vbuf.len(), boxr);

        // Create and fill index buffer
        let raw_ibuf: Vec<_> = (0..raw_vbuf.len() as u32).collect();
        let ibuf = IndexBuffer::new(
            facade,
            PrimitiveType::Points,
            &raw_ibuf
        ).unwrap();

        MeshBuffer {
            vbuf: vbuf,
            ibuf: ibuf,
        }
    }

    pub fn vbuf(&self) -> &VertexBuffer<Vertex> {
        &self.vbuf
    }

    pub fn ibuf(&self) -> &IndexBuffer<u32> {
        &self.ibuf
    }
}


#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);
