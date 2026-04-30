mod display;
mod iterator;
pub mod move_tracker;
pub mod rules;

// #[allow(dead_code)]
pub mod flag {
    pub(crate) const CELL_LEN                  : u8 = 2; 
    pub(crate) const CELL_MASK                 : u32 = (1 << CELL_LEN) - 1;

    pub const MINIBOARD_STATUS_POS             : u8 = 18;
    pub(crate) const STATUS_LEN                : u8 = 0b11;
    pub const STATUS_CONTESTABLE               : u8 = 0;
    pub const STATUS_X_WIN                     : u8 = 1;
    pub const STATUS_O_WIN                     : u8 = 2;
    pub const STATUS_DRAW                      : u8 = 3;

    pub(crate) const MINIBOARD_MOVE_COUNT_X    : u8 = 20;
    pub(crate) const MINIBOARD_MOVE_COUNT_O    : u8 = 24;
    pub(crate) const MINIBOARD_MOVE_COUNT_TOTAL: u8 = 28;
    pub(crate) const MOVE_COUNT_BIT_SIZE       : u8 = 0b1111;

    pub const NEW_GAME                         : u8 = 0;

    pub const EMPTY                            : u8 = 0;
    pub const X_PLAYER                         : u8 = 1;
    pub const O_PLAYER                         : u8 = 2;
}
use flag::*;

/// `u32` board row format:
/// most significant bit <- -> least significant bit
/// meta data bits `31–18` -- cells `17–0`
/// `[14 bits meta data — 18 bits cell data]`
///     * `9` cells x `2` bits per cell = `18` bits
///     * meta data `[0000 — 0000 — 0000 — 00]`
///         * `4` bits move_count_total — move_count o `4` bits — move_count x `4` bits — miniboard_status `2` bits
/// xo_miniboard_win_count: `[00 — 000 — 000]` -> `[empty — O win count — X win count]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub main_board: [u32; 9],
    pub last_move: (u8, u8), // The last valid move that was made
    pub xo_miniboard_win_count: u8,
}

impl Board {
    /// Creates a new, empty `Board`.
    ///
    // # Example
    /// ```
    /// use ut3_oxide::board::Board;
    ///
    /// let board = Board::new();
    /// ```
    pub fn new() -> Self {
        Board {
            main_board: [0; 9],
            last_move: (flag::NEW_GAME, flag::NEW_GAME),
            xo_miniboard_win_count: 0,
        }
    }

    /// Resets and zeroes the state of Board as if it was just initialised.
    ///
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// # let mut board = Board::default();
    /// board.do_move(4, 4, 1);
    /// assert_eq!(board.get_cell(4, 4), 1);
    ///
    /// // Reset the board
    /// board.reset();
    /// assert_eq!(board.get_cell(4, 4), 0);
    /// ```
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

