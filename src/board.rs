use std::fmt::{self};

pub mod flag {
    pub const MINIBOARD_STATUS:                       u8 = 20;
    pub (in crate::board) const STATUS_BIT_SIZE:      u8 = 0b11;
    pub const STATUS_CONTESTABLE:                     u8 = 0;
    pub const STATUS_X_WIN:                           u8 = 1;
    pub const STATUS_O_WIN:                           u8 = 2;
    pub const STATUS_DRAW:                            u8 = 3;

    pub (in crate::board) const MOVE_COUNT_X:         u8 = 22;
    pub (in crate::board) const MOVE_COUNT_O:         u8 = 26;
    pub (in crate::board) const MOVE_COUNT_BIT_SIZE:  u8 = 0b1111;

    pub const NEW_GAME:                               u8 = u8::MAX;

    pub const EMPTY:                                  u8 = 0;
    pub const X_SHAPE:                                u8 = 1;
    pub const O_SHAPE:                                u8 = 2;
}

const fn build_winning_lines() -> [[(u8, u8); 3]; 8] {

    const ROW_LINE: [(u8, u8); 9] = const {
        let mut y = 0u8;
        let mut row_line = [(0, 0); 9];
        while y < 3 {
            let mut x = 0u8;
            while x < 3 {
                let i = (x + y * 3) as usize;
                row_line[i] = (y, x);
                x += 1;
            }
            y += 1;    
        }

        row_line
    };

    const COL_LINE: [(u8, u8); 9] = const {
        let mut y = 0;
        let mut col_line = [(0, 0); 9];
        while y < 9 {
            let (row, col) = ROW_LINE[y];        
            col_line[y] = (col, row);
            y += 1;
        }
        
        col_line
    };

    const DIAGONAL_LINE: [(u8, u8); 6] = [
        (0, 0), (1, 1), (2, 2),
        (0, 2), (1, 1), (2, 0)
    ];

    let mut winning_lines = [[(0, 0); 3]; 8];
    let mut y = 0;
    while y < 3 {
        let i = y * 3;
        winning_lines[y] = [ROW_LINE[i], ROW_LINE[i + 1], ROW_LINE[i + 2]]; 
        winning_lines[y + 3] = [COL_LINE[i], COL_LINE[i + 1], COL_LINE[i + 2]]; 
        y += 1;
    }

    winning_lines[6] = [DIAGONAL_LINE[0], DIAGONAL_LINE[1], DIAGONAL_LINE[2]];
    winning_lines[7] = [DIAGONAL_LINE[3], DIAGONAL_LINE[4], DIAGONAL_LINE[5]];

    winning_lines
}

const WINNING_LINES: [[(u8, u8); 3]; 8] = build_winning_lines(); 

/// `u32` board row format:
/// most significant bit <- -> least significant bit
/// `[14 bits meta data -- 18 bits cell data]`
/// `9` cells x `2` bits per cell = `18` bits 
/// meta data `10` bits empty -- move_count `4` bits -- miniboard_status `2` bits
#[derive(Debug, Clone)]
pub struct Board {
    pub main_board: [u32; 9],
    last_move: (u8, u8),    // The last valid move that was made 
}

impl Board {

    pub fn new() -> Self {
        Board { main_board: [0; 9], last_move: (flag::NEW_GAME, flag::NEW_GAME) }
    }

    /* Cell Operations */
    #[inline(always)]
    pub const fn get_cell(&self, row: u8, column: u8) -> u8 {
        let offset = column * 2;
        let mask = 0b11 << offset;
        
        ((self.main_board[row as usize] & mask) >> offset) as u8
    }

    #[inline(always)]
    pub const fn set_cell(&mut self, row: u8, column: u8, xoshape: u8) {
        // clear bits
        let row = row as usize;
        let offset = column * 2;
        let mask = 0b11 << offset;
        self.main_board[row] &= !(mask as u32); 
        
        // set bits
        let mask = (xoshape as u32) << offset;
        self.main_board[row] |= mask;
    }

    fn _check_cell(&self, row: u8, column: u8, shape: u8) -> bool {
        let offset = column * 2;
        let mask = 0b11 << offset;

        let result = (self.main_board[row as usize] & mask) >> offset;
        shape as u32 == result
    }

    /* Miniboard Meta Data */

