pub struct ValidMoveIterator {
    pub bitfield: u128,
}

impl ValidMoveIterator {
    pub fn new(bitfield: u128) -> Self {
        Self { bitfield }
    }
}

impl Iterator for ValidMoveIterator {
    type Item = (u8, u8); 

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitfield == 0 {
            return None;
        }

        let offset = self.bitfield.trailing_zeros() as u8;
        if offset >= 81 {
            return None;
        }

        // clear the bit and return the move (row, col)
        self.bitfield ^= 1 << offset;
        Some((offset / 9, offset % 9))
    }
}

