use std::{array::IntoIter, ops::Range};

use cgmath::Point3;


mod iter;
// mod debug_view;

pub use self::iter::{Iter, IterElemMut, IterMut};
// pub use self::debug_view::DebugView;

/// A box in three dimensional space that is represented by one octree node
pub type Span = Range<Point3<f32>>;

pub trait SpanExt {
    fn center(&self) -> Point3<f32>;
    fn contains(&self, p: Point3<f32>) -> bool;
}

impl SpanExt for Span {
    fn center(&self) -> Point3<f32> {
        self.start + (self.end - self.start) / 2.0
    }

    fn contains(&self, p: Point3<f32>) -> bool {
        let s = self.start;
        let e = self.end;

        s.x <= p.x && s.y <= p.y && s.z <= p.z
            && p.x < e.x && p.y < e.y && p.z < e.z
    }
}

/// Recursively partitions three dimensional space into eight octants. In this
/// simple implementation all octants have the same size.
///
/// In this application it's used to store the representation of the octant in
/// order to allow different resolutions in different parts of space.
pub struct Octree<L, I> {
    span: Range<Point3<f32>>,
    root: Octnode<L, I>,
}

impl<L, I> Octree<L, I> {
    /// Creates an empty octree representing the given span
    pub fn spanning(span: Span) -> Self {
        Octree {
            span,
            root: Octnode::Leaf(None),
        }
    }

    /// Returns the span of the root element
    pub fn span(&self) -> Span {
        self.span.clone()
    }

    /// Returns the root node immutably.
    pub fn root(&self) -> NodeEntry<L, I> {
        NodeEntry {
            node: &self.root,
            span: self.span(),
        }
    }

    /// Returns the root node mutably.
    pub fn root_mut(&mut self) -> NodeEntryMut<L, I> {
        NodeEntryMut {
            span: self.span(),
            node: &mut self.root,
        }
    }

    /// Returns the leaf node which contains the point `p`.
    pub fn leaf_around_mut(&mut self, p: Point3<f32>) -> Option<NodeEntryMut<L, I>> {
        let mut node = self.root_mut();
        if !node.span().contains(p) {
            return None;
        }
        loop {
            if node.is_leaf() {
                return Some(node);
            } else {
                node = IntoIter::new(node.into_children().unwrap())
                    .find(|c| c.span().contains(p))
                    .unwrap();
            }
        }
    }

    /// Returns an iterator over *im*mutable nodes
    pub fn iter(&self) -> Iter<L, I> {
        Iter::new(self)
    }

    /// Returns an iterator over mutable nodes
    pub fn iter_mut(&mut self) -> IterMut<L, I> {
        IterMut::new(self)
    }
}

// `IntoIterator` impls for convenience
impl<'a, L, I> IntoIterator for &'a Octree<L, I> {
    type Item = NodeEntry<'a, L, I>;
    type IntoIter = Iter<'a, L, I>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, L, I> IntoIterator for &'a mut Octree<L, I> {
    type Item = IterElemMut<'a, L, I>;
    type IntoIter = IterMut<'a, L, I>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// One node of the octree. This type is implementation detail of the data
/// structure and isn't usually exposed to the user.
#[derive(Debug, PartialEq, Eq)]
enum Octnode<L, I> {
    /// At this point, space is not further subdivided
    Leaf(Option<L>),

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
    SubTree {
        children: Box<[Octnode<L, I>; 8]>,
        data: Option<I>,
    }
}



// ===========================================================================


/// An *im*mutable reference to a node inside the tree that knows about its
/// span.
#[derive(Debug, Clone)]
pub struct NodeEntry<'a, L: 'a, I: 'a> {
    node: &'a Octnode<L, I>,
    span: Span,
}

impl<'a, L, I> NodeEntry<'a, L, I> {
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
    pub fn leaf_data(&self) -> Option<&'a L> {
        match *self.node {
            Octnode::Leaf(ref data) => data.as_ref(),
            _ => None,
        }
    }

    /// If the referenced node `n` is *not* a leaf node, eight `NodeEntry`s
    /// referencing all eight children of `n` are returned; `None` otherwise.
    pub fn children(&self) -> Option<[Self; 8]> {
        match *self.node {
            Octnode::SubTree { ref children, ..} => {
                let spans = create_spans(self.span());
                let out = [0, 1, 2, 3, 4, 5, 6, 7].map(|i| {
                    NodeEntry { node: &children[i], span: spans[i].clone() }
                });

                Some(out)
            },
            _ => None,
        }
    }
}