    #[inline(always)]
    pub const fn set_meta_data(&mut self, miniboard: u8, flag_pos: u8, flag_size: u8, value: u8) {
        // clear the occupying bits
        let miniboard = miniboard as usize;
        let mask = (flag_size as u32) << flag_pos;
        self.main_board[miniboard] &= !(mask);

        // set the cleared bits
        let mask = (value as u32) << flag_pos;
        self.main_board[miniboard] |= mask;
    }

    #[inline(always)]
    pub const fn get_meta_data(&self, miniboard: u8, flag_pos: u8, flag_size: u8) -> u8 {
        assert!(miniboard < 9);

        let mask = (flag_size as u32) << flag_pos;
        ((self.main_board[miniboard as usize] & mask) >> flag_pos) as u8
    }

    #[inline(always)]
    pub fn get_status_of(&self, miniboard: u8) -> u8 {
        self.get_meta_data(miniboard, flag::MINIBOARD_STATUS, flag::STATUS_BIT_SIZE) 
    }

    #[inline(always)]
    pub fn set_status_of(&mut self, miniboard: u8, value: u8) {
        assert!(value <= flag::STATUS_DRAW);
        self.set_meta_data(miniboard, flag::MINIBOARD_STATUS, flag::STATUS_BIT_SIZE, value);
    }
    
    #[inline(always)]
    pub fn get_total_move_count_of(&self, miniboard: u8) -> u8 {
        let x_count = self.get_meta_data(miniboard, flag::MOVE_COUNT_X, flag::MOVE_COUNT_BIT_SIZE);
        let o_count = self.get_meta_data(miniboard, flag::MOVE_COUNT_O, flag::MOVE_COUNT_BIT_SIZE);
        x_count + o_count
    }

    #[inline(always)]
    pub fn get_shape_move_count_of(&self, miniboard: u8, shape: u8) -> u8 {
        // maps 1 => 22 and 2 => 26
        let shape_movecount = flag::MOVE_COUNT_X + (shape - 1) * 4;
        self.get_meta_data(miniboard, shape_movecount, flag::MOVE_COUNT_BIT_SIZE)
    }

    #[inline(always)]
    pub fn set_move_count_of(&mut self, miniboard: u8, value: u8, shape: u8) {
        // maps 1 => 22 and 2 => 26
        let shape_movecount = flag::MOVE_COUNT_X + (shape - 1) * 4;
        assert!(shape_movecount == flag::MOVE_COUNT_X || shape_movecount == flag::MOVE_COUNT_O);
        self.set_meta_data(miniboard, shape_movecount, flag::MOVE_COUNT_BIT_SIZE, value); 
    }

    /* Row-wise operations  */

    /// Create a mask where there are only 1s over the `18` bits for the u32 row
    /// with the remaining `14` being 0s to zero-out the row metadata.
    const fn get_row_cells(&self, row: u8) -> u32 {
        let mask = (1 << 18) - 1;
        self.main_board[row as usize] & mask 
    }
    
    //pub fn _get_row_metadata(&self, row: usize) -> u16 {
    //    (self.main_board[row] >> 18).try_into().unwrap()
    //}

    //* Moves *//

    /// Applies the move but doesn't check validity with `self.is_valid_move`, or apply 
    /// minboard status checks
    pub fn do_move(&mut self, row: u8, column: u8, xoshape: u8) {
        assert!(row < 9);
        assert!(column < 9);
        assert!(xoshape <= 2);
        debug_assert_ne!(xoshape, 0, "shape was 0! must be 1 or 2");

        self.set_cell(row, column, xoshape);
        self.last_move = (row, column);
        let miniboard = Self::move_miniboard(row, column);
        let move_count = self.get_shape_move_count_of(miniboard, xoshape);
        self.set_move_count_of(miniboard, move_count + 1, xoshape);

        assert!(move_count <= 9);
    }

    /// Returns the miniboard that move is in
    #[inline(always)]
    pub const fn move_miniboard(row: u8, column: u8) -> u8 {

        assert!(row < 9);
        assert!(column < 9);
        //let mb = (column / 3) + (row / 3) * 3;
        //println!("{row}, {column} -> {mb}");

        column / 3 + (row / 3) * 3  
    }

    //TODO: find better name than "corresponding"
    
