use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::board::{
    flag::{self, STATUS_CONTESTABLE},
    rules::WINNING_LINES,
    Board,
};

// TODO: adjust weights
// basic multiplier and weight adjustment for what is valued
const SCORE_UNIT: i16 = 10;
const CENTRE_CELL_CONTROL: i16 = 1;
const CELL_CORRESPONDING_SAME_MB: i16 = 2;
const MINIBOARD_WIN_COUNT: i16 = 4;
const CENTRE_MB_CONTROL: i16 = 5;
const UNCONESTABLE_MB_POINTING: i16 = 2;
const NEAR_WON_LINES: i16 = 2;
const NEAR_WON_MB_LINES: i16 = 1; // TODO: split usage into continuous and broken
//const CONTINUOUS_MB_LINES: i16 = 1;
//const BROKEN_MB_LINES: i16 = 1;

const DEPTH: u8 = 5;

// Winning lines composed into continous and broken line patterns
// [(0, 0), (0, 1), (0, 2)] -> [(0, 0), (0, 1), (0, 1), (0, 2), (0, 0), (0, 2)]
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

#[derive(Debug)]
pub struct AI {
    ai_shape: u8,
    opponent_shape: u8,
}

impl Default for AI {
    fn default() -> Self {
        Self::new(flag::O_PLAYER)
    }
}

// #[allow(dead_code)]
impl AI {
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

    pub fn calculate_move_par(&self, board: &Board, depth: u8) -> (u8, u8) {
        let depth = if depth > 0 { depth } else { DEPTH };

        let valid_moves_iterator = board.valid_moves();
        let bstmv = valid_moves_iterator
            .par_bridge()
            .map(|(row, column)| {
                let mut board = board.clone();
                board.do_move(row, column, self.ai_shape);
                let score = self.alphabeta_mm(&mut board, depth - 1, i16::MIN, i16::MAX, false);
                //println!("{score} {row}, {column}");
                // undo move instead of clone
                (score, (row, column))
            })
            .max_by_key(|(score, (_, _))| *score);

        //println!("{bstmv:?}");
        //println!("best {best_move:?} score {best_score} at depth {DEPTH}");
        bstmv.expect("a move should've been selected").1
    }

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
            //for (row, column) in ValidMoveIterator::new(board.valid_moves_bitfield()) {
            for (row, column) in board.valid_moves() {
                let mut board = board.clone();      //NOTE: hot
                board.do_move(row, column, self.ai_shape);
                score = score.max(self.alphabeta_mm(&mut board, depth - 1, alpha, beta, false));
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
                beta = beta.min(score);
                if score <= alpha {
                    break;
                }
            }

