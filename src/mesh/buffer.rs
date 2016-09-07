use core::math::*;
use core::Shape;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{VertexBuffer, IndexBuffer};
use std::ops::Range;

pub struct MeshBuffer {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u32>,
}

impl MeshBuffer {
    pub fn generate_for_cube<F, S>(facade: &F, cube: Range<Point3<f64>>, shape: &S) -> Self
        where F: Facade,
              S: Shape
    {
        assert!(cube.start.x < cube.end.x);
        assert!(cube.start.y < cube.end.y);
        assert!(cube.start.z < cube.end.z);

        let mut raw_vbuf = Vec::new();
        const RES: i32 = 50;
        for x in -RES..RES {
            for y in -RES..RES {
                for z in -RES..RES {
                    let p = Point3::new(
                        (x as f64) / (RES as f64),
                        (y as f64) / (RES as f64),
                        (z as f64) / (RES as f64)
                    );
                    let m = (p.to_vec().magnitude() as f32).powf(8.0);
                    if shape.contains(p) {
                        raw_vbuf.push(Vertex {
                            position: [p.x as f32, p.y as f32, p.z as f32],
                            color: [m, m, m],
                        });
                    }
                }
            }
        }



        let vbuf = VertexBuffer::new(facade, &raw_vbuf).unwrap();
        debug!("{:?} points", vbuf.len());

        // Create and fill index buffer
        // let raw_ibuf = [0, 1, 2, 3, 4, 5];
        let raw_ibuf: Vec<_> = (0..raw_vbuf.len() as u32).collect();
        let ibuf = IndexBuffer::new(
            facade,
            // PrimitiveType::TrianglesList,
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
