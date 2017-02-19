use util::iter;
use std::ops::Index;

/// A lookup table for regular 3D grids. Every cell in the grid contains one
/// value.
///
/// The table is structured in a way such that lookup tables for all children
/// in an octree can easily be obtained. All data for one child is saved
/// consecutive in memory, like so:
///
/// |~~~~~~~~~ -x ~~~~~~~~~||~~~~~~~~~ +x ~~~~~~~~~|
/// |~~~ -y ~~~||~~~ +y ~~~||~~~ -y ~~~||~~~ -y ~~~|
/// | -z || +z || -z || +z || -z || +z || -z || +z |
///    0     1     2     3     4     5     6     7
pub struct GridTable<T> {
    size: u32,
    data: Vec<T>,
}

impl<T> GridTable<T> {
    pub fn fill_with<F>(size: u32, mut filler: F) -> Self
        where F: FnMut(u32, u32, u32) -> T
    {
        assert!(size >= 2);

        let mut data = Vec::with_capacity((size as usize).pow(3));

        for (x, y, z) in iter::cube(size) {
            data.push(filler(x, y, z));
        }

        GridTable {
            size: size,
            data: data,
        }
    }
}

impl<T> Index<(u32, u32, u32)> for GridTable<T> {
    type Output = T;

    fn index(&self, (x, y, z): (u32, u32, u32)) -> &Self::Output {
        assert!(x < self.size);
        assert!(y < self.size);
        assert!(z < self.size);

        let idx =
            (x as usize) * (self.size as usize).pow(2)
            + (y as usize * self.size as usize)
            + z as usize;

        &self.data[idx]
    }
}
