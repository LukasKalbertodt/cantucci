
pub fn cube(size: u32) -> CubeIter {
    CubeIter {
        size: size,
        x: 0,
        y: 0,
        z: 0,
    }
}

pub struct CubeIter {
    size: u32,
    x: u32,
    y: u32,
    z: u32,
}

impl Iterator for CubeIter {
    type Item = (u32, u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x == self.size {
            None
        } else {
            let out = (self.x, self.y, self.z);

            // Handle "overflow"
            self.z += 1;
            if self.z == self.size {
                self.y += 1;
                self.z = 0;

                if self.y == self.size {
                    self.x += 1;
                    self.y = 0;
                }
            }

            Some(out)
        }
    }
}

pub fn square(size: u32) -> SquareIter {
    SquareIter {
        size: size,
        x: 0,
        y: 0,
    }
}

pub struct SquareIter {
    size: u32,
    x: u32,
    y: u32,
}

impl Iterator for SquareIter {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x == self.size {
            None
        } else {
            let out = (self.x, self.y);

            // Handle "overflow"
            self.y += 1;
            if self.y == self.size {
                self.x += 1;
                self.y = 0;
            }

            Some(out)
        }
    }
}
