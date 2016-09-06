use glium::backend::Facade;
use glium::{VertexBuffer, IndexBuffer};
use std::ops::Range;
use glium::index::PrimitiveType;
use core::math::*;

pub struct MeshBuffer {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u32>,
}

impl MeshBuffer {
    pub fn generate_for_cube<F: Facade>(facade: &F, cube: Range<Point3<f64>>)
        -> Self
    {
        assert!(cube.start.x < cube.end.x);
        assert!(cube.start.y < cube.end.y);
        assert!(cube.start.z < cube.end.z);


        fn is_in_set(p: Point3<f64>) -> bool {
            let mut z = p;

            const MAX_ITERS: u32 = 10;
            const BAILOUT: f64 = 2.5;
            const POWER: f64 = 8.0;

            for _ in 0..MAX_ITERS {

                let r = z.to_vec().magnitude();
                if r > BAILOUT {
                    return false;
                }

                // convert to polar coordinates
                let theta = (z.z / r).acos();
                let phi = f64::atan2(z.y, z.x);

                // scale and rotate the point
                let zr = r.powf(POWER);
                let theta = theta * POWER;
                let phi = phi * POWER;

                // convert back to cartesian coordinates
                z = zr * Point3::new(
                    theta.sin() * phi.cos(),
                    phi.sin() * theta.sin(),
                    theta.cos()
                );
                z += p.to_vec();
            }

            true
        }

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
                    if is_in_set(p) {
                        raw_vbuf.push(Vertex {
                            position: [p.x as f32, p.y as f32, p.z as f32],
                            color: [m, m, m],
                        });
                    }
                }
            }
        }



        let vbuf = VertexBuffer::new(facade, &raw_vbuf).unwrap();
        println!("{:?}", vbuf.len());

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
