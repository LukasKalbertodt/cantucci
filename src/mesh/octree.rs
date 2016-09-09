use std::ops::Range;
use core::math::*;
use arrayvec::ArrayVec;

/// A box in three dimensional space that is represented by one octree node
pub type Span = Range<Point3<f64>>;

pub trait SpanExt {
    fn center(&self) -> Point3<f64>;
    fn contains(&self, p: Point3<f64>) -> bool;
}

impl SpanExt for Span {
    fn center(&self) -> Point3<f64> {
        self.start + (self.end - self.start) / 2.0
    }

    fn contains(&self, p: Point3<f64>) -> bool {
        let s = self.start;
        let e = self.end;

        s.x < p.x && s.y < p.y && s.z < p.z
            && p.x < e.x && p.y < e.y && p.z < e.z
    }
}

/// Recursively partitions three dimensional space into eight octants. In this
/// simple implementation all octants have the same size.
///
/// In this application it's used to save the representation of the octant in
/// order to allow different resolutions in different parts of space.
pub struct Octree<T> {
    span: Range<Point3<f64>>,
    root: Octnode<T>,
}

impl<T> Octree<T> {
    /// Creates an empty octree representing the given span
    pub fn spanning(span: Span) -> Self {
        Octree {
            span: span,
            root: Octnode::Leaf(None),
        }
    }

    /// Returns the span of the root element
    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn root(&self) -> NodeEntry<T> {
        NodeEntry {
            node: &self.root,
            span: self.span(),
        }
    }

    pub fn root_mut(&mut self) -> NodeEntryMut<T> {
        NodeEntryMut {
            span: self.span(),
            node: &mut self.root,
        }
    }

    // pub fn leaf_around(&self) -> NodeEntry<T> {

    // }

    pub fn leaf_mut_around(&mut self, p: Point3<f64>) -> NodeEntryMut<T> {
        let mut node = self.root_mut();
        loop {
            if node.is_leaf() {
                return node;
            } else {
                node = node
                    .into_children()
                    .unwrap()
                    .into_iter()
                    .find(|c| c.span().contains(p))
                    .unwrap();
            }
        }
    }

    /// Returns an iterator over *im*mutable nodes
    pub fn iter(&self) -> Iter<T> {
        Iter {
            to_visit: vec![
                NodeEntry {
                    node: &self.root,
                    span: self.span(),
                }
            ]
        }
    }
}

// `IntoIterator` impls for convenience
impl<'a, T> IntoIterator for &'a Octree<T> {
    type Item = NodeEntry<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// One node of the octree. This type is implementation detail of the data
/// structure and isn't usually exposed to the user.
enum Octnode<T> {
    /// At this point, space is not further subdivided
    Leaf(Option<T>),

    /// Space is divided into eight octants, saved in this array. The order of
    /// the octants is as follows:
    ///
    /// - 0 => (-x, -y, -z)
    /// - 1 => (-x, -y, +z)
    /// - 2 => (-x, +y, -z)
    /// - 3 => (-x, +y, +z)
    /// - 4 => (+x, -y, -z)
    /// - 5 => (+x, -y, +z)
    /// - 6 => (+x, +y, -z)
    /// - 7 => (+x, +y, +z)
    SubTree(Box<[Octnode<T>; 8]>),
}



// ===========================================================================

const SPLIT_SPAN_DIFF_AMOUNT: &'static [Vector3<f64>; 8] = &[
    Vector3 { x: 0.0, y: 0.0, z: 0.0 },
    Vector3 { x: 0.0, y: 0.0, z: 1.0 },
    Vector3 { x: 0.0, y: 1.0, z: 0.0 },
    Vector3 { x: 0.0, y: 1.0, z: 1.0 },
    Vector3 { x: 1.0, y: 0.0, z: 0.0 },
    Vector3 { x: 1.0, y: 0.0, z: 1.0 },
    Vector3 { x: 1.0, y: 1.0, z: 0.0 },
    Vector3 { x: 1.0, y: 1.0, z: 1.0 },
];

/// An *im*mutable reference to a node inside the tree that knows about its
/// span.
pub struct NodeEntry<'a, T: 'a> {
    node: &'a Octnode<T>,
    span: Span,
}

