
/// Recursively partitions three dimensional space into eight octants. In this
/// simple implementation all octants have the same size.
///
/// In this application it's used to save the representation of the octant in
/// order to allow different resolutions in different parts of space.
pub enum Octree<T> {
    /// At this point, space is not further subdivided
    Leaf(T),

    /// Space is divided into eight octants, saved in this array. The order of
    /// the octants is as follows:
    ///
    /// - (-x, -y, -z)
    /// - (-x, -y, +z)
    /// - (-x, +y, -z)
    /// - (-x, +y, +z)
    /// - (+x, -y, -z)
    /// - (+x, -y, +z)
    /// - (+x, +y, -z)
    /// - (+x, +y, +z)
    SubTree(Box<[Octree<T>; 8]>),
}
