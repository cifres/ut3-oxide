use super::{Board, bitflag, iterator::ValidMoveIterator};

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

    const DIAGONAL_LINE: [(u8, u8); 6] = [(0, 0), (1, 1), (2, 2), (0, 2), (1, 1), (2, 0)];

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

pub const WINNING_LINES: [[(u8, u8); 3]; 8] = build_winning_lines();

impl Board {
    /// Returns if a `move` is `valid` or `invalid`
    /// validate moves by ensuring that invalidity if:
    /// * cell is occupied
    /// * miniboard is 'uncontestable' i.e. won by X or O, or drawn
    /// * miniboard coords don't correspond to last move
    /// * exception: corresponding board is uncontestable -- then we play anywhere else where a cells is
    ///   empty, and its board is contestable
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// let board = Board::default();
    /// assert!(board.is_valid_move(4, 4));
    /// ```
    pub fn is_valid_move(&self, row: u8, column: u8) -> bool {
        assert!(row < 9);
        assert!(column < 9);

        if self.is_first_move() {
            return true;
        }

        let (last_row, last_column) = self.last_move;
        let miniboard = Self::move_miniboard(row, column);
        let corresponding_miniboard = Self::move_corresponding_miniboard(last_row, last_column);

        let cell_is_empty = self.get_cell(row, column) == bitflag::EMPTY;

        if self.get_status_of(miniboard) != bitflag::STATUS_CONTESTABLE {
            return false;
        }

        // if the miniboard doesn't correspond with the last move...
        if miniboard != corresponding_miniboard {
            // and the corresponding miniboard is uncontestable, whereas the selected miniboard is
            // and the cell is empty, allow the exception
            if self.get_status_of(corresponding_miniboard) != bitflag::STATUS_CONTESTABLE
                && cell_is_empty
            {
                return true;
            }

            return false;
        }

        if !cell_is_empty {
            return false;
        }

        //println!("valid {:?}", (row, column));
        true
    }

    /// Generates a bitfield from the valid moves on the miniboards
    /// `1` == valid and `0` == invalid
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// let mut board = Board::default();
    /// assert_eq!(board.valid_moves_bitfield(), (1 << 81) - 1);    // every move is valid
    /// board.do_move(3, 3, 2);
    /// assert_eq!(board.valid_moves_bitfield(), 0b111000000111000000111);
    /// ```
    // TODO: order moves by importance?
    pub fn valid_moves_bitfield(&self) -> u128 {
        let mut moves_validity_bitfield = 0u128;

        // if corresponding is uncontestable, we fully validate the whole board
        // otherwise, we check for valid moves only in the corresponding miniboard
        let (last_row, last_column) = self.last_move;
        let corresponding_mb = Self::move_corresponding_miniboard(last_row, last_column);

        if !self.is_first_move() && self.get_status_of(corresponding_mb) == bitflag::STATUS_CONTESTABLE
        {
            let (starting_row, starting_column) =
                (corresponding_mb / 3 * 3, corresponding_mb % 3 * 3);
            for row in starting_row..starting_row + 3 {
                for column in starting_column..starting_column + 3 {
                    //let valid = self.is_valid_move(row, column);
                    let valid = self.get_cell(row, column) == bitflag::EMPTY;
                    let offset = column + row * 9;
                    moves_validity_bitfield |= (valid as u128) << offset;
                }
            }

            moves_validity_bitfield
        } else {
            for row in 0..9 {
                for column in 0..9 {
                    // We know the corresponding_mb is uncontestable/invalid
                    // so, only check for the other two rules; contestable miniboard and empty cell
                    // let valid = self.is_valid_move(row, column);
                    let valid = self.get_status_of(Self::move_miniboard(row, column))
                        == bitflag::STATUS_CONTESTABLE
                        && self.get_cell(row, column) == bitflag::EMPTY;

                    let offset = column + row * 9;
                    moves_validity_bitfield |= (valid as u128) << offset;
                }
            }

            moves_validity_bitfield
        }
    }

    /// Returns if the first move has yet to be made
    /// Simply checks if the latest move made was `empty`, i.e. not `X` or `O`
    pub(crate) fn is_first_move(&self) -> bool {
        let (last_row, last_column) = self.last_move;
        let last_cell = self.get_cell(last_row, last_column);

        last_cell == bitflag::EMPTY
    }

    /// Returns an iterator yielding valid `moves` as `(row, column)`
    pub fn valid_moves(&self) -> ValidMoveIterator {
        let bitfield = self.valid_moves_bitfield();

        ValidMoveIterator::new(bitfield)
    }

