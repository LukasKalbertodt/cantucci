use core::math::*;
use core::shape::Shape;
use octree::{Octree, Span, NodeEntryMut};
use util::grid::GridTable;


/// The size of all leaves in the distmap octree. Nodes with this size won't
/// be split.
const LEAF_SIZE: u32 = 16;

pub struct DistMap {
    tree: Octree<NodeState, ()>,
    resolution: u32,
}

impl DistMap {
    pub fn new(span: Span, shape: &Shape, resolution: u32) -> Self {
        assert!(resolution.is_power_of_two());

        fn gen(mut node: NodeEntryMut<NodeState, ()>, shape: &Shape, res: u32) {
            // If our resolution is still bigger, we want to split the node
            if res > LEAF_SIZE {
                node.split(None);
                for child in node.into_children().unwrap() {
                    gen(child, shape, res / 2);
                }
            } else {
                // We won't split the node anymore, but generate the leaf data
                // for it now.
                let span = node.span();
                let dists = GridTable::fill_with(res, |x, y, z| {
                    let v = Vector3::new(x, y, z).cast::<f32>() / (res as f32);
                    let p = span.start + (span.end - span.start).mul_element_wise(v);

                    shape.min_distance_from(p)
                });
                *node.leaf_data_mut().unwrap() = Some(NodeState::Exact(dists));
            }
        }

        let mut tree = Octree::spanning(span);
        gen(tree.root_mut(), shape, resolution);

        DistMap {
            tree: tree,
            resolution: resolution,
        }
    }

    pub fn at(&self, (mut x, mut y, mut z): (u32, u32, u32)) -> Result<f32, Inclusion> {
        assert!(x < self.resolution);
        assert!(y < self.resolution);
        assert!(z < self.resolution);

        // First we have to find the corresponding node in our octree. We do
        // this by choosing the correct node each level and carry a few values
        // with us while doing so.
        //
        // The `shift` value stores the number of binary digits on the right of
        // x/y/z which we are not interested in.
        // E.g. `resolution` is 64 (100_0000 in binary). As x/y/z need to be
        // smaller then the resolution, they have at most the rightmost six
        // bits set. In the first step of choosing the node, we are only
        // interested in the first of those six digits (thus, `shift` is 5). In
        // the next step we will have discarded this first digit (it's 0) and
        // are interested in the second of those six digits (`shift` is 4).
        //
        // x/y/z will be mutated to always describe the grid coordinate in the
        // current node!
        let mut shift = self.resolution.trailing_zeros() - 1;
        let mut node = self.tree.root();

        // Here we walk down the tree until we found a leaf (containing our
        // precious data).
        while !node.is_leaf() {
            // To find the right one of the eight children, we need to look at
            // a specific digit of the binary representation of x/y/z. The
            // digit we are looking at is determined by `shift` and is the
            // leftmost digit which is possibly set. Thus, to isolate it, we
            // only have to left shift by `shift`.
            let xb = x >> shift;
            let yb = y >> shift;
            let zb = z >> shift;

            // Those single bits are no combined into a three bit number which
            // is then used to index the children array. Look at the
            // documentation for `Octnode` to understand why we need to combine
            // the bits in this way (x being the most significant).
            let idx = (xb << 2 | yb << 1 | zb) as u8;

            // Both of these unwraps are safe: the loop condition already tells
            // us that this node is not a leaf node, thus has children. And
            // through all of the reasoning above we can see that xb/yb/zb are
            // either 1 or 0 and that idx can't be bigger than 7 in that case.
            node = node.child(idx).unwrap();

            // We need to adjust our shift value and the coordinates. We just
            // cut off the bit we were just looking at.
            //
            // E.g. this is the first loop iteration, resolution is 64 and x
            // is 40 (0b101000). As discussed above, `shift` is 5 now. We just
            // looked at the most significant bit of x and want to cut it off.
            //
            //     mask = (1 << shift) - 1 = 0b100000 - 1 = 0b11111
            //
            // With this mask, we will set the MSB to 0.
            let mask = (1 << shift) - 1;
            x &= mask;
            y &= mask;
            z &= mask;
            shift -= 1;
        }

        match *node.leaf_data().unwrap() {
            NodeState::Skipped(incl) => Err(incl),
            NodeState::Exact(ref data) => Ok(data[(x, y, z)]),
        }
    }

    pub fn inclusion_at(&self, coords: (u32, u32, u32)) -> Inclusion {
        self.at(coords)
            .map(Inclusion::from_dist)
            .unwrap_or_else(|i| i)
    }
}


enum NodeState {
    Skipped(Inclusion),
    Exact(GridTable<f32>),
}

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

    // pub fn is_inside(&self) -> bool {
    //     *self == Inclusion::Inside
    // }
}
