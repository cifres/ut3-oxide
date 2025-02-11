use std::fmt;

#[derive(Debug)]
pub struct Board {
    pub main_board: [u32; 9],
}

pub fn hello() {
   let n = 0u8;
    //let n = [1u16, false];
    println!("hey from board {n}, with size {}", size_of_val(&n));
}

impl Board {
    pub fn new() -> Self {
        Board { main_board: [0; 9] }
    }

    pub fn get_cell(&self, row: usize, column: usize) -> u32 {
        let offset = column * 2;
        let mask = 0b11 << offset;
        
        (self.main_board[row] & mask) >> offset
    }

    pub fn set_cell(&mut self, row: usize, column: usize, shape: u32) {
        // clear bits
        let offset = column * 2;
        let mask = 0b11 << offset;
        self.main_board[row] &= !mask;

        // set bits
        let mask = shape << offset;
        self.main_board[row] |= mask;
    }

    pub fn _check_cell(&self, row: usize, column: usize, shape: u32) -> bool {
        let offset = column * 2;
        let mask = 0b11 << offset;

        let result = (self.main_board[row] & mask) >> offset;
        shape == result
    }

    /* Row-wise operations  */

    /// Create a mask where there are only 1s over the 18 bits for the row
    /// with the remaining 14 being 0s to zero-out the row metadata.
    pub fn get_row_cells(&self, row: usize) -> u32 {
        let mask = (1 << 18) - 1;
        self.main_board[row] & mask 
    }
    
    pub fn get_row_metadata(&self, row: usize) -> u16 {
        (self.main_board[row] >> 18).try_into().unwrap()
    }


}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            for row in 0..9 {
                for column in 0..9 { 
                    let cell = self.get_cell(row, column);
                    if cell == 1 {
                        write!(f, "X ")?;
                    } else if cell == 2 {
                        write!(f, "O ")?;
                    } else {
                        write!(f, "_ ")?;
                    }
                    if (column + 1) % 3 == 0 {
                        write!(f, "| ")?;
                    } 
                } 
                writeln!(f)?;
                if (row + 1) % 3 == 0 {
                    writeln!(f, "— — — — — — — — — — — —")?;
                }
            }
            Ok(())
        } else {
            for i in 0..self.main_board.len() {
                writeln!(f, "[{i}]: cells[{:18b}]-meta[{:14b}]", self.get_row_cells(i), self.main_board[i] >> 18)?
            }
            Ok(())
        }
    }
}
