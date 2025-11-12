use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::board::{
    Board,
    flag::{self, STATUS_CONTESTABLE},
    rules::WINNING_LINES,
};

// TODO: adjust weights

// basic multiplier and weight adjustment for what is valued
const SCORE_UNIT:                   i16 = 10;
const MINIBOARD_WIN_COUNT:          i16 = 24; // increase?
const CENTRE_MB_CONTROL:            i16 = 25;
const CENTRE_CELL_CONTROL:          i16 =  1;
const FREE_MOVE_CELL_SAME_MB:       i16 =  3;
const UNCONTESTABLE_MB_POINTED_AT:  i16 =  3;
const CONTINUOUS_MB_LINES:          i16 =  3;
const BROKEN_MB_LINES:              i16 =  2;
const CONTINUOUS_CELL_LINES:        i16 =  2;
const BROKEN_CELL_LINES:            i16 =  1;
// const N: u16 = 7 * 24 + 25 + 9 + 9 * 3 - (9 * 3) + (3 * 7) + (3 * 4) + (2 * 7) + (2 * 4);

// NOTE: continuous lines count in both directions despite the overlap
// whereas broken ones count once only.
// TODO: rename continuous and broken lines?

const DEPTH: u8 = 5;

// Winning lines composed into continous and broken line patterns
// Format is [continuous a continuous b, broken] where A and B intersect the e.g.
// [(0, 0), (0, 1), (0, 2)] -> [(0, 0), (0, 1), (0, 1), (0, 2), ... // <-- continuous A and B
// ... (1, 0), (0, 2)] <-- broken line pattern
const LINE_PATTERNS: [[(u8, u8); 6]; 8] = const {
    let mut i = 0;
    // continuous is a 3-in-a-row split up
    // so miniboards [0, 1, 2] become [0, 1] [1, 2]
    let mut combined = [[(0u8, 0u8); 6]; 8];

    while i < WINNING_LINES.len() {
        let wline = WINNING_LINES[i];

        combined[i] = [wline[0], wline[1], wline[1], wline[2], wline[0], wline[2]];

        i += 1;
    }

    combined
};

#[derive(Debug, Clone, Copy)]
pub struct AI {
    ai_shape: u8,
    opponent_shape: u8,
}

impl Default for AI {
    fn default() -> Self {
        Self::new(flag::O_PLAYER)
    }
}

impl AI {
    /// Creates a new [`AI`] with either `X` or `O` shape
    /// representing `1` or `2`.
    /// # Example
    /// ```
    /// use ut3_oxide::{board::flag, ai::AI};
    /// let ai_x = AI::new(flag::X_PLAYER);
    /// let ai_o = AI::new(flag::O_PLAYER);
    /// ```
    pub fn new(ai_shape: u8) -> Self {
        //println!("Ai created as {ai_shape}");
        let opponent_shape = if ai_shape == flag::O_PLAYER {
            flag::X_PLAYER
        } else {
            flag::O_PLAYER
        };
        Self {
            ai_shape,
            opponent_shape,
        }
    }

    /// Generates a move in parallel at `depth` given the `Board` state.
    /// # Example
    /// ```
    /// # use ut3_oxide::{board::Board, ai::AI};
    /// let mut board = Board::default();
    /// let aio = AI::default();
    /// let (eval, ((row, column))) = aio.calculate_move_par(&board, 5);
    /// board.do_move(row, column, 2);  // 2 represents O
    /// ```
    pub fn calculate_move_par(&self, board: &Board, depth: u8) -> (i16, (u8, u8)) {
        let depth = if depth > 0 { depth } else { DEPTH };

        let bstmv = board.valid_moves()
            .par_bridge()
            .map(|(row, column)| {
                let mut board = board.clone();
                board.do_move(row, column, self.ai_shape);
                let score = self.alphabeta_mm(&mut board, depth - 1, i16::MIN, i16::MAX, false);
                (score, (row, column))
            })
            .max_by_key(|(score, (_, _))| *score);

        //println!("{bstmv:?}");
        //println!("best {best_move:?} score {best_score} at depth {DEPTH}");
        bstmv.expect("a move should've been selected")
    }