// ===========================================================================

// ===========================================================================

/// A *mutable* reference to a node inside the tree that knows about its span.
#[derive(Debug)]
pub struct NodeEntryMut<'a, L: 'a, I: 'a> {
    node: &'a mut Octnode<L, I>,
    span: Span,
}


impl<'a, L, I> NodeEntryMut<'a, L, I> {
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
    pub fn leaf_data(&self) -> Option<&Option<L>> {
        // Warning: you can't return `&'a` here. Don't waste time by
        // trying! See:
        // http://stackoverflow.com/q/42397056/2408867
        match *self.node {
            Octnode::Leaf(ref data) => Some(data),
            _ => None,
        }
    }

    /// If the referenced node is a leaf node and this leaf node contains a
    /// value, that value is returned; `None` otherwise.
    pub fn leaf_data_mut(&mut self) -> Option<&mut Option<L>> {
        // Warning: you can't return `&'a mut` here. Don't waste time by
        // trying! See:
        // http://stackoverflow.com/q/42397056/2408867
        match *self.node {
            Octnode::Leaf(ref mut data) => Some(data),
            _ => None,
        }
    }

    /// If the referenced node is a leaf node and this leaf node contains a
    /// value, that value is returned; `None` otherwise.
    pub fn into_leaf_data(self) -> Option<&'a mut Option<L>> {
        match *self.node {
            Octnode::Leaf(ref mut data) => Some(data),
            _ => None,
        }
    }

    pub fn into_inner_parts(self) -> Option<(&'a mut Option<I>, [Self; 8])> {
        match self.node {
            Octnode::SubTree { children, data } => {
                let spans = create_spans(self.span);
                let mut children = children.iter_mut();
                let children = [0, 1, 2, 3, 4, 5, 6, 7].map(|i| {
                    NodeEntryMut { node: children.next().unwrap(), span: spans[i].clone() }
                });

                Some((data, children))
            },
            _ => None,
        }
    }

    /// If the referenced node `n` is *not* a leaf node, eight `NodeEntry`s
    /// referencing all eight children of `n` are returned; `None` otherwise.
    ///
    /// This methods takes `self` because of borrowck limitations.
    pub fn into_children(self) -> Option<[Self; 8]> {
        // Warning: you can't change the `self` into a `&mut self` without
        // shortening the output lifetime! Don't waste time by trying! See:
        // http://stackoverflow.com/q/42397056/2408867
        match self.node {
            Octnode::SubTree { children, .. } => {
                let spans = create_spans(self.span);
                let mut children = children.iter_mut();
                let out = [0, 1, 2, 3, 4, 5, 6, 7].map(|i| {
                    NodeEntryMut { node: children.next().unwrap(), span: spans[i].clone() }
                });

                Some(out)
            },
            _ => None,
        }
    }

    /// Splits the `self` leaf into eight children and returns the data of
    /// the split leaf. *Note*: the referenced node has to be a leaf!
    pub fn split(&mut self, data: Option<I>) -> Option<L> {
        assert!(self.is_leaf());

        let out = match *self.node {
            Octnode::Leaf(ref mut data) => data.take(),
            _ => unreachable!(),
        };

        *self.node = Octnode::SubTree {
            children: Box::new([Octnode::Leaf(None); 8]),
            data,
        };

        out
    }
}

/// Creates 8 equally sized children spans of a passed parent span. The spans are defined
/// in a way that will be no gaps between them due to floating point precision errors.
pub fn create_spans(parent_span: Range<Point3<f32>>) -> [Range<Point3<f32>>; 8] {
    let start = parent_span.start;
    let end = parent_span.end;
    let center = start + (parent_span.end - start) / 2.0;
    [
        start .. center,
        Point3 { z: center.z, .. start } .. Point3 { z: end.z, .. center },
        Point3 { y: center.y, .. start } .. Point3 { y: end.y, .. center },
        Point3 { x: start.x, .. center } .. Point3 { x: center.x, .. end },
        Point3 { x: center.x, .. start } .. Point3 { x: end.x, .. center },
        Point3 { y: start.y, .. center } .. Point3 { y: center.y, .. end },
        Point3 { z: start.z, .. center } .. Point3 { z: center.z, .. end },
        center .. end,
    ]
}
