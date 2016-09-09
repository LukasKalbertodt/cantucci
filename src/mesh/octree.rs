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
    pub fn leaf_data(&'a mut self) -> Option<&'a mut Option<T>> {
        match *self.node {
            Octnode::Leaf(ref mut data) => Some(data),
            _ => None,
        }
    }

    /// If the referenced node `n` is *not* a leaf node, eight `NodeEntry`s
    /// referencing all eight children of `n` are returned; `None` otherwise.
    pub fn into_children(self) -> Option<[Self; 8]> {
        match *self.node {
            Octnode::SubTree(ref mut children) => {
                // TODO: fix span
                let mut iter = children.iter_mut();
                Some([
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                    NodeEntryMut { node: iter.next().unwrap(), span: self.span.clone() },
                ])
            },
            _ => None,
        }
    }
}
