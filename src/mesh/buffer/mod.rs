use std::time::{Duration, Instant};
use std::fmt;
use std::ops;

use core::math::*;
use core::Shape;
use octree::Span;
use util::ToArr;
use util::iter;
use util::grid::GridTable;
use util::time::DurationExt;

mod dist_map;

use self::dist_map::DistMap;


#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    distance_from_surface: f32,
}

implement_vertex!(Vertex, position, normal, distance_from_surface);

#[derive(Default, Clone, Copy)]
pub struct Timings {
    first: Duration,
    second: Duration,
    third: Duration,
}

impl fmt::Display for Timings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let all = self.first + self.second + self.third;
        write!(
            f,
            "{:>11} ({:>11}, {:>11}, {:>11})",
            all.display_ms(),
            self.first.display_ms(),
            self.second.display_ms(),
            self.third.display_ms(),
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
        }
    }
}


pub struct MeshBuffer {
    raw_vbuf: Vec<Vertex>,
    raw_ibuf: Vec<u32>,
    // resolution: u32, // TODO: use or delete
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
        fn interpolate(
            p: Point3<f32>,
            values: &[f32; 8],
            min: Point3<f32>,
            max: Point3<f32>,
        ) -> f32 {
            let d = (p - min).div_element_wise(max - min);

            let c00 = values[0] * (1.0 - d.x) + values[4] * d.x;
            let c01 = values[1] * (1.0 - d.x) + values[5] * d.x;
            let c10 = values[2] * (1.0 - d.x) + values[6] * d.x;
            let c11 = values[3] * (1.0 - d.x) + values[7] * d.x;

            let c0 = c00 * (1.0 - d.y) + c10 * d.y;
            let c1 = c01 * (1.0 - d.y) + c11 * d.y;

            c0 * (1.0 - d.z) + c1 * d.z
        }


        // Adjust span to avoid holes in between two boxes
        let span = {
            let overflow = (span.end - span.start) / (resolution - 1) as f32;
            span.start + -overflow .. span.end + overflow
        };

        let before_first = Instant::now();

        // First Step:
        // ===========
        //
        // We partition our box into regular cells. For each corner in between
        // the cells we calculate and save the estimated minimal distance from
        // the shape.
        // let dists = GridTable::fill_with(resolution, |x, y, z| {
        //     let v = Vector3::new(x, y, z).cast::<f32>() / (resolution as f32);
        //     let p = span.start + (span.end - span.start).mul_element_wise(v);

        //     shape.min_distance_from(p)
        // });