    /// Returns the miniboard number that the next move should be played in
    #[inline(always)]
    pub const fn move_corresponding_miniboard(row: u8, column: u8) -> u8 {
        (column % 3) + (row % 3) * 3
    }

    /// validate moves by ensuring that invalidity if:
    /// * cell is occupied
    /// * miniboard is 'uncontestable' i.e. won by X or O, or drawn
    /// * miniboard coords don't correspond to last move
    /// * exception: corresponding board is uncontestable -- then we play anywhere else where a cells is
    ///     empty, and it's board is contestable
    pub fn is_valid_move(&self, row: u8, column: u8) -> bool {
        assert!(row < 9);
        assert!(column < 9);

        let cell = self.get_cell(row, column); 
        let miniboard = Self::move_miniboard(row, column);
        let miniboard_status = self.get_status_of(miniboard);

        if self.last_move.0 == flag::NEW_GAME && self.last_move.1 == flag::NEW_GAME {
            //println!("valid {:?}", (row, column));
            return true;
        }

        if cell != 0 {
            return false;
        }
        
        let (last_row, last_column) = self.last_move;
        let corresponding_miniboard = Self::move_corresponding_miniboard(last_row, last_column);
        let corresponding_miniboard_status = self.get_status_of(corresponding_miniboard);

        if corresponding_miniboard_status != flag::STATUS_CONTESTABLE &&
        miniboard_status == flag::STATUS_CONTESTABLE { 
            return true; 
        }

        if miniboard_status != flag::STATUS_CONTESTABLE {
            return false;
        }

        if miniboard != corresponding_miniboard {
            return false;
        }

        //println!("valid {:?}", (row, column));
        true
    }

    pub fn reset(&mut self) {
        self.last_move = (flag::NEW_GAME, flag::NEW_GAME);
        self.main_board = [0; 9];
    }

    /// Checks and calculates the miniboard's status based on any winning lines
    /// Short-circuits and stops as soon as any winning line is found
    /// Use with `get_status_of(miniboard: u8)`
    /// Returns a `bool` with `true` indicating the miniboard's status isn't `flag::STATUS_CONTESTABLE`
    /// i.e, `flag::STATUS_X_WIN`, `flag::STATUS_O_WIN`, or `flag::STATUS_DRAW` 
    pub fn calculate_miniboard_status(&mut self, miniboard: u8) -> bool {

        // Don't recheck/recalculate winning lines for an uncontestable (won/lost/drawn) miniboard
        // Remove to trigger recalculation for usage with AI do/undo move pattern
        if self.get_status_of(miniboard) != flag::STATUS_CONTESTABLE {
            println!("nocalc cached {miniboard} as state {}", self.get_status_of(miniboard));
            return true;
        }

        let last_player = self.get_cell(self.last_move.0, self.last_move.1);
        assert_ne!(last_player, flag::EMPTY);

        // early exit miniboards that cannot be won/lost/drawn yet.
        if self.get_shape_move_count_of(miniboard, last_player) < 3 {
            println!("nocalc cached {miniboard} as state {} -- movecount < 3 for {last_player}", self.get_status_of(miniboard));
            debug_assert_eq!(self.get_status_of(miniboard), flag::STATUS_CONTESTABLE);
            return false;
        }

        // movecount == 9 drawn
        // Compare winning-line pattern masked miniboard cells to X and O wonline pattern
        // E.g. match row pattern to 101010 (2, 2, 2) -> 42 
        // or 010101 (1, 1, 1) -> 21 for O and X win respectively  
        let cells = self.get_miniboard_cells(miniboard);

        for line in WINNING_LINES {
            //println!("{line:?}");
            let mut line_mask = 0u32;
            let mut xo_wonline = 0u32;

            // Create a mask to later get the cells for that winning line
            // Mask in those cells but as if X or O occupied those cells 
            for (row, column) in line {
                let offset = (column + row * 3) * 2;
                line_mask |= 0b11 << offset;
                xo_wonline |= (last_player as u32) << offset;
            }

            // If a line's cells are all the same, a winning line is formed
            // so it's (0 for no wonline OR 1 for a wonline) * (1 as X OR 2 as O) 
            // i.e. (0 OR 1) * (1 OR 2) 
            let masked_line = cells & line_mask;
            let xowin = masked_line == xo_wonline;
            let status = (xowin as u8) * last_player; 

            // Only X or O can win a miniboard which are represented by 1 and 2 respectively
            // with draw being 3
            assert!(status < flag::STATUS_DRAW);   

            // Short-circuit and return early if a winning line is matched
            self.set_status_of(miniboard, status);
            if status > flag::STATUS_CONTESTABLE {
                println!("calc {miniboard} as {}", self.get_status_of(miniboard));
                return true;
            }
        }

        // Reaching here means no winning lines were found
        // So, we check for a draw -- otherwise, it's still contestable
        let move_count = self.get_total_move_count_of(miniboard);
        if move_count == 9 {
            println!("calcd with draw for {miniboard} as {}", self.get_status_of(miniboard));
            self.set_status_of(miniboard, flag::STATUS_DRAW);
            return true;
        }
        
        debug_assert_eq!(self.get_status_of(miniboard), flag::STATUS_CONTESTABLE);

        false
    }