    /// Returns the evaluation of the current state after simulating moves till `depth`
    /// through `α-β pruning` variation of `minimax`.
    /// # Example
    /// ```
    /// # use ut3_oxide::{board::Board, ai::AI};
    /// # let depth = 1;
    /// // initial call example
    /// let mut board = Board::default();
    /// let aio = AI::default();
    /// // for move in moves...
    /// // let mut board = board.clone();
    /// // board.do_move(row, column, self.ai_shape);
    /// // let score = self.alphabeta_mm(&mut board, depth - 1, i16::MIN, i16::MAX, false);
    /// ```
    fn alphabeta_mm(
        &self,
        board: &mut Board,
        depth: u8,
        mut alpha: i16,
        mut beta: i16,
        is_max: bool,
    ) -> i16 {
        if depth == 0 || board.calculate_game_status() != flag::STATUS_CONTESTABLE {
            return self.evaluate(board);
        }

        if is_max {
            let mut score = i16::MIN;
            for (row, column) in board.valid_moves() {
                let mut board = board.clone();      //NOTE: hot
                board.do_move(row, column, self.ai_shape);
                score = score.max(self.alphabeta_mm(&mut board, depth - 1, alpha, beta, false));
                //score = score.max(self.alphabeta_mm(board, depth - 1, alpha, beta, false));
                //board.undo_move();
                alpha = alpha.max(score);
                if score >= beta {
                    break;
                }
            }

            score
        } else {
            let mut score = i16::MAX;
            for (row, column) in board.valid_moves() {
                let mut board = board.clone();
                board.do_move(row, column, self.opponent_shape);
                score = score.min(self.alphabeta_mm(&mut board, depth - 1, alpha, beta, true));
                //score = score.min(self.alphabeta_mm(board, depth - 1, alpha, beta, true));
                //board.undo_move();
                beta = beta.min(score);
                if score <= alpha {
                    break;
                }
            }

            score
        }
    }

