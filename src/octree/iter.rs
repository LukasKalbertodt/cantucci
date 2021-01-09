use super::{NodeEntry, NodeEntryMut, Octree, Span};
use std::array::IntoIter;


/// An iterator over *im*mutable references of nodes
pub struct Iter<'a, L: 'a, I: 'a> {
    to_visit: Vec<NodeEntry<'a, L, I>>,
}

impl<'a, L, I> Iter<'a, L, I> {
    pub fn new(tree: &'a Octree<L, I>) -> Self {
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

impl<'a, L: 'a, I: 'a> Iterator for Iter<'a, L, I> {
    type Item = NodeEntry<'a, L, I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.to_visit.pop().map(|next| {
            if let Some(children) = next.children() {
                self.to_visit.extend(IntoIter::new(children));
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
pub enum IterElemMut<'a, L: 'a, I: 'a> {
    Leaf {
        span: Span,
        data: &'a mut Option<L>,
    },
    Inner {
        span: Span,
        data: &'a mut Option<I>,
    },
}

impl<'a, L, I> IterElemMut<'a, L, I> {
    pub fn into_leaf(self) -> Option<(Span, &'a mut Option<L>)> {
        match self {
            IterElemMut::Leaf { span, data } => Some((span, data)),
            _ => None,
        }
    }

    pub fn span(&self) -> Span {
        match *self {
            IterElemMut::Leaf { ref span, .. } => span,
            IterElemMut::Inner { ref span, .. } => span,
        }.clone()
    }
}


/// An iterator over mutable references of nodes
pub struct IterMut<'a, L: 'a, I: 'a> {
    to_visit: Vec<NodeEntryMut<'a, L, I>>,
}

impl<'a, L, I> IterMut<'a, L, I> {
    pub fn new(tree: &'a mut Octree<L, I>) -> Self {
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

impl<'a, L: 'a, I: 'a> Iterator for IterMut<'a, L, I> {
    type Item = IterElemMut<'a, L, I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.to_visit.pop().map(|next| {
            if next.is_leaf() {
                IterElemMut::Leaf {
                    span: next.span(),
                    data: next.into_leaf_data().unwrap(),
                }
            } else {
                let span = next.span();
                let (data, children) = next.into_inner_parts().unwrap();
                self.to_visit.extend(IntoIter::new(children));
                IterElemMut::Inner { span, data }
            }
        })
    }
}