            score
        }
    }

    /// Winning = `i16::MAX`, Drawing = 0, Losing = `i16::MIN`
    /// ± score for:
    /// * won/lost miniboards
    /// * centre-control: ai_shape in centre of miniboard and [near-]winning centre miniboard (#4)
    /// * pointing to [near]-won miniboards?
    /// * near-won/lost miniboards/full board
    ///     * continuous relative +/- 10
    ///     * unconnected relative +/- 5
    ///     * interrupted/broken relative +/- 7
    /// * sending to miniboard
    ///
    /// Returns the evaluation of the board state
    #[no_mangle]
    pub fn evaluate(&self, board: &Board) -> i16 {
        // should the main board produce and return miniboard statuses or the AI?
        // for aishape won miniboards += score_mult
        // for oppshape won miniboards -= score_mult

        let mut score = 0;

        // Miniboards won
        score += ((board.get_miniboard_win_count_of(self.ai_shape) as i16)
            - (board.get_miniboard_win_count_of(self.opponent_shape)) as i16)
            * SCORE_UNIT
            * MINIBOARD_WIN_COUNT;

        // println!("@ mbs won {score}");
        // centre-control: board-wide and in individual miniboards
        let centre_status = board.get_status_of(4);
        score += (((centre_status == self.ai_shape) as i16)
            - ((centre_status == self.opponent_shape) as i16))
            * SCORE_UNIT
            * CENTRE_MB_CONTROL;

        // println!("@ centre mb {score}");

        // todo combine crude miniboard checks into one
        // crude individual miniboard check
        // TODO: combine centre cells in one u32 and compare?
        const CENTRE_CELL_OFFSET: u8 = 8; // (col 1 + row 1 * 3) * 2
        for cells in (0..9).map(|n| board.get_miniboard_cells(n)) {
            let mask = 0b11 << CENTRE_CELL_OFFSET;
            let centre_cell = ((cells & mask) >> CENTRE_CELL_OFFSET) as u8;

            score += (((centre_cell == self.ai_shape) as i16)
                - ((centre_cell == self.opponent_shape) as i16))
                * SCORE_UNIT
                * CENTRE_CELL_CONTROL;
        }

        //for miniboard in 0..9 {
        //    let cells = board.get_miniboard_cells(miniboard);
        //    let mask = 0b11 << CENTRE_CELL_OFFSET;
        //    let centre_cell = ((cells & mask) >> CENTRE_CELL_OFFSET) as u8;
        //
        //    score += (((centre_cell == self.ai_shape) as i16)
        //        - ((centre_cell == self.opponent_shape) as i16))
        //        * SCORE_UNIT
        //        * CENTRE_CELL_CONTROL;
        //}

        // println!("@ centre cell {score}");

        // Reverse it: get uncontestable MBs then directly check would-be corresponding cells
        // miniboard 4 (1, 1) -> (1, 1), (1, 4), (1, 7)
        let cells_pointing_to_uncontestable_mbs = (0..9).filter_map(|mb_status| {
            if board.get_status_of(mb_status) == STATUS_CONTESTABLE {
                return None;
            }

            let mut cells = Vec::with_capacity(9);
            //let mut cells_int = 0u32;
            let (mb_row, mb_col) = ((mb_status / 3), (mb_status % 3)); 
            // TODO: use step_by to do it row by row instead of cell by cell
            // 0..9 -> step_by 3 -> row 1 col 1 | 4 | 7 ...
            // TODO: consider packing into a u32
            //for i in (0..9).step_by(3) {
            //}
            for row in 0..3 {
                for col in 0..3 {
                    //println!("{:?}", (row, col, mb_row + row * 3, mb_col + col * 3, mb_status));
                    // board.main_board[row]...
                    let cell_state = board.get_cell(mb_row + row * 3, mb_col + col * 3);
                    cells.push(cell_state);
                    //let offset = ((col) + (row * 3)) * 2;
                    //cells_int |= (cell_state as u32) << offset;
                }
            }

            //println!("{mb_status} -> {cells_int}");
            //println!("{cells:?} vs {cells_int:032b}");
            Some(cells) 
            //Some(cells_int)
        });

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
            //        * UNCONESTABLE_MB_POINTING;
            //    }
            //}

        for uncontestable_mb in cells_pointing_to_uncontestable_mbs {
            for pointing_cell in uncontestable_mb {
                let ai_pointing = pointing_cell == self.ai_shape;
                let opp_pointing = pointing_cell == self.opponent_shape;

                score += (opp_pointing as i16 - ai_pointing as i16)
                    * SCORE_UNIT
                    * UNCONESTABLE_MB_POINTING;
            }
        }

        // for every active miniboard, get its cells and check for near won patterns
        // NOTE: hot: this loop 
        let active_miniboard_cells = (0..9).filter_map(|miniboard| {
            if board.get_total_move_count_of(miniboard) > 0 {
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
                    let offset = (column + row * 3) * 2;
                    let mask = 0b11 << offset;
                    let cell = ((cells & mask) >> offset) as u8;

                    let normalised_offset = i * 2;
                    pattern |= (cell as u16) << normalised_offset;
                    pattern_ai |= (self.ai_shape as u16) << normalised_offset;
                    pattern_opp |= (self.opponent_shape as u16) << normalised_offset;
                }

                let ai_continous = (pattern_ai & 0b1111 == pattern & 0b1111
                    || pattern_ai & (0b1111 << 4) == pattern & (0b1111 << 4))
                    as i16;

                let opp_continous = (pattern_opp & 0b1111 == pattern & 0b1111
                    || pattern_opp & (0b1111 << 4) == pattern & (0b1111 << 4))
                    as i16;

                let ai_unconnected =
                    ((pattern_ai & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;
                let opp_unconnected =
                    ((pattern_opp & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;

                // todo reward continuous_line for ai
                score += ((ai_continous + ai_unconnected) - (opp_continous + opp_unconnected))
                    * SCORE_UNIT
                    * NEAR_WON_MB_LINES;
            }
        }
        // println!("@ uncontestable correspondence {score}");

        //TODO lines: continuous, unconnected  for cells of miniboards, and miniboards of the game
        // continuous pattern strong for 2:  normal winning line: 10 10 10 -> 10 10 [00] OR [00] 10 10
        // continuous pattern weak for 2:  normal winning line: 10 10 10 -> 10 10 [01] OR [01] 10 10
        // unconnected pattern strong for 2:  normal winning line: 10 10 10 -> 10 [00] 10
        // unconnected pattern weak for 2:  normal winning line: 10 10 10 -> 10 [01] 10
        // e.g. winning line [(0, 0), (1, 0), (2, 0)] -> [(0, 0), (1, 0), (1, 0), (2, 0)]
        // 2 wins at this offeset: 1010_1000 & 0b1111 << 4 -> 1010_0000 == 1010_0000 <- 1010_1010 & 0b1111 << 4
        // no one wins at this offset: 1010_0100 & 0b1111 -> 0000_0100 != 0000_1010 <- 1010_1010 & 0b1111

        // for each row of pattern, make u32?
        let mbss = board.get_miniboard_statuses();
        for line in LINE_PATTERNS {
            let mut pattern = 0u16;
            let mut pattern_ai = 0u16;
            let mut pattern_opp = 0u16;
            for (i, (row, column)) in line.iter().enumerate() {
                let miniboard = column + row * 3;
                let status = (mbss & (0b11 << (miniboard * 2))) >> (miniboard * 2);

                let normalised_offset = i * 2;
                pattern |= (status as u16) << normalised_offset;
                pattern_ai |= (self.ai_shape as u16) << normalised_offset;
                pattern_opp |= (self.opponent_shape as u16) << normalised_offset;
            }

            let ai_continous = (pattern_ai & 0b1111 == pattern & 0b1111
                || pattern_ai & (0b1111 << 4) == pattern & (0b1111 << 4))
                as i16;

            let opp_continous = (pattern_opp & 0b1111 == pattern & 0b1111
                || pattern_opp & (0b1111 << 4) == pattern & (0b1111 << 4))
                as i16;

            let ai_unconnected = ((pattern_ai & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;
            let opp_unconnected =
                ((pattern_opp & (0b1111 << 8)) == (pattern & (0b1111 << 8))) as i16;

            // todo reward continuous_line for ai
            score += ((ai_continous + ai_unconnected) - (opp_continous + opp_unconnected))
                * SCORE_UNIT
                * NEAR_WON_LINES;
        }
        // println!("@ continuous/unconnected mb lines {score}");

        /* Sending to free boards */
        // free board = board with a cell that redirects to its board
        // e.g. 0, 0 -> 0, 4, 4 -> mb 4, 8, 8 -> 8
        for row in board.main_board.iter().step_by(4) {
            let first = (row & 0b11) as u8;
            let middle = ((row & (0b11 << 8)) >> 8) as u8;
            let end = ((row & (0b11 << 16)) >> 16) as u8;

            let ai_used_free = (((first == self.ai_shape) as u8)
                + ((middle == self.ai_shape) as u8)
                + ((end == self.ai_shape) as u8)) as i16;

            let opp_used_free = (((first == self.opponent_shape) as u8)
                + ((middle == self.opponent_shape) as u8)
                + ((end == self.opponent_shape) as u8)) as i16; 
            
            //println!("")
            score += (ai_used_free - opp_used_free) * SCORE_UNIT * CELL_CORRESPONDING_SAME_MB;
        }

        let game_status = board.calculate_game_status();
        if game_status == self.opponent_shape {
            score = i16::MIN;
        } else if game_status == self.ai_shape {
            score = i16::MAX;
        }
        
        // println!("final {score}");

        score
    }
}

#[test]
fn ai_evaluate() {
    let ai = AI::default();
    let mut board = Board::new();
    board.do_move(3, 0, 2);

    let score = ai.evaluate(&board);
    assert_eq!(score, 0);

    // winning the centre cell and miniboard
    board.do_move(3, 3, 2);
    board.do_move(4, 4, 2);
    board.do_move(5, 5, 2);
    let _ = board.calculate_game_status();
    let score = ai.evaluate(&board);
    assert_eq!(
        score,
        SCORE_UNIT * CENTRE_MB_CONTROL
            + SCORE_UNIT * CENTRE_CELL_CONTROL
            + SCORE_UNIT * MINIBOARD_WIN_COUNT
            + SCORE_UNIT * NEAR_WON_MB_LINES * 2
            + SCORE_UNIT * CELL_CORRESPONDING_SAME_MB
            - SCORE_UNIT * UNCONESTABLE_MB_POINTING
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
            - SCORE_UNIT * NEAR_WON_MB_LINES * 2
            - SCORE_UNIT * CELL_CORRESPONDING_SAME_MB
            + SCORE_UNIT * UNCONESTABLE_MB_POINTING
    );
    board.reset();

    // winning/losing miniboards
    board.do_move(2, 6, 2);
    board.do_move(2, 7, 2);
    board.do_move(2, 8, 2);
    let _ = board.calculate_game_status();

    board.do_move(3, 6, 1);
    board.do_move(3, 7, 1);
    board.do_move(3, 8, 1);
    let _ = board.calculate_game_status();

    board.do_move(3, 0, 2);
    board.do_move(3, 1, 2);
    board.do_move(3, 2, 2);

    let ai_mbs_won = 2;
    let opp_mbs_won = 1;
    let net_near_won_mb_lines = 4 - 2;

    let _ = board.calculate_game_status();
    let score = ai.evaluate(&board);
    assert_eq!(
        score,
        ((ai_mbs_won - opp_mbs_won) * SCORE_UNIT * MINIBOARD_WIN_COUNT
            + SCORE_UNIT * NEAR_WON_MB_LINES * net_near_won_mb_lines)
    );

    board.reset();

    // near won/lost continous/unconnected lines
    println!("continuous_line test");
    board.do_move(3, 1, 2);
    board.set_status_of(5, 2);
    board.set_status_of(8, 2);

    board.set_status_of(0, 2);
    board.set_status_of(3, 2);

    let score = ai.evaluate(&board);
    assert_eq!(score, SCORE_UNIT * NEAR_WON_LINES * 4);
    board.reset();

    board.do_move(1, 0, 2);
    board.set_status_of(0, 2);
    board.set_status_of(6, 2);

    board.set_status_of(1, 1);
    board.set_status_of(7, 1);

    board.set_status_of(2, 2);
    board.set_status_of(8, 2);

    let score = ai.evaluate(&board);

    // corners * 2?
    assert_eq!(score, 5 * SCORE_UNIT * NEAR_WON_LINES);

    board.reset();
    // pointing to uncontestable miniboards
    board.set_status_of(3, 1);
    board.set_status_of(5, 2);

    board.do_move(2, 0, 2);
    board.do_move(7, 0, 2);
    board.do_move(2, 3, 1);

    assert_eq!(
        ai.evaluate(&board),
        (1 - 2) * SCORE_UNIT * UNCONESTABLE_MB_POINTING
    );
}

#[test]
fn ai_minimax() {
    // let mut board = Board::new();
    // let ai = AI::default();
    //
    // board.do_move(2, 4, 1);
    // let aimove = ai.calculate_move(&board, 6);
    // println!("{aimove:?}");
}


#[test]
fn cells_int() {
    let mut board = Board::new();
    let aix = AI::default();

    board.do_move(4, 4, 1);
    board.do_move(4, 7, 1);
    board.do_move(4, 1, 1);
    board.do_move(1, 1, 2);
    board.set_status_of(4, 1);
    board.set_status_of(5, 2);
    board.set_status_of(0, 1);
    board.set_status_of(8, 1);

    aix.evaluate(&board);
}