    // TODO: only check miniboards with move_count > 2 
    // Compare against last player's move only for masking 
    /// Determine if there's a winner with `flag::STATUS_X_WIN` or `flag::STATUS_O_WIN`
    /// or neither through `flag::STATUS_DRAW` or `flag::STATUS_CONTESTABLE`
    pub fn calculate_game_status(&mut self) -> u8 {
        // Get miniboard statuses in a winning line of the last_move's miniboard
        let (row, column) = self.last_move;
        let last_player = self.get_cell(row, column);
        println!("\n{last_player}");
        let miniboard = Self::move_miniboard(row, column);
        let status_changed = self.calculate_miniboard_status(miniboard);
        let status = self.get_status_of(miniboard);

        // If the miniboard's status is won by the last move that was just masked_line
        // then check for board winning lines intersecting that miniboard
        // otherwise, exit because the game's state cannot change 
        if !status_changed && status != last_player {
            return flag::STATUS_CONTESTABLE;
        }

        // in a winning line...
        // get winning lines intersecting with the last move's miniboard
        let coords = (miniboard / 3, miniboard % 3);
        let potential_lines = WINNING_LINES
            .iter()
            .filter(|line| line.contains(&coords))
            .collect::<Vec<_>>();
        //println!("{potential_lines:?}");

        for line in potential_lines {
            let mut statusbits = 0u32;
            let mut xoline = 0u32;
            for &(row, column) in line {
                let miniboard = column + row * 3;
                let _ = self.calculate_miniboard_status(miniboard);
                let status = self.get_status_of(miniboard);
                //println!("{status}");
                
                let offset = (column + row * 3) * 2;
                statusbits |= (status as u32) << offset;
                xoline |= (last_player as u32) << offset;
            }
            let won = statusbits == xoline;
            //println!("w/l {won} = {statusbits} - {statusbits:032b} - {xoline}");
            if won {
                return last_player;
            }
        }

        // draw checking -- 1 contestable board suffices as a contestable game technically
        for miniboard in 0..self.main_board.len() as u8 {
            let status = self.get_status_of(miniboard);
            if status == flag::STATUS_CONTESTABLE {
                return flag::STATUS_CONTESTABLE;
            }
        }

        // otherwise, it's drawn.
        flag::STATUS_DRAW
    }

    /// Returns a miniboard's cells in;
    pub fn get_miniboard_cells(&self, miniboard: u8) -> u32  {
        assert!(miniboard < 9);

        let mut cells: u32 = 0;
        let (starting_row, starting_column) = (miniboard / 3 * 3, miniboard % 3 * 3);
        // Options are 0, 3, or 6 for starting rows/columns of a miniboard
        debug_assert!(starting_row % 3 == 0 && starting_row <= 6);
        debug_assert!(starting_column % 3 == 0 && starting_column <= 6);

        for row in 0..3 {
            for column in 0..3 {
                // * 2 to account for cell bit length of 2;
                let mask_offset = (row * 3 + column) * 2;
                let cell = self.get_cell(starting_row + row, starting_column + column);
                let mask = (cell as u32) << mask_offset;
                cells |= mask; 
            }
        }

        cells
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
                    //write!(f, "{:01} ", column + (row * 3) * 3)?;
                    if cell == 1 {
                        //write!(f, "X ")?;
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
                writeln!(f, "[{i}]: cells[{:18b}]-meta[{:14b}]", self.get_row_cells(i as u8), self.main_board[i] >> 18)?
            }
            Ok(())
        }
    }
}
