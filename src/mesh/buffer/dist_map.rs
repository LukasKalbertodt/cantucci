use core::math::*;
use core::shape::Shape;
use octree::Span;
use util::iter;

/// How often the box is split in one dimension. One split results in 8
/// children (like in an octree). Two splits result in 8² = 64 children, and
/// so on ...
const NUM_SPLITS: u32 = 4;

/// When we split x times in one dimension, we will have 2^x many sections in
/// that dimension;
const SECTIONS: u32 = 1 << NUM_SPLITS;

pub struct DistMap {
    // tree: Octree<NodeState, ()>,
    lookup: Box<[BoxState]>,
    dists: Box<[f32]>,
    resolution: u32,
    chunk_len: u32,
    mask_low_bits: u32,
    num_low_bits: u8,
}

enum BoxState {
    Skipped(Inclusion),
    /// The integer functions as an index into the `dists` array.
    Exact(u32),
}

#[derive(Copy, Clone, Debug)]
pub enum QueryResult {
    Skipped(Inclusion),
    Exact(f32),
}

impl QueryResult {
    pub fn inclusion(&self) -> Inclusion {
        match *self {
            QueryResult::Skipped(incl) => incl,
            QueryResult::Exact(dist) => Inclusion::from_dist(dist),
        }
    }

    pub fn is_inside(&self) -> bool {
        self.inclusion().is_inside()
    }

    pub fn is_outside(&self) -> bool {
        self.inclusion().is_outside()
    }

    pub fn dist(&self) -> Option<f32> {
        match *self {
            QueryResult::Skipped(_) => None,
            QueryResult::Exact(dist) => Some(dist),
        }
    }
}

impl DistMap {
    pub fn new(span: Span, shape: &Shape, resolution: u32) -> Self {
        assert!(resolution.is_power_of_two());
        assert!(resolution >= SECTIONS);

        let chunk_res = resolution / SECTIONS;
        let chunk_span_size = (span.end - span.start) / SECTIONS as f32;
        let chunk_radius = chunk_span_size.magnitude() / 2.0;

        /// We prepare both data arrays. The lookup array has exactly
        /// 8^NUM_SPLITS elements. We can't really say anything about the size
        /// of the array holding the exact values.
        let mut lookup = Vec::with_capacity(8usize.pow(NUM_SPLITS));
        let mut dists = Vec::new();

        for (x, y, z) in iter::cube(SECTIONS) {
            let chunk_start = span.start + Vector3::new(x, y, z)
                .cast::<f32>()
                .mul_element_wise(chunk_span_size);
            let chunk_center = chunk_start + chunk_span_size / 2.0;

            let center_dist = shape.min_distance_from(chunk_center);
            //
            if (center_dist * 1.0).abs() > chunk_radius {
                lookup.push(BoxState::Skipped(Inclusion::from_dist(center_dist)));
            } else {

                // We will fill in all distances for the current box, so we need
                // more space in the `dists` array.
                dists.reserve((chunk_res as usize).pow(3));
                let idx = dists.len();

                for (x, y, z) in iter::cube(chunk_res) {
                    let v = Vector3::new(x, y, z).cast::<f32>() / (chunk_res as f32);
                    let p = chunk_start + chunk_span_size.mul_element_wise(v);

                    dists.push(shape.min_distance_from(p));
                }

                lookup.push(BoxState::Exact(idx as u32));
            }
        }

        DistMap {
            lookup: lookup.into_boxed_slice(),
            dists: dists.into_boxed_slice(),
            resolution: resolution,
            num_low_bits: (resolution.trailing_zeros() - NUM_SPLITS) as u8,
            chunk_len: chunk_res.pow(3),
            mask_low_bits: (resolution >> NUM_SPLITS) - 1,
        }
    }

    pub fn at(&self, (x, y, z): (u32, u32, u32)) -> QueryResult {
        assert!(x < self.resolution);
        assert!(y < self.resolution);
        assert!(z < self.resolution);


        // To calculate the index in `lookup` we take bits from x, y and z:
        //
        //     idx = 0bxxyyzz;
        //
        // We use NUM_SPLITS bits per dimension. This results in
        // 2^(3 * NUM_SPLITS) = 8^NUM_SPLITS possible indices.
        //
        // `resolution` is a power of two, so its bit pattern looks like
        // b100_0000 (for resolution = 64). The number of trailing zeros is 6,
        // which is saved in `num_bits`. We only look at the last `num_bits`
        // bits of x/y/z; the other ones have to be 0 anyway (note the asserts
        // above).
        //
        // Of these `num_bits` bits of x/y/z we use the NUM_SPLITS most
        // significant ones to index `lookup`.
        let xb = x >> self.num_low_bits;
        let yb = y >> self.num_low_bits;
        let zb = z >> self.num_low_bits;
        let idx = (xb << (2 * NUM_SPLITS)) | (yb << NUM_SPLITS) | zb;
        // println!("{}", idx);

        match self.lookup[idx as usize] {
            BoxState::Skipped(incl) => QueryResult::Skipped(incl),
            BoxState::Exact(offset) => {
                let offset = offset as usize;
                let curr_box = &self.dists[offset..offset + self.chunk_len as usize];

                // Now we need to index this second array by using the
                // remaining bits of x/y/z.
                let xb = x & self.mask_low_bits;
                let yb = y & self.mask_low_bits;
                let zb = z & self.mask_low_bits;
                let inner_idx = (xb << (2 * self.num_low_bits))
                    | (yb << self.num_low_bits)
                    | zb;

                // if inner_idx == 4096 {
                //     println!("{:?} => {:?}", (x, y, z), (xb, yb, zb));
                // }

                QueryResult::Exact(curr_box[inner_idx as usize])
            }
        }

    }

    pub fn inclusion_at(&self, coords: (u32, u32, u32)) -> Inclusion {
        self.at(coords).inclusion()
    }
}


// enum NodeState {
//     Skipped(Inclusion),
//     Exact(GridTable<f32>),
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Inclusion {
    Inside,
    Outside,
}

impl Inclusion {
    pub fn from_dist(dist: f32) -> Self {
        if dist < 0.0 {
            Inclusion::Inside
        } else {
            Inclusion::Outside
        }
    }

    pub fn is_inside(&self) -> bool {
        *self == Inclusion::Inside
    }

    pub fn is_outside(&self) -> bool {
        *self == Inclusion::Outside
    }
}