    // Winning = `i16::MAX`, Drawing = 0, Losing = `i16::MIN`
    // ± score for:
    // * won/lost miniboards
    // * centre-control: ai_shape in centre of miniboard and [near-]winning centre miniboard (#4)
    // * pointing to [near]-won miniboards?
    // * near-won/lost miniboards/full board
    //     * continuous relative +/- 10
    //     * unconnected relative +/- 5
    //     * interrupted/broken relative +/- 7
    // * sending to miniboard
    /// Returns the evaluation of the board state. Used internally to inform good/bad moves
    /// # Example
    /// ```
    /// # use ut3_oxide::{board::Board, ai::AI};
    /// let mut board = Board::default();
    /// let aio = AI::default();
    /// let eval = aio.evaluate(&board);
    /// ```
    //#[unsafe(no_mangle)]
    pub fn evaluate(&self, board: &Board) -> i16 {

        let game_status = board.calculate_game_status();
        if game_status == self.opponent_shape {
            return i16::MIN;
        } else if game_status == self.ai_shape {
            return i16::MAX;
        }

        let mut score = 0;

        /////* Miniboards won */////

        score += ((board.get_miniboard_win_count_of(self.ai_shape) as i16)
            - (board.get_miniboard_win_count_of(self.opponent_shape)) as i16)
            * SCORE_UNIT
            * MINIBOARD_WIN_COUNT;

        // println!("@ mbs won {score}");

        /////* centre-control: board-wide and in individual miniboards */////

        let centre_status = board.get_status_of(4);
        score += (((centre_status == self.ai_shape) as i16)
            - ((centre_status == self.opponent_shape) as i16))
            * SCORE_UNIT
            * CENTRE_MB_CONTROL;

        // println!("@ centre mb {score}");

        // Produces repeating 0b001100 across the 18-bit range 
        const REP_UNIT: u32 = ((1 << 18) - 1) / 0b111111;
        let ai_centre_cell_mask = REP_UNIT * ((self.ai_shape as u32) << 2);
        let opp_centre_cell_mask = REP_UNIT * ((self.opponent_shape as u32) << 2);
        // row-by-row get the centre cell of each miniboard
        // then check if ai or opp
        for row in (1..board.main_board.len()).step_by(3) {
            let row = board.main_board[row];
            let aicnt = (row & ai_centre_cell_mask).count_ones() as i16;
            let oppcnt = (row & opp_centre_cell_mask).count_ones() as i16;
            score += (aicnt - oppcnt) * SCORE_UNIT * CENTRE_CELL_CONTROL;
        }

        // println!("@ centre cell {score}");

        /////* Pointing to uncontestable miniboards */////

        // Reverse it: get uncontestable MBs then directly check would-be corresponding cells
        // miniboard 4 (1, 1) -> (1, 1), (1, 4), (1, 7)
        let cells_pointing_to_uncontestable_mbs = (0..9).filter_map(|mb| {
            if board.get_status_of(mb) == STATUS_CONTESTABLE {
                return None;
            }

            let mut cells: Vec<u8> = Vec::with_capacity(9);
            //let mut cells_int = 0u32;
            let (mb_row, mb_col) = ((mb / 3), (mb % 3));
            // row 1 col 1 | 4 | 7 ...
            for row in 0..3 {
                for col in 0..3 {
                    let cell_state = board.get_cell(mb_row + row * 3, mb_col + col * 3);
                    cells.push(cell_state);
                    //let offset = ((col) + (row * 3)) * 2;
                    //cells_int |= (cell_state as u32) << offset;
                }
            }

            Some(cells)
            //Some(cells_int)
        });

        for uncontestable_mb in cells_pointing_to_uncontestable_mbs {
            for pointing_cell in uncontestable_mb {
                let ai_pointing = pointing_cell == self.ai_shape;
                let opp_pointing = pointing_cell == self.opponent_shape;

                score += (opp_pointing as i16 - ai_pointing as i16)
                    * SCORE_UNIT
                    * UNCONTESTABLE_MB_POINTED_AT;
            }
        }
        // println!("@ uncontestable correspondence {score}");

        //for pointing_cell in cells_pointing_to_uncontestable_mbs {
        //    for i in 0..9 {
        //    // extract cells one by one
        //    let offset = i * 2;
        //    let cell = ((pointing_cell & (0b11 << offset)) >> offset) as u8;
        //    let ai_pointing = cell == self.ai_shape;
        //    let opp_pointing = cell == self.opponent_shape;
        //
        //    score += (opp_pointing as i16 - ai_pointing as i16)
        //        * SCORE_UNIT
        //        * UNCONTESTABLE_MB_POINTING;
        //    }
        //}

        /////* Near-won cell miniboard patterns */////

        // for every active miniboard, get its cells and check for near won patterns
        // NOTE: hot: this loop
        let active_miniboard_cells = (0..9).filter_map(|miniboard| {
            if board.get_player_move_count_of(miniboard, self.ai_shape) >= 2
                || board.get_player_move_count_of(miniboard, self.opponent_shape) >= 2
            {
                Some(board.get_miniboard_cells(miniboard))
            } else {
                None
            }
        });

        for cells in active_miniboard_cells {
            // continuous_line/broken line patterns within miniboards
            for line in LINE_PATTERNS {
                let mut pattern = 0u16;
                let mut pattern_ai = 0u16;
                let mut pattern_opp = 0u16;
                // TODO: extract into resuable function which compares a u16 of 4 bit groups * 3
                // patterns to ai/opponent patterns
                for (i, (row, column)) in line.iter().enumerate() {
                    let offset = (row * 3 + column) * 2;
                    let cell = ((cells >> offset) & 0b11) as u8;

                    let normalised_offset = i * 2;
                    pattern |= (cell as u16) << normalised_offset;
                    pattern_ai |= (self.ai_shape as u16) << normalised_offset;
                    pattern_opp |= (self.opponent_shape as u16) << normalised_offset;
                }

                let ai_continous = ((pattern_ai & 0b1111 == pattern & 0b1111) as i16)
                    + ((pattern_ai & (0b1111 << 4) == pattern & (0b1111 << 4)) as i16);

                let opp_continous = ((pattern_opp & 0b1111 == pattern & 0b1111) as i16)
                    + ((pattern_opp & (0b1111 << 4) == pattern & (0b1111 << 4)) as i16);

                let ai_broken =
                    ((pattern_ai & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;
                let opp_broken =
                    ((pattern_opp & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;

                // NOTE: broken line overlaps with continuous_line which further boosts
                // continuous_line falsely, while a good boost.
                score += ((ai_continous - opp_continous) * SCORE_UNIT * CONTINUOUS_CELL_LINES)
                    + ((ai_broken - opp_broken) * SCORE_UNIT * BROKEN_CELL_LINES);
                // println!("{score} ai {ai_continous} {ai_unconnected} opp {opp_continous} {opp_unconnected}");
                //score += (ai_unconnected - opp_unconnected) * SCORE_UNIT * BROKEN_MB_LINES;
                //score += ((ai_continous + ai_unconnected) - (opp_continous + opp_unconnected))
                //    * SCORE_UNIT
                //    * NEAR_WON_CELL_LINES;
            }
        }

        // println!("@ continuous/unconnected cell lines {score}");

        /////* Near-won miniboard patterns */////

        // continuous pattern strong for 2:  normal winning line: 10 10 10 -> 10 10 [00] OR [00] 10 10
        // continuous pattern weak for 2:  normal winning line: 10 10 10 -> 10 10 [01] OR [01] 10 10
        // unconnected pattern strong for 2:  normal winning line: 10 10 10 -> 10 [00] 10
        // unconnected pattern weak for 2:  normal winning line: 10 10 10 -> 10 [01] 10
        // e.g. winning line [(0, 0), (1, 0), (2, 0)] -> [(0, 0), (1, 0), (1, 0), (2, 0)]
        // 2 wins at this offeset: 1010_1000 & 0b1111 << 4 -> 1010_0000 == 1010_0000 <- 1010_1010 & 0b1111 << 4
        // no one wins at this offset: 1010_0100 & 0b1111 -> 0000_0100 != 0000_1010 <- 1010_1010 & 0b1111

        let mbss = board.get_miniboard_statuses();
        for line in LINE_PATTERNS {
            let mut pattern = 0u16;
            let mut pattern_ai = 0u16;
            let mut pattern_opp = 0u16;
            for (i, (row, column)) in line.iter().enumerate() {
                let miniboard = column + row * 3;
                //let status = (mbss & (0b11 << (miniboard * 2))) >> (miniboard * 2);
                let status = (mbss >> (miniboard * 2)) & 0b11;

                let normalised_offset = i * 2;
                pattern |= (status as u16) << normalised_offset;
                pattern_ai |= (self.ai_shape as u16) << normalised_offset;
                pattern_opp |= (self.opponent_shape as u16) << normalised_offset;
            }

            let ai_continous = ((pattern_ai & 0b1111 == pattern & 0b1111) as i16)
                + ((pattern_ai & (0b1111 << 4) == pattern & (0b1111 << 4)) as i16);

            let opp_continous = ((pattern_opp & 0b1111 == pattern & 0b1111) as i16)
                + ((pattern_opp & (0b1111 << 4) == pattern & (0b1111 << 4)) as i16);

            let ai_broken = ((pattern_ai & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;
            let opp_broken =
                ((pattern_opp & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;

            score += ((ai_continous - opp_continous) * SCORE_UNIT * CONTINUOUS_MB_LINES)
                + ((ai_broken - opp_broken) * SCORE_UNIT * BROKEN_MB_LINES);
            // println!("{score} mb {line:?} ai {ai_continous} {ai_unconnected} opp {opp_continous} {opp_unconnected}");
            //score += ((ai_continous + ai_unconnected) - (opp_continous + opp_unconnected))
            //    * SCORE_UNIT
            //    * NEAR_WON_MB_LINES;
        }
        // println!("@ continuous/unconnected mb lines {score}");

        /////* Sending to free boards */////

        // free board = board with a cell that redirects to its board
        // e.g. 0, 0 -> 0, 4, 4 -> mb 4, 8, 8 -> 8
        // TODO: explore why .into_iter is up to 15% slower on some platforms
        for row in board.main_board.iter().step_by(4) {
            let first = (row & 0b11) as u8;
            let middle = ((row >> 8) & 0b11) as u8;
            let end = ((row >> 16) & 0b11) as u8;
            // simplified from (row & (0b11 << dx)) >> dx, where dx == offset

            let ai_used_free = (((first == self.ai_shape) as u8)
                + ((middle == self.ai_shape) as u8)
                + ((end == self.ai_shape) as u8)) as i16;

            let opp_used_free = (((first == self.opponent_shape) as u8)
                + ((middle == self.opponent_shape) as u8)
                + ((end == self.opponent_shape) as u8)) as i16;

            score += (ai_used_free - opp_used_free) * SCORE_UNIT * FREE_MOVE_CELL_SAME_MB;
        }
        // println!("@ free mb {score}");

        // println!("final {score}");

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub(crate) fn ai_evaluate() {
        let ai = AI::new(flag::O_PLAYER);
        let mut board = Board::default();
        board.do_move(3, 0, 2);

        let score = ai.evaluate(&board);
        assert_eq!(score, 0);

        // winning the centre cell and miniboard
        board.do_move(3, 3, 2);
        board.do_move(4, 4, 2);
        board.do_move(5, 5, 2);
        let _ = board.calculate_game_status();
        let score = ai.evaluate(&board);
        // TODO: check score chain leading up to cells continuous/unconnected check
        assert_eq!(
            score,
            SCORE_UNIT * MINIBOARD_WIN_COUNT
                + SCORE_UNIT * CENTRE_MB_CONTROL
                + SCORE_UNIT * CENTRE_CELL_CONTROL
                + SCORE_UNIT * CONTINUOUS_CELL_LINES * 2
                + SCORE_UNIT * BROKEN_CELL_LINES
                + SCORE_UNIT * FREE_MOVE_CELL_SAME_MB
                - SCORE_UNIT * UNCONTESTABLE_MB_POINTED_AT
        );
        board.reset();

        // losing the centre cell and miniboard
        board.do_move(3, 3, 1);
        board.do_move(4, 4, 1);
        board.do_move(5, 5, 1);
        let _ = board.calculate_game_status();
        let score = ai.evaluate(&board);
        assert_eq!(
            score,
            -SCORE_UNIT * CENTRE_MB_CONTROL
                - SCORE_UNIT * CENTRE_CELL_CONTROL
                - SCORE_UNIT * MINIBOARD_WIN_COUNT
                - SCORE_UNIT * CONTINUOUS_CELL_LINES * 2
                - SCORE_UNIT * BROKEN_CELL_LINES
                - SCORE_UNIT * FREE_MOVE_CELL_SAME_MB
                + SCORE_UNIT * UNCONTESTABLE_MB_POINTED_AT
        );
        board.reset();

        // winning/losing miniboards
        board.do_move(2, 6, 2);
        board.do_move(2, 7, 2);
        board.do_move(2, 8, 2);

        board.do_move(3, 6, 2);
        board.do_move(3, 7, 2);
        board.do_move(3, 8, 2);

        board.do_move(3, 0, 1);
        board.do_move(3, 1, 1);
        board.do_move(3, 2, 1);

        let score = ai.evaluate(&board);
        assert_eq!(score,
            SCORE_UNIT * MINIBOARD_WIN_COUNT // 2 - 1
                + SCORE_UNIT * CONTINUOUS_MB_LINES
                + SCORE_UNIT * CONTINUOUS_CELL_LINES * (4 - 2)
                + SCORE_UNIT * BROKEN_CELL_LINES,   // 2 - 1
        );

        board.reset();

        // near won/lost continous/unconnected lines
        println!("continuous_line test");
        board.do_move(3, 1, 2);
        board.set_status_of(5, 2);
        board.set_status_of(8, 2);

        board.set_status_of(0, 2);
        board.set_status_of(3, 2);

        board.display_mb_statuses();

        let score = ai.evaluate(&board);
        assert_eq!(score, SCORE_UNIT * CONTINUOUS_MB_LINES * 2 + SCORE_UNIT * BROKEN_MB_LINES * 2);

        board.reset();
        board.do_move(1, 0, 2);
        board.set_status_of(0, 2);
        board.set_status_of(6, 2);

        board.set_status_of(1, 1);
        board.set_status_of(7, 1);

        board.set_status_of(2, 2);
        board.set_status_of(8, 2);

        board.display_mb_statuses();
        let score = ai.evaluate(&board);

        assert_eq!(score, SCORE_UNIT * BROKEN_MB_LINES * 5); // 6 - 1

        board.reset();
        // pointing to uncontestable miniboards
        board.set_status_of(3, 1);
        board.set_status_of(5, 2);

        board.do_move(2, 0, 2);
        board.do_move(7, 0, 2);
        board.do_move(2, 3, 1);

        assert_eq!(
            ai.evaluate(&board),
            (1 - 2) * SCORE_UNIT * UNCONTESTABLE_MB_POINTED_AT
        );
    }
}
