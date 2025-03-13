pub mod rules;
pub mod display;
pub mod iterator;

use iterator::ValidMoveIterator;
// TODO: move miniboard, move_corresponding_miniboard, miniboard_starting_coord lookup tables
// miniboard 1d -> miniboard 2d:     mb // 3, mb % 3
// miniboard -> starting_coord:      mb // 3 * 3, mb % 3 * 3
// move -> miniboard:                row // 3 * 3 + column 
// move -> corresponding mb:        (column % 3) + (row % 3) * 3
// miniboard -> corresponding cells: 
// mb 1, 2 (5) <- row * 3, col
//  (1, 2) (1, 5) (1, 8)
//  (4, 2) (4, 5) (4, 8)
//  (7, 2) (7, 5) (7, 8)
// mb -> (mb // 3) + (row * 3), (mb % 3) + col * 3 

#[allow(dead_code)]
pub mod flag {
    pub const MINIBOARD_STATUS:                          u8 = 20;
    pub (in crate::board) const STATUS_BIT_SIZE:         u8 = 0b11;
    pub const STATUS_CONTESTABLE:                        u8 = 0;
    pub const STATUS_X_WIN:                              u8 = 1;
    pub const STATUS_O_WIN:                              u8 = 2;
    pub const STATUS_DRAW:                               u8 = 3;

    pub (in crate::board) const MINIBOARD_MOVE_COUNT_X:  u8 = 22;
    pub (in crate::board) const MINIBOARD_MOVE_COUNT_O:  u8 = 26;
    pub (in crate::board) const MOVE_COUNT_BIT_SIZE:     u8 = 0b1111;

    pub const NEW_GAME:                                  u8 = u8::MAX;

    pub const EMPTY:                                     u8 = 0;
    pub const X_PLAYER:                                  u8 = 1;
    pub const O_PLAYER:                                  u8 = 2;
}

/// `u32` board row format:
/// most significant bit <- -> least significant bit
/// `[14 bits meta data -- 18 bits cell data]`
/// `9` cells x `2` bits per cell = `18` bits 
/// meta data `4` bits empty -- move_count o `4` bits -- move_count x `4` bits -- miniboard_status `2` bits
/// xo_miniboard_win_count: `[00 -- 000 -- 000]` -> `[empty -- O win count -- X win count]`
#[derive(Debug, Clone)]
pub struct Board {
    pub main_board: [u32; 9],
    pub (crate) last_move: (u8, u8),    // The last valid move that was made 
    pub (crate) xo_miniboard_win_count: u8,
}

impl Board {