        let dists = DistMap::new(span.clone(), shape, resolution);

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
        let mut vertices = GridTable::fill_with(resolution - 1, |_, _, _| None);
        for (x, y) in iter::square(resolution - 1) {
            // Calculate the position of all eight corners of the current cell
            // in world space. The term "lower corner" describes the corner
            // with the lowest x, y and z coordinates.
            let (mut corners, step_z) = {
                // The world space distance between two corners/between the
                // center points of two cells.
                let step = (span.end - span.start) / (resolution - 1) as f32;

                // World position of this cell's lower corner
                let p0 = span.start
                    + Vector3::new(x, y, 0).cast::<f32>().mul_element_wise(step);

                (
                    [
                        p0 + Vector3::new(   0.0,    0.0,    0.0),
                        p0 + Vector3::new(   0.0,    0.0, step.z),
                        p0 + Vector3::new(   0.0, step.y,    0.0),
                        p0 + Vector3::new(   0.0, step.y, step.z),
                        p0 + Vector3::new(step.x,    0.0,    0.0),
                        p0 + Vector3::new(step.x,    0.0, step.z),
                        p0 + Vector3::new(step.x, step.y,    0.0),
                        p0 + Vector3::new(step.x, step.y, step.z),
                    ],
                    step.z
                )
            };

            // Every second value is a dummy value which will be overwritten
            let mut samples = [
                dists.at((x    , y    , 0)),
                dists.at((x    , y    , 0)),
                dists.at((x    , y + 1, 0)),
                dists.at((x    , y + 1, 0)),
                dists.at((x + 1, y    , 0)),
                dists.at((x + 1, y    , 0)),
                dists.at((x + 1, y + 1, 0)),
                dists.at((x + 1, y + 1, 0)),
            ];

            for z in 0..resolution - 1 {
                samples[1] = dists.at((x    , y    , z + 1));
                samples[3] = dists.at((x    , y + 1, z + 1));
                samples[5] = dists.at((x + 1, y    , z + 1));
                samples[7] = dists.at((x + 1, y + 1, z + 1));

                // First, check if the current cell is only partially inside the
                // shape (if the cell intersects the shape's surface). If that's
                // not the case, we won't generate a vertex for this cell.
                let partially_in = !(
                    samples.iter().all(|qr| qr.is_inside()) ||
                    samples.iter().all(|qr| qr.is_outside())
                );

                if partially_in {

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
                            samples[from].inclusion() != samples[to].inclusion()
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
                            let (d_from, d_to) = if samples[from].is_inside() {
                                (samples[from].dist().unwrap_or(0.0), samples[to].dist().unwrap_or(0.0))
                            } else {
                                (-samples[from].dist().unwrap_or(0.0), -samples[to].dist().unwrap_or(0.0))
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

                    let dists = [
                        samples[0].dist().unwrap_or(0.0),
                        samples[1].dist().unwrap_or(0.0),
                        samples[2].dist().unwrap_or(0.0),
                        samples[3].dist().unwrap_or(0.0),
                        samples[4].dist().unwrap_or(0.0),
                        samples[5].dist().unwrap_or(0.0),
                        samples[6].dist().unwrap_or(0.0),
                        samples[7].dist().unwrap_or(0.0),
                    ];

                    // Now we only calculate some meta data which might be used to
                    // color the vertex.
                    // let dist_p = shape.min_distance_from(p);
                    // let dist_p = 0.0;

                    let normal = {
                        let delta = 1.0 * (span.end - span.start) / resolution as f32;

                        let v = Vector3::new(
                            interpolate(p + Vector3::unit_x() * delta.x, &dists, corners[0], corners[7])
                                - interpolate(p +  Vector3::unit_x() * -delta.x, &dists, corners[0], corners[7]),
                            interpolate(p + Vector3::unit_y() * delta.y, &dists, corners[0], corners[7])
                                - interpolate(p +  Vector3::unit_y() * -delta.y, &dists, corners[0], corners[7]),
                            interpolate(p + Vector3::unit_z() * delta.z, &dists, corners[0], corners[7])
                                - interpolate(p +  Vector3::unit_z() * -delta.z, &dists, corners[0], corners[7]),
                        );
                        if v.magnitude() == 0.0 {

                            // println!("{:?}", v);
                            // println!("dists: {:#?}", dists);
                            println!("---------------");
                            println!("p={:?}", p);
                            println!("samples: {:#?}", samples);
                            // panic!();
                            Vector3::new(
                                shape.min_distance_from(p + Vector3::unit_x() * delta.x)
                                    - shape.min_distance_from(p +  Vector3::unit_x() * -delta.x),
                                shape.min_distance_from(p + Vector3::unit_y() * delta.y)
                                    - shape.min_distance_from(p +  Vector3::unit_y() * -delta.y),
                                shape.min_distance_from(p + Vector3::unit_z() * delta.z)
                                    - shape.min_distance_from(p +  Vector3::unit_z() * -delta.z),
                            ).normalize()
                        } else {
                            v.normalize()
                        }

                        // Vector3::new(1.0, 0.0, 0.0)
                    };
                    let distp2 = interpolate(p, &dists, corners[0], corners[7]);
                    // let distp2 = 0.0;


                    raw_vbuf.push(Vertex {
                        position: p.to_vec().cast::<f32>().to_arr(),
                        normal: normal.to_arr(),
                        distance_from_surface: distp2,
                    });
                    vertices[(x, y, z)] = Some(raw_vbuf.len() as u32 - 1);
                }

                // Prepare running values for next iteration
                samples[0] = samples[1];
                samples[2] = samples[3];
                samples[4] = samples[5];
                samples[6] = samples[7];

                for corner in &mut corners {
                    corner.z += step_z;
                }
            }
        }

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
        for (x, y, z) in iter::cube(resolution - 1) {
            // We iterate over all edges by iterating over all lower corners of
            // all cells.
            //
            // About all those `unwrap()` calls: if the edge is crossing the
            // surface (which is checked in the if conditions below), then we
            // generated a vertex for all of the adjacent cells (as they,
            // by definition, also cross the surface). So the Options we access
            // are always `Some()`.

            // Edge from the current corner pointing in +x direction
            if y > 0 && z > 0 && dists.inclusion_at((x, y, z)) != dists.inclusion_at((x + 1, y, z))  {
            // if y > 0 && z > 0 && dists[(x, y, z)].signum() != dists[(x + 1, y, z)].signum()  {
                let v0 = vertices[(x, y - 1, z - 1)].unwrap();
                let v1 = vertices[(x, y - 1, z    )].unwrap();
                let v2 = vertices[(x, y    , z - 1)].unwrap();
                let v3 = vertices[(x, y    , z    )].unwrap();

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }

            // Edge from the current corner pointing in +y direction
            if x > 0 && z > 0 && dists.inclusion_at((x, y, z)) != dists.inclusion_at((x, y + 1, z))  {
            // if x > 0 && z > 0 && dists[(x, y, z)].signum() != dists[(x, y + 1, z)].signum()  {
                let v0 = vertices[(x - 1, y, z - 1)].unwrap();
                let v1 = vertices[(x - 1, y, z    )].unwrap();
                let v2 = vertices[(x,     y, z - 1)].unwrap();
                let v3 = vertices[(x,     y, z    )].unwrap();

                raw_ibuf.extend_from_slice(&[
                    v0, v1, v2,
                    v1, v2, v3,
                ]);
            }

            // Edge from the current corner pointing in +z direction
            if x > 0 && y > 0 && dists.inclusion_at((x, y, z)) != dists.inclusion_at((x, y, z + 1))  {
            // if x > 0 && y > 0 && dists[(x, y, z)].signum() != dists[(x, y, z + 1)].signum()  {
                let v0 = vertices[(x - 1, y - 1, z)].unwrap();
                let v1 = vertices[(x - 1, y    , z)].unwrap();
                let v2 = vertices[(x,     y - 1, z)].unwrap();
                let v3 = vertices[(x,     y    , z)].unwrap();

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
        };

        trace!(
            "Generated {:5} points, {:5} triangles in {}",
            raw_vbuf.len(),
            raw_ibuf.len() / 3,
            timings,
        );

        (
            MeshBuffer {
                raw_vbuf: raw_vbuf,
                raw_ibuf: raw_ibuf,
                // resolution: resolution,  // TODO: use or delete
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

    // TODO: use or delete
    // pub fn resolution(&self) -> u32 {
    //     self.resolution
    // }
}






// mod benchi {
//     extern crate test;

//     use self::test::Bencher;
//     use super::*;
//     use core::shape::Mandelbulb;

//     #[bench]
//     fn bench_mandelbulb_10(b: &mut Bencher) {
//         let shape = Mandelbulb::classic(5, 2.5);
//         b.iter(|| MeshBuffer::generate_for_box(
//             &(Point3::new(-1.2, -1.2, -1.2) .. Point3::new(1.2, 1.2, 1.2)),
//             &shape,
//             16,
//         ))
//     }

//     // #[bench]
//     // fn bench_mandelbulb_25(b: &mut Bencher) {
//     //     let shape = Mandelbulb::classic(5, 2.5);
//     //     b.iter(|| MeshBuffer::generate_for_box(
//     //         &(Point3::new(-1.2, -1.2, -1.2) .. Point3::new(1.2, 1.2, 1.2)),
//     //         &shape,
//     //         25,
//     //     ))
//     // }

//     // #[bench]
//     // fn bench_mandelbulb_50(b: &mut Bencher) {
//     //     let shape = Mandelbulb::classic(5, 2.5);
//     //     b.iter(|| MeshBuffer::generate_for_box(
//     //         &(Point3::new(-1.2, -1.2, -1.2) .. Point3::new(1.2, 1.2, 1.2)),
//     //         &shape,
//     //         50,
//     //     ))
//     // }

//     #[bench]
//     fn classic_5(b: &mut Bencher) {
//         let shape = Mandelbulb::classic(5, 2.5);
//         let points = [
//             Point3::new(0.0, 0.0, 0.0),
//             Point3::new(1.0, 0.0, 0.0),
//             Point3::new(0.0, 1.0, 0.0),
//             Point3::new(0.0, 0.0, 1.0),
//             Point3::new(0.3, 0.3, 0.4),
//         ];

//         b.iter(|| {
//             points.iter().map(|&p| shape.min_distance_from(p)).sum::<f32>()
//         })
//     }
// }
