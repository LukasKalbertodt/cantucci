use core::math::*;
use core::Shape;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{VertexBuffer, IndexBuffer};
use mesh::octree::Span;
use std::ops::Index;
use util::ToArr;
use util::iter::cube;

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

        let grid = GridTable::fill_with(resolution + 1, |x, y, z| {
            let v = Vector3::new(x, y, z).cast::<f64>() / (resolution as f64);
            let p = span.start + (span.end - span.start).mul_element_wise(v);

            shape.distance(p).min
        });


        let mut raw_vbuf = Vec::new();

        // Iterate over all cells of the grid
        let points = GridTable::fill_with(resolution, |x, y, z| {
            // Calculate the corresponding point in world space
            let v = Vector3::new(x, y, z).cast::<f64>() / (resolution as f64);
            let p0 = span.start + (span.end - span.start).mul_element_wise(v);
            let step = (span.end - span.start) / resolution as f64;

            let corners = [
                p0 + Vector3::new(   0.0,    0.0,    0.0),
                p0 + Vector3::new(   0.0,    0.0, step.z),
                p0 + Vector3::new(   0.0, step.y,    0.0),
                p0 + Vector3::new(   0.0, step.y, step.z),
                p0 + Vector3::new(step.x,    0.0,    0.0),
                p0 + Vector3::new(step.x,    0.0, step.z),
                p0 + Vector3::new(step.x, step.y,    0.0),
                p0 + Vector3::new(step.x, step.y, step.z),
            ];

            let distances = [
                grid[(x    , y    , z    )],
                grid[(x    , y    , z + 1)],
                grid[(x    , y + 1, z    )],
                grid[(x    , y + 1, z + 1)],
                grid[(x + 1, y    , z    )],
                grid[(x + 1, y    , z + 1)],
                grid[(x + 1, y + 1, z    )],
                grid[(x + 1, y + 1, z + 1)],
            ];

            let partially_in = !(
                distances.iter().all(|&d| d < 0.0) ||
                distances.iter().all(|&d| d > 0.0)
            );

            if partially_in {
                let edge_ids = [
                    // in +x direciton
                    (0, 4),
                    (1, 5),
                    (2, 6),
                    (3, 7),

                    // in +y direction
                    (0, 2),
                    (1, 3),
                    (4, 6),
                    (5, 7),

                    // in +z direction
                    (0, 1),
                    (2, 3),
                    (4, 5),
                    (6, 7),
                ];

                let points: Vec<_> = edge_ids.iter()
                    // we are only interested in the edges with shape crossing
                    .filter(|&&(from, to)| {
                        distances[from].signum() != distances[to].signum()
                    })
                    // weight middle point with distances
                    .map(|&(from, to)| {
                        let mut d_from = distances[from];
                        let mut d_to = distances[to];

                        if d_from > 0.0 {
                            d_from = -d_from;
                            d_to = -d_to;
                        }

                        let weight_from = if d_to == d_from {
                            0.5
                        } else {
                            let d_diff = d_to - d_from;
                            clamp(-d_from / d_diff, 0.0, 1.0)
                        };
                        lerp(corners[from], corners[to], weight_from)
                    })
                    .collect();
                let p = Point3::centroid(&points);

                let dist_p = shape.distance(p);
                let color = (p.to_vec().magnitude() * 0.85).powf(8.0);
                // let color = color * clamp(dist_p.min * 1000.0, 0.0, 1.0);

                raw_vbuf.push(Vertex {
                    position: p.to_vec().cast::<f32>().to_arr(),
                    color: [color as f32; 3],
                });
                raw_vbuf.len() as u32 - 1
            } else {
                // This is a bit hacky, but we will never access this number
                0
            }
        });

        let mut raw_ibuf = Vec::new();

        // Iterate over all edges by iterating over all points
        for (x, y, z) in cube(resolution) {
            // Edge from this point to point in +x direction
            if y > 0 && z > 0 && grid[(x, y, z)].signum() != grid[(x + 1, y, z)].signum()  {
                let v0 = points[(x, y - 1, z - 1)];
                let v1 = points[(x, y - 1, z    )];
                let v2 = points[(x, y    , z - 1)];
                let v3 = points[(x, y    , z    )];

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }
            // Edge from this point to point in +y direction
            if x > 0 && z > 0 && grid[(x, y, z)].signum() != grid[(x, y + 1, z)].signum()  {
                let v0 = points[(x - 1, y, z - 1)];
                let v1 = points[(x - 1, y, z    )];
                let v2 = points[(x,     y, z - 1)];
                let v3 = points[(x,     y, z    )];

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }
            // Edge from this point to point in +z direction
            if x > 0 && y > 0 && grid[(x, y, z)].signum() != grid[(x, y, z + 1)].signum()  {
                let v0 = points[(x - 1, y - 1, z)];
                let v1 = points[(x - 1, y    , z)];
                let v2 = points[(x,     y - 1, z)];
                let v3 = points[(x,     y    , z)];

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }
        }

        debug!(
            "Generated {} points in box ({:?}) @ {} res",
            raw_ibuf.len() / 3,
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
            PrimitiveType::TrianglesList,
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
        assert!(size >= 2);

        let mut data = Vec::with_capacity(size.pow(3) as usize);

        for (x, y, z) in cube(size) {
            data.push(filler(x, y, z));
        }

        GridTable {
            size: size,
            data: data,
        }
    }
}

impl<T> Index<(u32, u32, u32)> for GridTable<T> {
    type Output = T;

    fn index(&self, (x, y, z): (u32, u32, u32)) -> &Self::Output {
        assert!(x < self.size);
        assert!(y < self.size);
        assert!(z < self.size);

        let idx = x * self.size.pow(2) + y * self.size + z;

        &self.data[idx as usize]
    }
}
