//! Types and functions dealing with positions within the source code
//!

use std::ops::{self, Add, Sub};
use std::cmp::{min, max};
use std::fmt;

// Helps implementing basic operators, like `Add` and `Sub`
macro_rules! impl_math {
    ($ty_name:ident, $trait_name:ident, $fun_name:ident) => {
        impl $trait_name for $ty_name {
            type Output = $ty_name;

            fn $fun_name(self, rhs: $ty_name) -> $ty_name {
                $ty_name($trait_name::$fun_name(self.0, rhs.0))
            }
        }
    }
}

// ----------------------------------------------------------------------------
/// Type do index one byte in a source code. It should be rather small, since
/// it's used a lot.
pub type SrcOffset = u32;

/// Position within source specified by byte offset. This is not equal to
/// `CharPos` thanks to UTF-8 and multibyte chars. This type always represents
/// positions relative to the whole codemap.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct BytePos(pub SrcOffset);

impl_math!(BytePos, Add, add);
impl_math!(BytePos, Sub, sub);


// ----------------------------------------------------------------------------
/// A region within the source specified by first and last byte offset. `lo`
/// byte is included in the span, `hi` byte is excluded.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Low byte, inclusive
    pub lo: BytePos,
    /// High byte, exclusive
    pub hi: BytePos,
}

impl Span {
    /// Creates a span that points to a single char
    pub fn single(pos: BytePos) -> Span {
        Span { lo: pos, hi: pos + BytePos(1) }
    }

    /// Crates an empty span (points in between to chars)
    pub fn empty_at(pos: BytePos) -> Span {
        Span { lo: pos, hi: pos }
    }

    /// Creates a span from a lo and hi (shorter than struct constructor
    /// syntax)
    pub fn new(lo: BytePos, hi: BytePos) -> Span {
        Span { lo: lo, hi: hi }
    }

    /// Creates a span from a tuple
    pub fn from_pair((lo, hi): (BytePos, BytePos)) -> Span {
        Span { lo: lo, hi: hi }
    }

    /// Creates a dummy span. Should be used with caution.
    pub fn dummy() -> Span {
        Span { lo: BytePos(1), hi: BytePos(0) }
    }

    /// Checks if the this span is a dummy span
    pub fn is_dummy(&self) -> bool {
        self.lo.0 == 1 && self.hi.0 == 0
    }

    /// Checks if the span is empty
    pub fn is_empty(&self) -> bool {
        self.lo == self.hi
    }

    /// Returns the length (number of bytes) of the span or 0 if it's a dummy
    /// span
    pub fn len(&self) -> Option<SrcOffset> {
        if self.is_dummy() {
            None
        } else {
            Some((self.hi - self.lo).0)
        }
    }

    /// Returns the smallest span which encloses both given spans
    ///
    /// If one of given spans is a dummy span, it is ignored and the other span
    /// is returned. If both spans are dummy spans, a dummy span is returned.
    pub fn hull(&self, other: &Self) -> Span {
        if self.is_dummy() {
            *other
        } else if other.is_dummy() {
            *self
        } else {
            Span {
                lo: min(self.lo, other.lo),
                hi: max(self.hi, other.hi),
            }
        }
    }

    /// Checks if this span contains another span. A dummy span never contains
    /// any other span and is never contained in another span.
    pub fn contains(&self, other: Self) -> bool {
        !self.is_dummy()
            && !other.is_dummy()
            && self.lo <= other.lo
            && self.hi >= other.hi
    }

    /// Returns a range for indexing strings and vectors
    pub fn into_range(self) -> ops::Range<usize> {
        ops::Range {
            start: self.lo.0 as usize,
            end: self.hi.0 as usize,
        }
    }

    pub fn spanning<T>(self, data: T) -> Spanned<T> {
        Spanned {
            data: data,
            span: self,
        }
    }
}

// custom `Debug` impl to shorten debug output and improve readability
impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@({}, {})", self.lo.0, self.hi.0)
    }
}

pub struct Spanned<T> {
    pub data: T,
    pub span: Span,
}


// ----------------------------------------------------------------------------
/// Represents a line index.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct LineIdx(pub SrcOffset);

impl fmt::Display for LineIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0 + 1).fmt(f)
    }
}

/// Represents a column index.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct ColIdx(pub SrcOffset);

impl_math!(LineIdx, Add, add);
impl_math!(LineIdx, Sub, sub);
impl_math!(ColIdx, Add, add);
impl_math!(ColIdx, Sub, sub);


/// Location within one file specified by line and column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Loc {
    pub line: LineIdx,
    pub col: ColIdx,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_spans() {
        use super::Span;

        let s = Span::new(BytePos(3), BytePos(10));
        assert_eq!(s, Span { lo: BytePos(3), hi: BytePos(10) });
        assert_eq!(s, Span::from_pair((BytePos(3), BytePos(10))));
        assert_eq!(s.len(), Some(7));
        assert!(!s.is_dummy());

        assert!(s.contains(Span::new(BytePos(4), BytePos(9))));
        assert!(s.contains(Span::new(BytePos(3), BytePos(10))));
        assert!(s.contains(Span::new(BytePos(5), BytePos(10))));
        assert!(s.contains(Span::new(BytePos(3), BytePos(8))));
        assert!(!s.contains(Span::new(BytePos(2), BytePos(8))));
        assert!(!s.contains(Span::new(BytePos(3), BytePos(11))));
        assert!(!s.contains(Span::new(BytePos(1), BytePos(12))));
        assert!(!s.contains(Span::dummy()));

        assert_eq!(Span::single(BytePos(15)), Span::new(BytePos(15), BytePos(16)));
    }

    #[test]
    fn dummy_spans() {
        use super::Span;

        let d = Span::dummy();
        assert_eq!(d, Span::dummy());
        assert_eq!(d.len(), None);
        assert!(d.is_dummy());

        assert!(!d.contains(Span::new(BytePos(4), BytePos(9))));
        assert!(!d.contains(Span::new(BytePos(3), BytePos(0))));
        assert!(!d.contains(Span::dummy()));
    }

    #[test]
    fn span_hulls() {
        use super::Span;

        let d = Span::dummy();
        let a = Span::new(BytePos(1), BytePos(5));
        let b = Span::new(BytePos(3), BytePos(7));
        let c = Span::new(BytePos(7), BytePos(9));

        assert_eq!(a.hull(&d), a);
        assert_eq!(d.hull(&a), a);
        assert_eq!(d.hull(&d), d);

        assert_eq!(a.hull(&b), Span::new(BytePos(1), BytePos(7)));
        assert_eq!(a.hull(&c), Span::new(BytePos(1), BytePos(9)));
        assert_eq!(b.hull(&c), Span::new(BytePos(3), BytePos(9)));
    }
}
