use core::math::*;
use core::Shape;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{VertexBuffer, IndexBuffer};
use mesh::octree::Span;
use std::ops::Index;
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

        let grid = GridTable::fill_with(resolution, |x, y, z| {
            let v = Vector3::new(x, y, z).cast::<f64>() / (resolution as f64);
            let p = span.start + (span.end - span.start).mul_element_wise(v);

            shape.contains(p)
        });

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

/// A lookup table for regular 3D grids. Every cell in the grid contains one
/// value.
///
/// The table is structured in a way such that lookup tables for all children
/// in an octree can easily be obtained. All data for one child is saved
/// consecutive in memory, like so:
///
/// |~~~~~~~~~ -x ~~~~~~~~~||~~~~~~~~~ +x ~~~~~~~~~|
/// |~~~ -y ~~~||~~~ +y ~~~||~~~ -y ~~~||~~~ -y ~~~|
/// | -z || +z || -z || +z || -z || +z || -z || +z |
///    0     1     2     3     4     5     6     7
struct GridTable<T> {
    size: u32,
    data: Vec<T>,
}

impl<T> GridTable<T> {
    fn fill_with<F>(size: u32, mut filler: F) -> Self
        where F: FnMut(u32, u32, u32) -> T
    {
        assert!(size.is_power_of_two());
        assert!(size >= 2);

        fn fill<T, F: FnMut(u32, u32, u32) -> T>(
            data: &mut Vec<T>,
            size: u32, (ox, oy, oz): (u32, u32, u32),
            mut filler: &mut F
        ) {
            if size == 1 {
                data.push(filler(ox, oy, oz));
            } else {
                let half = size / 2;
                fill(data, half, (ox       , oy       , oz       ), filler);
                fill(data, half, (ox       , oy       , oz + half), filler);
                fill(data, half, (ox       , oy + half, oz       ), filler);
                fill(data, half, (ox       , oy + half, oz + half), filler);
                fill(data, half, (ox + half, oy       , oz       ), filler);
                fill(data, half, (ox + half, oy       , oz + half), filler);
                fill(data, half, (ox + half, oy + half, oz       ), filler);
                fill(data, half, (ox + half, oy + half, oz + half), filler);
            }
        }

        let mut data = Vec::with_capacity(size.pow(3) as usize);
        fill(&mut data, size, (0, 0, 0), &mut filler);

        GridTable {
            size: size,
            data: data,
        }
    }
}

impl<T> Index<(u32, u32, u32)> for GridTable<T> {
    type Output = T;

    fn index(&self, (mut x, mut y, mut z): (u32, u32, u32)) -> &Self::Output {
        assert!(x < self.size);
        assert!(y < self.size);
        assert!(z < self.size);

        let mut size = self.size;
        let mut idx = 0;

        while size > 1 {
            if x >= size / 2 {
                idx += size.pow(3) / 2;
                x -= size / 2;
            }
            if y >= size / 2 {
                idx += size.pow(3) / 4;
                y -= size / 2;
            }
            if z >= size / 2 {
                idx += size.pow(3) / 8;
                z -= size / 2;
            }
            size /= 2;
        }

        &self.data[idx as usize]
    }
}