    pub fn new() -> Self {
        Board { 
            main_board: [0; 9],
            last_move: (flag::NEW_GAME, flag::NEW_GAME),
            xo_miniboard_win_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.last_move = (flag::NEW_GAME, flag::NEW_GAME);
        self.main_board = [0; 9];
        self.xo_miniboard_win_count = 0;
    }

    /////* Cell Operations */////

    #[inline]
    pub fn get_cell(&self, row: u8, column: u8) -> u8 {
        assert!(row < 9);
        assert!(column < 9);
        let offset = column * 2;
        let mask = 0b11 << offset;

        ((self.main_board[row as usize] & mask) >> offset) as u8
    }

    #[inline]
    pub fn set_cell(&mut self, row: u8, column: u8, value: u8) {
        assert!(row < 9);
        assert!(column < 9);
        // clear bits
        let row = row as usize;
        let offset = column * 2;
        let mask = 0b11 << offset;
        self.main_board[row] &= !(mask as u32); 
        
        // set bits
        let mask = (value as u32) << offset;
        self.main_board[row] |= mask;
    }

    /////* Miniboard Meta Data */////

    #[inline(always)]
    pub fn get_meta_data(&self, miniboard: u8, flag_pos: u8, flag_size: u8) -> u8 {
        assert!(miniboard < 9);

        let mask = (flag_size as u32) << flag_pos;
        ((self.main_board[miniboard as usize] & mask) >> flag_pos) as u8
    }

    #[inline(always)]
    pub fn set_meta_data(&mut self, miniboard: u8, flag_pos: u8, flag_size: u8, value: u8) {
        // clear the occupying bits
        let miniboard = miniboard as usize;
        let mask = (flag_size as u32) << flag_pos;
        self.main_board[miniboard] &= !mask;

        // set the cleared bits
        let mask = (value as u32) << flag_pos;
        self.main_board[miniboard] |= mask;
    }

    // NOTE: hot function
    #[inline(always)]
    pub fn get_status_of(&self, miniboard: u8) -> u8 {
        self.get_meta_data(miniboard, flag::MINIBOARD_STATUS, flag::STATUS_BIT_SIZE) 
    }

    #[inline(always)]
    pub fn set_status_of(&mut self, miniboard: u8, value: u8) {
        assert!(value <= flag::STATUS_DRAW);
        self.set_meta_data(miniboard, flag::MINIBOARD_STATUS, flag::STATUS_BIT_SIZE, value);
    }
    
    pub fn get_miniboard_win_count_of(&self, player: u8) -> u8 {
        assert!(player == flag::X_PLAYER || player == flag::O_PLAYER);
        let offset = (player - 1) * 3;
        let mask = 0b111 << offset; 
        (self.xo_miniboard_win_count & mask) >> offset
    }

    pub fn set_miniboard_win_count_of(&mut self, player: u8, value: u8) {
        assert!(player == flag::X_PLAYER || player == flag::O_PLAYER);
        assert!(value <= 7);    // 7 is the max for 0b111
        let offset = (player - 1) * 3;

        // clear bits
        let mask = 0b111 << offset; 
        let mut win_count = self.xo_miniboard_win_count & !mask;

        // set bits
        win_count |= value << offset;
        self.xo_miniboard_win_count = win_count;
    }

    //NOTE: hot function
    //#[inline(always)]
    pub fn get_player_move_count_of(&self, miniboard: u8, player: u8) -> u8 {
        // maps 1 => 22 and 2 => 26
        let player_flag = flag::MINIBOARD_MOVE_COUNT_X + (player - 1) * 4;
        debug_assert!(
            player_flag == flag::MINIBOARD_MOVE_COUNT_X
            || player_flag == flag::MINIBOARD_MOVE_COUNT_O
        );
        self.get_meta_data(miniboard, player_flag, flag::MOVE_COUNT_BIT_SIZE)
    }

    //#[inline(always)]
    pub fn set_player_move_count_of(&mut self, miniboard: u8, value: u8, player: u8) {
        // maps 1 => 22 and 2 => 26
        let player_flag = flag::MINIBOARD_MOVE_COUNT_X + (player - 1) * 4;
        debug_assert!(
            player_flag == flag::MINIBOARD_MOVE_COUNT_X
            || player_flag == flag::MINIBOARD_MOVE_COUNT_O
        );
        self.set_meta_data(miniboard, player_flag, flag::MOVE_COUNT_BIT_SIZE, value); 
    }

    //#[inline(always)]
    pub fn get_total_move_count_of(&self, miniboard: u8) -> u8 {
        let x_count = self.get_meta_data(miniboard, flag::MINIBOARD_MOVE_COUNT_X, flag::MOVE_COUNT_BIT_SIZE);
        let o_count = self.get_meta_data(miniboard, flag::MINIBOARD_MOVE_COUNT_O, flag::MOVE_COUNT_BIT_SIZE);
        x_count + o_count
    }
    
    /////* Moves */////

    /// Applies the move but doesn't check validity with `self.is_valid_move`, or apply 
    /// minboard status checks
    pub fn do_move(&mut self, row: u8, column: u8, player: u8) {
        assert!(row < 9);
        assert!(column < 9);
        assert!(player <= 2);
        debug_assert_ne!(player, 0, "'player' was 0! must be 1 or 2");

        self.set_cell(row, column, player);
        self.last_move = (row, column);

        let miniboard = Self::move_miniboard(row, column);
        let move_count = self.get_player_move_count_of(miniboard, player);
        assert!(move_count <= 9);

        self.set_player_move_count_of(miniboard, move_count + 1, player);
        self.calculate_miniboard_status(miniboard);

    }

    /// Returns the miniboard that move is in
    //#[inline]
    pub fn move_miniboard(row: u8, column: u8) -> u8 {
        debug_assert!(row < 9);
        debug_assert!(column < 9);

        column / 3 + (row / 3) * 3  
    }

    //TODO: find better name than "corresponding"
    
    /// Returns the miniboard number that the next move should be played in
    //#[inline]
    pub fn move_corresponding_miniboard(row: u8, column: u8) -> u8 {
        (column % 3) + (row % 3) * 3
    }

    // TODO: miniboard cells iterator?
    /// Offsets a cell by `(row * 3 + column) * 2`
    /// Returns a miniboard's cells as u32
    pub fn get_miniboard_cells(&self, miniboard: u8) -> u32  {
        assert!(miniboard < 9);

        let mut cells: u32 = 0;
        let (starting_row, starting_column) = (miniboard / 3 * 3, miniboard % 3 * 3);
        // Options are 0, 3, or 6 for starting rows/columns of a miniboard
        debug_assert!(starting_row % 3 == 0 && starting_row <= 6);
        debug_assert!(starting_column % 3 == 0 && starting_column <= 6);

        // do row by row, so 
        // for miniboard 4 -> [3,3 3,4 3,5] [4,3 4,4 4,5] [5,3 5,4 5,5] 
        // not cell by cell
        let col_offset = starting_column * 2;
        let columns_mask = 0b11_11_11 << col_offset;
        for row in 0..3 {
            // * 2 to account for cell bit length of 2;
            let row_cells =
                (self.main_board[(starting_row + row) as usize] & columns_mask) >> col_offset;
            let cell_mask_offset = (row * 3) * 2;
            cells |= row_cells << cell_mask_offset;
        }

        cells
    }

    /// Returns the miniboard statuses and packs them into a `u32`
    pub fn get_miniboard_statuses(&self) -> u32 {
        let mut statuses = 0u32;
        for mb in 0..self.main_board.len() as u8 {
            let status = self.get_status_of(mb);
            let offset = mb * 2;

            statuses |= (status as u32) << offset;
        }

        statuses
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}


// TODO: move all tests to right place
#[test]
fn statusesmb() {
    let mut board = Board::new();
    board.set_status_of(0, 2);
    board.set_status_of(1, 1);
    board.set_status_of(2, 2);
    board.set_status_of(8, 1);

    let statuses = board.get_miniboard_statuses();
    assert_eq!(statuses, 0b10000_000000_100110);
}

#[test]
fn get_miniboard_cells_v2() {
    let mut board = Board::new();
    board.set_cell(3, 3, 1);
    board.set_cell(3, 4, 2);
    board.set_cell(3, 5, 2);

    board.set_cell(4, 4, 2);

    board.set_cell(5, 5, 1);

    let cells = board.get_miniboard_cells(4);
    assert_eq!(cells, 0b010000_001000_101001);
}
