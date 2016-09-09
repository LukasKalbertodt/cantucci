use std::ops::Range;
use core::math::*;
use arrayvec::ArrayVec;

/// A box in three dimensional space that is represented by one octree node
pub type Span = Range<Point3<f64>>;

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

    /// Returns an iterator over *im*mutable nodes
    pub fn iter(&self) -> Iter<T> {
        Iter {
            to_visit: vec![
                NodeEntry {
                    node: &self.root,
                    span: self.span.clone(),
                }
            ]
        }
    }

    /// Returns an iterator over *mutable* nodes
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            to_visit: vec![&mut self.root],
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
// impl<'a, T: Clone> IntoIterator for &'a mut Octree<T> {
//     type Item = NodeEntry<'a, T>;
//     type IntoIter = IterMut<'a, T>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter_mut()
//     }
// }

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

/// A reference to a node inside the tree that knows about its span.
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

    /// If the referenced node `n` is *not+ a leaf node, eight `NodeEntry`s
    /// referencing all eight children of `n` are returned; `None` otherwise.
    pub fn children(&self) -> Option<[Self; 8]> {
        match *self.node {
            Octnode::SubTree(ref children) => {
                // TODO: fix span
                Some([
                    NodeEntry { node: &children[0], span: self.span.clone() },
                    NodeEntry { node: &children[1], span: self.span.clone() },
                    NodeEntry { node: &children[2], span: self.span.clone() },
                    NodeEntry { node: &children[3], span: self.span.clone() },
                    NodeEntry { node: &children[4], span: self.span.clone() },
                    NodeEntry { node: &children[5], span: self.span.clone() },
                    NodeEntry { node: &children[6], span: self.span.clone() },
                    NodeEntry { node: &children[7], span: self.span.clone() },
                ])
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

/// An iterator over *mutable* references of nodes
pub struct IterMut<'a, T: 'a> {
    to_visit: Vec<&'a mut Octnode<T>>,
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut Option<T>;

    fn next<'b>(&'b mut self) -> Option<&'a mut Option<T>> {
        let next: &'a mut Octnode<T> = self.to_visit.pop().unwrap();

        match *next {
            Octnode::Leaf(ref mut inner) => Some(inner),
            _ => None,
        }
    }
}
