use std::time::{Duration, Instant};
use std::fmt;
use std::ops;

use math::*;
use shape::Shape;
use octree::Span;
use util::ToArr;
use util::iter::cube;
use util::grid::GridTable;
use util::time::DurationExt;


pub struct MeshBuffer {
    raw_vbuf: Vec<Vertex>,
    raw_ibuf: Vec<u32>,
}

impl MeshBuffer {
    pub fn generate_for_box(
        span: &Span,
        shape: &Shape,
        resolution: u32,
    ) -> (Self, Timings) {
        assert!(span.start.x < span.end.x);
        assert!(span.start.y < span.end.y);
        assert!(span.start.z < span.end.z);
        assert!(resolution != 0);
        assert!(resolution.is_power_of_two());

        Self::naive_surface_nets(span, shape, resolution)
    }

    /// Implementation of the "Surface Nets" algorithm.
    ///
    /// In particular, in this implementation the position of the vertex inside
    /// the 3D-cell is simply the centroid of all edge crossings. This rather
    /// easy version is described [in this article][1] ("naive surface nets").
    ///
    /// The article will also help understand this algorithm. Compared to
    /// other algorithms for rendering iso surfaces, this one is relatively
    /// easy to implement while still working fairly nice.
    ///
    /// In the future we might want to switch to the "Dual Contouring" scheme
    /// as it preserves sharp features of the shape (see #2).
    ///
    /// [1]: https://0fps.net/2012/07/12/smooth-voxel-terrain-part-2/
    fn naive_surface_nets(
        span: &Span,
        shape: &Shape,
        resolution: u32,
    ) -> (Self, Timings) {
        // Adjust span to avoid holes in between two boxes
        let span = {
            let overflow = (span.end - span.start) / resolution as f32;
            span.start + -overflow .. span.end + overflow
        };

        let before_first = Instant::now();

        // First Step:
        // ===========
        //
        // We partition our box into regular cells. For each corner in between
        // the cells we calculate and save the estimated minimal distance from
        // the shape.
        let dists = GridTable::fill_with(resolution + 1, |x, y, z| {
            let v = Vector3::new(x, y, z).cast::<f32>() / (resolution as f32);
            let p = span.start + (span.end - span.start).mul_element_wise(v);

            shape.min_distance_from(p)
        });

        let before_second = Instant::now();


        // Second Step:
        // ============
        //
        // Next, we will iterate over all cells of the box (unlike before
        // where we iterated over corners). For each cell crossing the shape's
        // surface, we will generate one vertex. The `points` grid table holds
        // the index of the vertex corresponding to the cell, or `None` if the
        // cell does not cross the surface.
        //
        let mut raw_vbuf = Vec::new();
        let points = GridTable::fill_with(resolution, |x, y, z| {
            // Calculate the position of all eight corners of the current cell
            // in world space. The term "lower corner" describes the corner
            // with the lowest x, y and z coordinates.
            let corners = {
                // The world space distance between two corners/between the
                // center points of two cells.
                let step = (span.end - span.start) / resolution as f32;

                // World position of this cell's lower corner
                let p0 = span.start
                    + Vector3::new(x, y, z).cast::<f32>().mul_element_wise(step);

                [
                    p0 + Vector3::new(   0.0,    0.0,    0.0),
                    p0 + Vector3::new(   0.0,    0.0, step.z),
                    p0 + Vector3::new(   0.0, step.y,    0.0),
                    p0 + Vector3::new(   0.0, step.y, step.z),
                    p0 + Vector3::new(step.x,    0.0,    0.0),
                    p0 + Vector3::new(step.x,    0.0, step.z),
                    p0 + Vector3::new(step.x, step.y,    0.0),
                    p0 + Vector3::new(step.x, step.y, step.z),
                ]
            };

            // The estimated minimal distances of all eight corners calculated
            // in the prior step.
            let distances = [
                dists[(x    , y    , z    )],
                dists[(x    , y    , z + 1)],
                dists[(x    , y + 1, z    )],
                dists[(x    , y + 1, z + 1)],
                dists[(x + 1, y    , z    )],
                dists[(x + 1, y    , z + 1)],
                dists[(x + 1, y + 1, z    )],
                dists[(x + 1, y + 1, z + 1)],
            ];

            // First, check if the current cell is only partially inside the
            // shape (if the cell intersects the shape's surface). If that's
            // not the case, we won't generate a vertex for this cell.
            let partially_in = !(
                distances.iter().all(|&d| d < 0.0) ||
                distances.iter().all(|&d| d > 0.0)
            );

            if !partially_in {
                // FIXME
                // This is a bit hacky, but we will never access this number
                return None;
            }

            // We want to iterate over all 12 edges of the cell. Here, we list
            // all edges by specifying their corner indices.
            const EDGES: [(usize, usize); 12] = [
                // Edges whose endpoints differ in the x coordinate (first
                // corner id is -x, second is +x).
                (0, 4),     //    -y -z
                (1, 5),     //    -y +z
                (2, 6),     //    +y -z
                (3, 7),     //    +y +z

                // Edges whose endpoints differ in the y coordinate (first
                // corner id is -y, second is +y).
                (0, 2),     // -x    -z
                (1, 3),     // -x    +z
                (4, 6),     // +x    -z
                (5, 7),     // +x    +z

                // Edges whose endpoints differ in the z coordinate (first
                // corner id is -z, second is +z).
                (0, 1),     // -x -y
                (2, 3),     // -x +y
                (4, 5),     // +x -y
                (6, 7),     // +x +y
            ];

            // Get all edge crossings. These are points where the edges of the
            // current cell intersect the surface... more or less. We do NOT
            // find the correct crossing point by ray marching, as this would
            // require more queries to the shape (our current bottleneck for
            // mandelbulb).
            //
            // Instead, we simply weight both endpoints of the edge by the
            // already calculated distances. Improving this might be worth
            // experimenting (see #1).
            let points: Vec<_> = EDGES.iter().cloned()

                // We are only interested in the edges with shape crossing. The
                // edge crosses the shape iff the endpoints' estimated minimal
                // distances have different signs ("minus" means: inside the
                // shape).
                .filter(|&(from, to)| {
                    distances[from].signum() != distances[to].signum()
                })

                // Next, we convert the edge into a vertex on said edge. We
                // could just use the center point of the two endpoints. But
                // weighting each endpoint with the estimated minimal distance
                // from the shape results in a mesh more closely representing
                // the shape.
                .map(|(from, to)| {
                    // Here we want to make sure that `d_from` is  negative and
                    // `d_to` is positive.
                    //
                    // Remember: we already know that both distances have
                    // different signs!
                    let (d_from, d_to) = if distances[from] < 0.0 {
                        (distances[from], distances[to])
                    } else {
                        (-distances[from], -distances[to])
                    };

                    // This condition is only true if `d_from == -0.0`. In
                    // theory this might happen, so we better deal with it.
                    let weight_from = if d_to == d_from {
                        0.5
                    } else {
                        // Here we calculate the weight (a number between 0 and
                        // 1 inclusive) for the `from` endpoint. `delta` is
                        // the difference between the two distances.
                        //
                        // First we will shift the distance to "the right",
                        // making it positive. Then, we scale it by delta.
                        //
                        // - d_from + delta is always >= 0.0
                        // - d_from + delta is always <= delta
                        // ==> `(d_from + delta) / delta` is always in 0...1
                        //
                        // For d_from == 0 and d_to > 0:
                        // - d_from + delta == delta
                        // ==> result is: delta / delta == 1
                        //
                        // For d_from < 0 and d_to == 0:
                        // - d_from + delta == 0
                        // ==> result is: 0 / delta == 0
                        let delta = d_to - d_from;
                        (d_from + delta) / delta
                    };

                    lerp(corners[from], corners[to], weight_from)
                })
                .collect();

            // As described in the article above, we simply use the centroid
            // of all edge crossings.
            let p = Point3::centroid(&points);

            // Now we only calculate some meta data which might be used to
            // color the vertex.
            let dist_p = shape.min_distance_from(p);

            let normal = {
                let delta = 0.7 * (span.end - span.start) / resolution as f32;
                Vector3::new(
                    shape.min_distance_from(p + Vector3::unit_x() * delta.x)
                        - shape.min_distance_from(p +  Vector3::unit_x() * -delta.x),
                    shape.min_distance_from(p + Vector3::unit_y() * delta.y)
                        - shape.min_distance_from(p +  Vector3::unit_y() * -delta.y),
                    shape.min_distance_from(p + Vector3::unit_z() * delta.z)
                        - shape.min_distance_from(p +  Vector3::unit_z() * -delta.z),
                ).normalize()
            };

            raw_vbuf.push(Vertex {
                position: p.to_vec().cast::<f32>().to_arr(),
                normal: normal.to_arr(),
                distance_from_surface: dist_p,
            });
            Some(raw_vbuf.len() as u32 - 1)
        });

        let before_third = Instant::now();


        // Third step:
        // ===========
        //
        // We already have all vertices, now we need to generate the faces
        // of our resulting mesh. For each edge crossing the surface of our
        // shape, we will generate one face. This face's vertices are the
        // vertices inside the four cells the edge is adjacent to.
        //
        let mut raw_ibuf = Vec::new();
        for (x, y, z) in cube(resolution) {
            // We iterate over all edges by iterating over all lower corners of
            // all cells.
            //
            // About all those `unwrap()` calls: if the edge is crossing the
            // surface (which is checked in the if conditions below), then we
            // generated a vertex for all of the adjacent cells (as they,
            // by definition, also cross the surface). So the Options we access
            // are always `Some()`.

            // Edge from the current corner pointing in +x direction
            if y > 0 && z > 0 && dists[(x, y, z)].signum() != dists[(x + 1, y, z)].signum()  {
                let v0 = points[(x, y - 1, z - 1)].unwrap();
                let v1 = points[(x, y - 1, z    )].unwrap();
                let v2 = points[(x, y    , z - 1)].unwrap();
                let v3 = points[(x, y    , z    )].unwrap();

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }

            // Edge from the current corner pointing in +y direction
            if x > 0 && z > 0 && dists[(x, y, z)].signum() != dists[(x, y + 1, z)].signum()  {
                let v0 = points[(x - 1, y, z - 1)].unwrap();
                let v1 = points[(x - 1, y, z    )].unwrap();
                let v2 = points[(x,     y, z - 1)].unwrap();
                let v3 = points[(x,     y, z    )].unwrap();

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }

            // Edge from the current corner pointing in +z direction
            if x > 0 && y > 0 && dists[(x, y, z)].signum() != dists[(x, y, z + 1)].signum()  {
                let v0 = points[(x - 1, y - 1, z)].unwrap();
                let v1 = points[(x - 1, y    , z)].unwrap();
                let v2 = points[(x,     y - 1, z)].unwrap();
                let v3 = points[(x,     y    , z)].unwrap();

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }
        }

        let after_third = Instant::now();
        let timings = Timings {
            first: before_second - before_first,
            second: before_third - before_second,
            third: after_third -  before_third,
            vertices: raw_vbuf.len() as u32,
            faces: raw_ibuf.len() as u32 / 6,
        };

        trace!(
            "Generated {:6} points, {:6} faces in {}",
            raw_vbuf.len(),
            raw_ibuf.len() / 6,
            timings,
        );

        (
            MeshBuffer {
                raw_vbuf: raw_vbuf,
                raw_ibuf: raw_ibuf,
            },
            timings
        )
    }

    pub fn raw_vbuf(&self) -> &[Vertex] {
        &self.raw_vbuf
    }

    pub fn raw_ibuf(&self) -> &[u32] {
        &self.raw_ibuf
    }
}


/// Stores some information about how long various passes of the mesh
/// generation algorithm were running as well as how many vertices and faces
/// were created.
#[derive(Default, Clone, Copy)]
pub struct Timings {
    first: Duration,
    second: Duration,
    third: Duration,
    vertices: u32,
    faces: u32,
}

impl fmt::Display for Timings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let all = self.first + self.second + self.third;
        write!(
            f,
            "{:>11} ({:>11}, {:>11}, {:>11}) => [{:6} verts, {:6} faces]",
            all.display_ms(),
            self.first.display_ms(),
            self.second.display_ms(),
            self.third.display_ms(),
            self.vertices,
            self.faces,
        )
    }
}

impl ops::Add for Timings {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Timings {
            first: self.first + other.first,
            second: self.second + other.second,
            third: self.third + other.third,
            vertices: self.vertices + other.vertices,
            faces: self.faces + other.faces,
        }
    }
}

/// Per vertex data in the generated mesh.
#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    distance_from_surface: f32,
}

implement_vertex!(Vertex, position, normal, distance_from_surface);
