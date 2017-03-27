use super::{SrcOffset, BytePos, LineIdx, ColIdx, Loc};
use std::cell::RefCell;
use std::fmt;

/// Stores the content of a file and keeps track of some position meta data,
/// such as linebreaks.
pub struct FileMap {
    /// Original filename or dummy filename of the form "<...>"
    filename: String,

    /// The whole content of the file
    src: String,

    /// List of line beginnings. Wrapped in a `RefCell`, because `FileMap`s
    /// are shared a lot via `Rc` and this is the only thing requiring
    /// mutability. Borrowing it will never panic, because:
    /// - it's only borrowed within methods of `FileMap`
    /// - the borrow always ends with those methods
    /// - no method is called/active while another one is active (as long as
    ///   no borrowing method is calling another borrowing method)
    lines: RefCell<Vec<BytePos>>,
}

impl FileMap {
    /// Creates a new Filemap from existing buffers for the filename and
    /// content of the file.
    pub fn new<U, V>(filename: U, src: V) -> FileMap
        where U: Into<String>,
              V: Into<String>
    {
        FileMap {
            filename: filename.into(),
            src: src.into(),
            lines: RefCell::new(vec![BytePos(0)]),
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    /// Adds a new line beginning with the given BytePos to the list. Line
    /// beginnings need to be added in order!
    pub fn add_line(&self, offset: BytePos) {
        self.lines.borrow_mut().push(offset);
    }

    /// Returns the number of lines.
    pub fn num_lines(&self) -> usize {
        self.lines.borrow().len()
    }

    /// Returns the (0-based) index of the line in which the byte with the
    /// given `offset` lives.
    pub fn get_line_idx(&self, offset: BytePos) -> LineIdx {
        // If `offset` is not a line beginning, `binary_search` returns the
        // index of the next line. Hence `-1`.
        LineIdx(self.lines.borrow()
            .binary_search(&offset)
            .unwrap_or_else(|e| e - 1) as u32)
    }

    /// Returns the location of the given bytes as line and col numbers within
    /// this file.
    pub fn get_loc(&self, offset: BytePos) -> Loc {
        let line = self.get_line_idx(offset);
        let col = offset - self.lines.borrow()[line.0 as usize];

        Loc { line: line, col: ColIdx(col.0) }
    }

    /// Returns the line with the given index or `None` if it is invalid.
    pub fn get_line(&self, line: LineIdx) -> Option<&str> {
        self.lines.borrow().get(line.0 as usize).map(|&BytePos(start)| {
            let end = self.src[start as usize..]
                          .find("\n")
                          .unwrap_or(self.src.len() - start as usize);
            &self.src[start as usize .. (end + start as usize)]
        })
    }

    /// Returns the byte offset of the first symbol in `line`
    pub fn get_line_start(&self, line: LineIdx) -> Option<BytePos> {
        self.lines.borrow().get(line.0 as usize).map(|&pos| pos)
    }

    /// Searches for line endings and collects all line beginnings in the
    /// source string. It starts searching at the latest line beginning in the
    /// list so far (or at the beginning of none was added yet).
    ///
    /// Normally this is done while lexing to avoid iterating
    /// over the whole string multiple times. Mostly handy for tests.
    pub fn find_lines(&self) {
        // We can unwrap here, because the vector contains at least one element
        let last_line_so_far = self.lines.borrow().last().unwrap().0 as usize;
        for (pos, c) in self.src[last_line_so_far..].char_indices() {
            // it doesn't matter if there was a '\n' or '\r\n'
            if c == '\n' {
                let line_start = (pos + c.len_utf8()) as SrcOffset;
                self.lines.borrow_mut().push(BytePos(line_start));
            }
        }
    }
}

impl fmt::Debug for FileMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Dummy<'a>(&'a [BytePos]);
        impl<'a> fmt::Debug for Dummy<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let items = self.0.iter().map(|v| v.0).enumerate();
                f.debug_map().entries(items).finish()
            }
        }

        f.debug_struct("FileMap")
            .field("filename", &self.filename)
            .field("src", &format!("<long string> (len {})", self.src.len()))
            .field("lines", &Dummy(&self.lines.borrow()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_lookup() {
        let map = FileMap::new("<dummy>", "foo\r\nbär\nbaz");
        map.find_lines();

        // "foo\r\n" is 5 bytes. "bär\n" is 5 bytes, too.
        assert_eq!(*map.lines.borrow(),
            vec![BytePos(0), BytePos(5), BytePos(10)]
        );

        macro_rules! is_at {
            ($offset:expr => [$line:expr, $col:expr]) => {
                assert_eq!(map.get_loc(BytePos($offset)), Loc {
                    line: LineIdx($line), col: ColIdx($col)
                });
            }
        }

        is_at!(0 => [0, 0]);
        is_at!(1 => [0, 1]);
        is_at!(4 => [0, 4]);
        is_at!(5 => [1, 0]);
        is_at!(9 => [1, 4]);
        is_at!(10 => [2, 0]);
        is_at!(12 => [2, 2]);
    }
}
