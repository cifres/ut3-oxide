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

pub struct MiniboardStatusesIterator {
    pub bitfield: u32,
    offset: u8,
}

impl MiniboardStatusesIterator {
    pub fn new(bitfield: u32) -> Self {
        Self {
            bitfield,
            offset: 0,
        }
    }
}

impl Iterator for MiniboardStatusesIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == 9 {
            return None;
        }

        let offset = self.offset * 2;

        let mask = 0b11 << offset;
        let status = ((self.bitfield & mask) >> offset) as u8;
        self.offset += 1;

        Some(status)
    }
}