        ((self.main_board[row as usize] >> offset) & CELL_MASK) as u8
    }

    #[inline]
    pub fn set_cell(&mut self, row: u8, column: u8, value: u8) {
        assert!(row < 9);
        assert!(column < 9);

        // clear bits
        let offset = column * 2;
        self.main_board[row as usize] &= !(CELL_MASK << offset);

        // set bits
        self.main_board[row as usize] |= (value as u32) << offset;
    }

    /////* Miniboard Meta Data */////

    #[inline(always)]
    fn get_meta_data(&self, miniboard: u8, flag_pos: u8, flag_len: u8) -> u8 {
        assert!(miniboard < 9);

        let mask = (flag_len as u32) << flag_pos;
        ((self.main_board[miniboard as usize] & mask) >> flag_pos) as u8
    }

    #[inline(always)]
    fn set_meta_data(&mut self, miniboard: u8, flag_pos: u8, flag_size: u8, value: u8) {
        // clear the occupying bits
        let miniboard = miniboard as usize;
        let mask = (flag_size as u32) << flag_pos;
        self.main_board[miniboard] &= !mask;

        // set the cleared bits
        let mask = (value as u32) << flag_pos;
        self.main_board[miniboard] |= mask;
    }

    /// Returns the status of a `miniboard`.
    // NOTE: hot function
    #[inline(always)]
    pub fn get_status_of(&self, miniboard: u8) -> u8 {
        self.get_meta_data(miniboard, flag::MINIBOARD_STATUS_POS, flag::STATUS_LEN)
    }

    /// Sets the status of a `miniboard`.
    #[inline(always)]
    pub fn set_status_of(&mut self, miniboard: u8, value: u8) {
        assert!(value <= flag::STATUS_DRAW);
        self.set_meta_data(
            miniboard,
            flag::MINIBOARD_STATUS_POS,
            flag::STATUS_LEN,
            value,
        );
    }

    /// Returns the number of `miniboards` won by a `player`.
    pub fn get_miniboard_win_count_of(&self, player: u8) -> u8 {
        assert!(player == flag::X_PLAYER || player == flag::O_PLAYER);
        let offset = (player - 1) * 3;
        let mask = 0b111 << offset;
        (self.xo_miniboard_win_count & mask) >> offset
    }

    /// Sets the number of `miniboards` won by a `player`.
    fn set_miniboard_win_count_of(&mut self, player: u8, value: u8) {
        assert!(player == flag::X_PLAYER || player == flag::O_PLAYER);
        assert!(value <= 7); // 7 is the max for 0b111, a the logical max in a ut3 game
        let offset = (player - 1) * 3;

        // clear bits
        let mask = 0b111 << offset;
        let mut win_count = self.xo_miniboard_win_count & !mask;

        // set bits
        win_count |= value << offset;
        self.xo_miniboard_win_count = win_count;
    }

    /// Returns the number of `moves` a player has made in a `miniboard`.
    /// # Example
    /// ```ignore
    /// # use ut3_oxide::board::Board;
    /// let mut board = Board::default();
    /// board.do_move(4, 4, 2);
    /// board.do_move(4, 5, 2);
    /// board.do_move(3, 5, 2);
    ///
    /// board.do_move(4, 3, 1);
    ///
    /// // miniboard 4, player 2
    /// assert_eq!(board.get_player_move_count_of(4, 2), 3);
    /// ```
    // NOTE: hot function
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

    /// Sets the number of `moves` a player has made in a `miniboard`.
    //#[inline(always)]
    fn set_player_move_count_of(&mut self, miniboard: u8, value: u8, player: u8) {
        // maps 1 => 22 and 2 => 26
        let player_flag = flag::MINIBOARD_MOVE_COUNT_X + (player - 1) * 4;
        debug_assert!(
            player_flag == flag::MINIBOARD_MOVE_COUNT_X
                || player_flag == flag::MINIBOARD_MOVE_COUNT_O
        );
        self.set_meta_data(miniboard, player_flag, flag::MOVE_COUNT_BIT_SIZE, value);
    }

    /// Returns the total number of `moves` both `players` have made in a `miniboard`.
    //#[inline(always)]
    pub fn get_total_move_count_of(&self, miniboard: u8) -> u8 {
        //let x_count = self.get_meta_data(miniboard, flag::MINIBOARD_MOVE_COUNT_X, flag::MOVE_COUNT_BIT_SIZE);
        //let o_count = self.get_meta_data(miniboard, flag::MINIBOARD_MOVE_COUNT_O, flag::MOVE_COUNT_BIT_SIZE);
        //x_count + o_count
        self.get_meta_data(
            miniboard,
            flag::MINIBOARD_MOVE_COUNT_TOTAL,
            flag::MOVE_COUNT_BIT_SIZE,
        )
    }

    fn set_total_move_count_of(&mut self, miniboard: u8, value: u8) {
        // maps 1 => 22 and 2 => 26
        self.set_meta_data(
            miniboard,
            flag::MINIBOARD_MOVE_COUNT_TOTAL,
            flag::MOVE_COUNT_BIT_SIZE,
            value,
        );
    }

    /////* Moves */////

    /// Applies the move but doesn't check validity with `self.is_valid_move`, or apply
    /// minboard status checks
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// let mut board = Board::default();
    /// board.do_move(4, 4, 2);
    /// assert_eq!(board.get_cell(4, 4), 2);
    /// ```
    pub fn do_move(&mut self, row: u8, column: u8, player: u8) {
        assert!(row < 9);
        assert!(column < 9);
        assert!(player <= 2);
        debug_assert_ne!(player, 0, "'player' was 0! must be 1 or 2");
        let miniboard = Self::move_miniboard(row, column);

        self.set_cell(row, column, player);
        self.last_move = (row, column);

        let move_count = self.get_player_move_count_of(miniboard, player);
        debug_assert!(move_count <= 9);

        self.set_player_move_count_of(miniboard, move_count + 1, player);
        self.set_total_move_count_of(miniboard, self.get_total_move_count_of(miniboard) + 1);
        _ = self.calculate_miniboard_status(miniboard);
    }


    /// Returns the `miniboard` that `move` is in.
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// let miniboard = Board::move_miniboard(7, 6);
    /// assert_eq!(miniboard, 8);
    /// ```
    //#[inline]
    pub fn move_miniboard(row: u8, column: u8) -> u8 {
        debug_assert!(row < 9);
        debug_assert!(column < 9);

        (row / 3) * 3 + column / 3
    }

    //TODO: find better name than "corresponding"

    /// Returns the miniboard number that the next move should be played in,
    /// typically based on the previous move.
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// let miniboard = Board::move_corresponding_miniboard(1, 7);
    /// assert_eq!(miniboard, 4);
    /// ```
    #[inline(always)]
    pub fn move_corresponding_miniboard(row: u8, column: u8) -> u8 {
        //_MOVE_CORRESPONDING_MINIBOARD[((column + row * 9)) as usize]
        column % 3 + (row % 3 * 3)
    }

    /// Returns a miniboard's cells as u32
    /// Offsets a cell by `(row * 3 + column) * 2`
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// # let mut board = Board::default();
    /// board.do_move(0, 0, 1);
    /// board.do_move(0, 1, 2);
    /// board.do_move(0, 2, 1);
    ///
    /// let cells = board.get_miniboard_cells(0);
    /// assert_eq!(cells, 0b000000_000000_011001);
    /// ```
    pub fn get_miniboard_cells(&self, miniboard: u8) -> u32 {
        assert!(miniboard < 9);

        let mut cells: u32 = 0;
        let (starting_row, starting_column) = (miniboard / 3 * 3, miniboard % 3 * 3);
        // Options are 0, 3, or 6 for starting rows/columns of a miniboard
        debug_assert!(starting_row % 3 == 0 && starting_row <= 6);
        debug_assert!(starting_column % 3 == 0 && starting_column <= 6);

        // do row by row, so
        // for miniboard 4 -> [(3,3) (3,4) (3,5)] [(4,3) (4,4) (4,5)] [(5,3) (5,4) (5,5)]
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
    /// starting with the least significant bit for miniboard `0`
    /// <- Minboard 8 --- Miniboard 0 ->
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// # let mut board = Board::default();
    /// board.set_status_of(1, 1);
    /// board.set_status_of(4, 1);
    /// board.set_status_of(8, 2);
    ///
    /// let statuses = board.get_miniboard_statuses();
    /// assert_eq!(statuses, 0b100000_000100_000100);
    /// ```
    pub fn get_miniboard_statuses(&self) -> u32 {
        let mut statuses = 0u32;
        for mb in 0..self.main_board.len() as u8 {
            let status = self.get_status_of(mb);
            let offset = mb * 2;

            statuses |= (status as u32) << offset;
        }

        statuses
    }

    pub fn _get_miniboard_statuses_iter(&self) -> iterator::MiniboardStatusesIterator {
        let bitfield = self.get_miniboard_statuses();

        iterator::MiniboardStatusesIterator::new(bitfield)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            main_board: [0; 9],
            last_move: (flag::NEW_GAME, flag::NEW_GAME),
            xo_miniboard_win_count: 0,
        }
    }
}

impl From<Board> for String {
    fn from(board: Board) -> Self {
        let mut board_str = board.main_board.iter().fold(String::with_capacity(50), |mut acc, row| {
            acc.push_str(format!("{row}:").as_str());
            acc
        });

        board_str.push_str(&format!("{}:", board.last_move.0));
        board_str.push_str(&format!("{}:", board.last_move.1));
        board_str.push_str(&format!("{}", board.xo_miniboard_win_count));

        board_str
    }
}

#[cfg(test)]
mod tests;