impl<'a, T> NodeEntry<'a, T> {
    /// Returns `true` if the referenced node is a leaf node
    pub fn is_leaf(&self) -> bool {
        match *self.node {
            Octnode::Leaf(_) => true,
            _ => false,
        }
    }

    /// Returns the span of the referenced node
    pub fn span(&self) -> Span {
        self.span.clone()
    }

    /// If the referenced node is a leaf node and this leaf node contains a
    /// value, that value is returned; `None` otherwise.
    pub fn leaf_data(&self) -> Option<&'a T> {
        match *self.node {
            Octnode::Leaf(ref data) => data.as_ref(),
            _ => None,
        }
    }

    /// If the referenced node `n` is *not* a leaf node, eight `NodeEntry`s
    /// referencing all eight children of `n` are returned; `None` otherwise.
    pub fn children(&self) -> Option<ArrayVec<[Self; 8]>> {
        match *self.node {
            Octnode::SubTree(ref children) => {
                let start = self.span.start;
                let half_diff = (self.span.end - start) / 2.0;

                Some(children
                    .iter()
                    .zip(SPLIT_SPAN_DIFF_AMOUNT)
                    .map(|(child, &diff_amount)| {
                        let child_start = start + half_diff.mul_element_wise(diff_amount);
                        let child_end = child_start + half_diff;

                        NodeEntry {
                            node: child,
                            span: child_start .. child_end,
                        }
                    })
                    .collect()
                )
            },
            _ => None,
        }
    }
}

// ===========================================================================

/// An iterator over *im*mutable references of nodes
pub struct Iter<'a, T: 'a> {
    to_visit: Vec<NodeEntry<'a, T>>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = NodeEntry<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.to_visit.pop().map(|next| {
            if let Some(children) = next.children() {
                self.to_visit.extend(ArrayVec::from(children));
            }
            next
        })
    }
}

// ===========================================================================

/// A *mutable* reference to a node inside the tree that knows about its span.
pub struct NodeEntryMut<'a, T: 'a> {
    node: &'a mut Octnode<T>,
    span: Span,
}


impl<'a, T> NodeEntryMut<'a, T> {
    /// Returns `true` if the referenced node is a leaf node
    pub fn is_leaf(&self) -> bool {
        match *self.node {
            Octnode::Leaf(_) => true,
            _ => false,
        }
    }

    /// Returns the span of the referenced node
    pub fn span(&self) -> Span {
        self.span.clone()
    }

    /// If the referenced node is a leaf node and this leaf node contains a
    /// value, that value is returned; `None` otherwise.
    pub fn leaf_data<'b>(&'b mut self) -> Option<&'b mut Option<T>> {
        match *self.node {
            Octnode::Leaf(ref mut data) => Some(data),
            _ => None,
        }
    }

    /// If the referenced node `n` is *not* a leaf node, eight `NodeEntry`s
    /// referencing all eight children of `n` are returned; `None` otherwise.
    pub fn into_children(self) -> Option<ArrayVec<[Self; 8]>> {
        match *self.node {
            Octnode::SubTree(ref mut children) => {
                let start = self.span.start;
                let half_diff = (self.span.end - start) / 2.0;

                Some(children
                    .iter_mut()
                    .zip(SPLIT_SPAN_DIFF_AMOUNT)
                    .map(|(child, &diff_amount)| {
                        let child_start = start + half_diff.mul_element_wise(diff_amount);
                        let child_end = child_start + half_diff;

                        NodeEntryMut {
                            node: child,
                            span: child_start .. child_end,
                        }
                    })
                    .collect()
                )
            },
            _ => None,
        }
    }

    /// Splits the `self` leaf into eight children and returns the data of
    /// the split leaf. *Note*: the referenced node has to be a leaf!
    pub fn split(&mut self) -> Option<T> {
        use std::iter;

        assert!(self.is_leaf());


        let out = match *self.node {
            Octnode::Leaf(ref mut data) => data.take(),
            _ => unreachable!(),
        };

        let eight_nones = iter::repeat(()).map(|_| Octnode::Leaf(None));
        let empty_children: ArrayVec<_> = eight_nones.collect();

        *self.node = Octnode::SubTree(
            Box::new(empty_children.into_inner().ok().unwrap())
        );

        out
    }
}
