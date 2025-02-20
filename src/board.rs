use std::fmt;

/// u32 board row format:
/// most significant bit <- -> least significant bit
/// [14 bits meta data] - [18 bits cell data]
/// meta data [10 bits empty -- move_count 4 bits -- miniboard_status 2 bits]
pub mod flag {
    pub const MINIBOARD_STATUS:     u32     = 20;
    pub const STATUS_BIT_SIZE:      u32     = 0b11;
    pub const STATUS_CONTESTABLE:     u8      = 0;
    pub const STATUS_X_WIN:         u8      = 1;
    pub const STATUS_O_WIN:         u8      = 2;
    pub const STATUS_DRAW:          u8      = 3;

    pub const MINIBOARD_MOVE_COUNT: u32     = 22;
    pub const MOVE_COUNT_BIT_SIZE:  u32     = 0b1111;

    pub const NEW_GAME:             u8      = 255;
}

#[derive(Debug)]
pub struct Board {
    pub main_board: [u32; 9],
    prev_move: (usize, usize),
}

impl Board {

    pub fn new() -> Self {
        Board { main_board: [0; 9], prev_move: (flag::NEW_GAME as usize, flag::NEW_GAME as usize) }
    }

    /* Cell Operations */
    pub const fn get_cell(&self, row: usize, column: usize) -> u32 {
        let offset = column * 2;
        let mask = 0b11 << offset;
        
        (self.main_board[row] & mask) >> offset
    }

    pub const fn set_cell(&mut self, row: usize, column: usize, xoshape: u32) {
        // clear bits
        let offset = column * 2;
        let mask = 0b11 << offset;
        self.main_board[row] &= !mask; 
        
        // set bits
        let mask = xoshape << offset;
        self.main_board[row] |= mask;
    }

    fn _check_cell(&self, row: usize, column: usize, shape: u32) -> bool {
        let offset = column * 2;
        let mask = 0b11 << offset;

        let result = (self.main_board[row] & mask) >> offset;
        shape == result
    }

    /* Miniboard Meta Data */

    //#[inline(always)]
    pub const fn set_meta_data(&mut self, miniboard: usize, flag_pos: u32, flag_size: u32, value: u32) {
        // clear the occupying bits
        let mask = flag_size << flag_pos;
        self.main_board[miniboard] &= !mask;

        // set the cleared bits
        let mask = value << flag_pos;
        self.main_board[miniboard] |= mask;
    }

    pub const fn get_meta_data(&self, miniboard: usize, flag_pos: u32, flag_size: u32) -> u32 {
        let mask = flag_size << flag_pos;
        (self.main_board[miniboard] & mask) >> flag_pos
    }

    // Get a miniboard's metadata from its corresponding row 
    // where row n holds miniboard n's metadata
    /* Row-wise operations  */

    /// Create a mask where there are only 1s over the 18 bits for the row
    /// with the remaining 14 being 0s to zero-out the row metadata.
    fn get_row_cells(&self, row: usize) -> u32 {
        let mask = (1 << 18) - 1;
        self.main_board[row] & mask 
    }
    
    //pub fn _get_row_metadata(&self, row: usize) -> u16 {
    //    (self.main_board[row] >> 18).try_into().unwrap()
    //}

    //* Moves *//

    /// Applies the move but doesn't check validity with `self.is_valid_move`, or apply 
    /// minboard status checks
    pub const fn do_move(&mut self, row: usize, column: usize, xoshape: u32) {
        self.set_cell(row, column, xoshape);
        self.prev_move = (row, column);
        let miniboard = Self::move_miniboard(row, column);
        let move_count = self.get_meta_data(miniboard, flag::MINIBOARD_MOVE_COUNT, flag::MOVE_COUNT_BIT_SIZE);
        self.set_meta_data(miniboard, flag::MINIBOARD_MOVE_COUNT, flag::MOVE_COUNT_BIT_SIZE, move_count + 1); 
    }

    /// Returns the miniboard that move is in
    pub const fn move_miniboard(row: usize, column: usize) -> usize {

        //let mb = (column / 3) + (row / 3) * 3;
        //println!("{row}, {column} -> {mb}");

        (column / 3) + (row / 3) * 3  
    }

    //TODO: find better name than "corresponding"
    
    /// Returns the miniboard number that the next move should be played in
    pub const fn move_corresponding_miniboard(row: usize, column: usize) -> usize {
        (column % 3) + (row % 3) * 3
    }

    /// validate moves by ensuring that invalidity if:
    /// 1) cell is occupied
    /// 2) miniboard is 'uncontestable' i.e. won by X or O, or drawn
    /// 3) miniboard coords don't correspond to previous move
    /// 4) exception: corresponding board is uncontestable -- then we play anywhere else where a cells is
    ///     empty, and it's board is contestable
    pub fn is_valid_move(&self, row: usize, column: usize) -> bool {

        let cell = self.get_cell(row, column); 
        let miniboard = Self::move_miniboard(row, column);
        let miniboard_status = self.get_meta_data(miniboard, flag::MINIBOARD_STATUS, flag::STATUS_BIT_SIZE);

        if self.prev_move == (flag::NEW_GAME as usize, flag::NEW_GAME as usize) {
            println!("valid {:?}", (row, column));
            return true;
        }

        if cell != 0 {
            return false;
        }
        
        let (prev_row, prev_column) = self.prev_move;
        let corresponding_miniboard = Self::move_corresponding_miniboard(prev_row, prev_column);
        let corresponding_miniboard_status = self.get_meta_data(corresponding_miniboard, flag::MINIBOARD_STATUS, flag::STATUS_BIT_SIZE);

        if corresponding_miniboard_status != flag::STATUS_CONTESTABLE as u32 &&
        miniboard_status == flag::STATUS_CONTESTABLE as u32 { 
            return true; 
        }

        if miniboard_status != flag::STATUS_CONTESTABLE as u32 {
            return false;
        }

        if miniboard != corresponding_miniboard {
            return false;
        }

        println!("valid {:?}", (row, column));
        true
    }

    pub const fn reset(&mut self) {
        self.prev_move = (flag::NEW_GAME as usize, flag::NEW_GAME as usize);
        self.main_board = [0; 9];
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // print the board with Xs and Os
        if f.alternate() {
            for row in 0..9 {
                for column in 0..9 { 
                    let cell = self.get_cell(row, column);
                    write!(f, "{row},{column} ")?;
                    if cell == 1 {
                    } else if cell == 2 {
                        //write!(f, "O ")?;
                    } else {
                        //write!(f, "_ ")?;
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

            //for row in 0..9 {
            //    for column in 0..9 { 
            //        let cell = self.get_cell(row, column);
            //        if cell == 1 {
            //            write!(f, "X ")?;
            //        } else if cell == 2 {
            //            write!(f, "O ")?;
            //        } else {
            //            write!(f, "_ ")?;
            //        }
            //        if (column + 1) % 3 == 0 {
            //            write!(f, "| ")?;
            //        } 
            //    } 
            //    writeln!(f)?;
            //    if (row + 1) % 3 == 0 {
            //        writeln!(f, "— — — — — — — — — — — —")?;
            //    }
            //}
            //Ok(())
        } else {
            for i in 0..self.main_board.len() {
                writeln!(f, "[{i}]: cells[{:18b}]-meta[{:14b}]", self.get_row_cells(i), self.main_board[i] >> 18)?
            }
            Ok(())
        }
    }
}
