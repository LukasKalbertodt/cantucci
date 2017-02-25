use arrayvec::ArrayVec;

use super::{NodeEntry, NodeEntryMut, Octree, Span};


/// An iterator over *im*mutable references of nodes
pub struct Iter<'a, T: 'a> {
    to_visit: Vec<NodeEntry<'a, T>>,
}

impl<'a, T> Iter<'a, T> {
    pub fn new(tree: &'a Octree<T>) -> Self {
        Iter {
            to_visit: vec![
                NodeEntry {
                    node: &tree.root,
                    span: tree.span(),
                }
            ]
        }
    }
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


/// The mutable iterator produces elements of this type.
///
/// The mutable iterator can't produce `NodeEntryMut`, because multiple mutable
/// references could be obtained which could lead to iterator invalidation or
/// worse (which is of course prevented by the borrow checker).
pub enum IterElemMut<'a, T: 'a> {
    Leaf {
        span: Span,
        data: &'a mut Option<T>,
    },
    Inner(Span),
}

impl<'a, T> IterElemMut<'a, T> {
    pub fn into_leaf(self) -> Option<(Span, &'a mut Option<T>)> {
        match self {
            IterElemMut::Leaf { span, data } => Some((span, data)),
            _ => None,
        }
    }

    pub fn span(&self) -> Span {
        match *self {
            IterElemMut::Leaf { ref span, .. } => span,
            IterElemMut::Inner(ref span) => span,
        }.clone()
    }
}


/// An iterator over mutable references of nodes
pub struct IterMut<'a, T: 'a> {
    to_visit: Vec<NodeEntryMut<'a, T>>,
}

impl<'a, T> IterMut<'a, T> {
    pub fn new(tree: &'a mut Octree<T>) -> Self {
        IterMut {
            to_visit: vec![
                NodeEntryMut {
                    span: tree.span(),
                    node: &mut tree.root,
                }
            ]
        }
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = IterElemMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.to_visit.pop().map(|next| {
            if next.is_leaf() {
                IterElemMut::Leaf {
                    span: next.span(),
                    data: next.into_leaf_data().unwrap(),
                }
            } else {
                let span = next.span();
                let children = next.into_children().unwrap();
                self.to_visit.extend(ArrayVec::from(children));
                IterElemMut::Inner(span)
            }
        })
    }
}