    /// Returns a `bool` with `true` indicating the miniboard's status isn't `flag::STATUS_CONTESTABLE`
    /// Checks and calculates the miniboard's status based on any winning lines
    /// Short-circuits and stops as soon as any winning line is found
    /// Use with `get_status_of(miniboard: u8)`
    /// i.e, `flag::STATUS_X_WIN`, `flag::STATUS_O_WIN`, or `flag::STATUS_DRAW`
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// let mut board = Board::default();
    /// board.set_status_of(4, 2);
    /// assert!(board.calculate_miniboard_status(4));
    /// ```
    pub fn calculate_miniboard_status(&mut self, miniboard: u8) -> bool {
        assert!(miniboard < 9);

        // Don't recheck/recalculate winning lines for an uncontestable (won/lost/drawn) miniboard
        if self.get_status_of(miniboard) != bitflag::STATUS_CONTESTABLE {
            //println!("nocalc cached {miniboard} as state {}", self.get_status_of(miniboard));
            return true;
        }

        let last_player = self.get_cell(self.last_move.0, self.last_move.1);
        debug_assert_ne!(last_player, bitflag::EMPTY);

        // early exit miniboards that cannot be won/lost/drawn yet.
        // equivalent to get_total_move_count_of < 3 but more accurate because it's impossible to
        // not win a miniboard if player move count > 6
        let player_move_count = self.get_player_move_count_of(miniboard, last_player);
        if player_move_count < 3 {
            //println!("nocalc cached {miniboard} as state {} -- movecount < 3 for {last_player}", self.get_status_of(miniboard));
            debug_assert_eq!(self.get_status_of(miniboard), bitflag::STATUS_CONTESTABLE);
            return false;
        }

        // cheap check because logically you auto-win a miniboard if you win 7 moves in it
        // 7 Xs for example guarentee that X has won that miniboard
        if player_move_count > 6 {
            self.set_status_of(miniboard, last_player);
            return true;
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
                let offset = (row * 3 + column) * 2;
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
            debug_assert!(status < bitflag::STATUS_DRAW);

            // Short-circuit and return early if a winning line is matched
            if status > bitflag::STATUS_CONTESTABLE {
                self.set_status_of(miniboard, status);
                //println!("calc {miniboard} as {}", self.get_status_of(miniboard));
                self.set_miniboard_win_count_of(
                    last_player,
                    self.get_miniboard_win_count_of(last_player) + 1,
                );

                return true;
            }
        }

        // Reaching here means no winning lines were found
        // So, we check for a draw -- otherwise, it's still contestable
        let move_count = self.get_total_move_count_of(miniboard);
        if move_count == 9 {
            //println!("calcd with draw for {miniboard} as {}", self.get_status_of(miniboard));
            self.set_status_of(miniboard, bitflag::STATUS_DRAW);
            return true;
        }

        debug_assert_eq!(self.get_status_of(miniboard), bitflag::STATUS_CONTESTABLE);

        false
    }

    // TODO: only check miniboards with move_count > 2

    // Compare against last player's move only for masking
    /// Determine if there's a winner with `flag::STATUS_X_WIN` or `flag::STATUS_O_WIN`
    /// or neither through `flag::STATUS_DRAW` or `flag::STATUS_CONTESTABLE`
    /// # Example
    /// ```
    /// # use ut3_oxide::board::Board;
    /// let mut board = Board::default();
    /// // Win miniboard 4
    /// board.do_move(4, 3, 1);
    /// board.do_move(4, 4, 1);
    /// board.do_move(4, 5, 1);
    /// // Emulate winnning two more in a line
    /// board.set_status_of(3, 1);
    /// board.set_status_of(5, 1);
    /// assert_eq!(board.calculate_game_status(), 1);
    /// ```
    pub fn calculate_game_status(&self) -> u8 {
        let (row, column) = self.last_move;
        let last_player = self.get_cell(row, column);
        //println!("\n{last_player}");
        let last_miniboard = Self::move_miniboard(row, column);
        let mb_status = self.get_status_of(last_miniboard);

        // If the miniboard's status is won by the last move that was just made
        // then check for board winning lines intersecting that miniboard
        // otherwise, exit because the game's state cannot change

        // maybe todo?: if we just won/drew one and we've won less than 3, and there are other
        // contestable miniboards, then it's still contestable
        if mb_status == bitflag::STATUS_CONTESTABLE {
            return bitflag::STATUS_CONTESTABLE;
        }

        // cheap check because it's logically impossible to not win if you win 7 miniboards
        if self.get_miniboard_win_count_of(last_player) > 6 {
            return last_player;
        }
        // get winning lines intersecting with the last move's miniboard
        let last_mb_coords = (last_miniboard / 3, last_miniboard % 3);
        let potential_lines = WINNING_LINES
            .iter()
            .filter(|line| line.contains(&last_mb_coords));

        for line in potential_lines {
            let mut statusbits = 0u32;
            let mut xoline = 0u32;
            for &(row, column) in line {
                let miniboard = column + row * 3;
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
        // TODO: put miniboard statuses into bitfield and call .trailing_zeros for efficiency
        // also useful for ai
        for miniboard in 0..9 {
            let status = self.get_status_of(miniboard);
            if status == bitflag::STATUS_CONTESTABLE {
                return bitflag::STATUS_CONTESTABLE;
            }
        }

        // otherwise, it's drawn.
        bitflag::STATUS_DRAW
    }
}
