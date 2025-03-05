use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

use crate::board::{Board, flag, WINNING_LINES, ValidMoveIterator};

// basic multiplier and weight adjustment for what is valued
const SCORE_UNIT: i16 = 10;
const CENTRE_CELL_CONTROL: i16 = 2;
const MINIBOARD_WIN_COUNT: i16 = 2;
const CENTRE_MB_CONTROL: i16 = 5;
const NEAR_WON_LINES: i16 = 1;

#[derive(Debug)]
pub struct AI {
    ai_shape: u8,
    opponent_shape: u8
}

impl Default for AI {
    fn default() -> Self {
        Self::new(flag::O_PLAYER)
    }
}

#[allow(dead_code)]
impl AI {
    pub fn new(ai_shape: u8) -> Self {
        //println!("Ai created as {ai_shape}");
        let opponent_shape = if ai_shape == flag::O_PLAYER { flag::X_PLAYER } else { flag::O_PLAYER };
        Self { ai_shape, opponent_shape }
    }

    pub fn calculate_move_par(&self, board: &Board, depth: u8) -> (u8, u8) {
        
        const DEPTH: u8 = 6;
        let depth = depth.max(DEPTH);

        let mut best_move = (0, 0);
        let mut best_score = i16::MIN;

        let valid_move_iterator = ValidMoveIterator::new(board.valid_moves_bitfield());
        let bstmv = valid_move_iterator.par_bridge().map(|(row, column)| {
            let mut board = board.clone();
            board.do_move(row, column, self.ai_shape);
            let score = self.alphabeta_mm(&mut board, depth - 1, false);
            println!("{score} {row}, {column}");
            // undo move instead of clone
            (score, (row, column))
        }).max_by_key(|(score, (_, _))| *score);

        println!("{bstmv:?}");
        //println!("best {best_move:?} score {best_score} at depth {DEPTH}");
        bstmv.expect("a move should've been selected").1
    }

    pub fn calculate_move(&self, board: &Board, depth: u8) -> (u8, u8) {
        
        const DEPTH: u8 = 6;
        let depth = depth.max(DEPTH);

        let mut best_move = (0, 0);
        let mut best_score = i16::MIN;
        for (row, column) in ValidMoveIterator::new(board.valid_moves_bitfield()) {
            let mut board = board.clone();
            board.do_move(row, column, self.ai_shape);
            let score = self.alphabeta_mm(&mut board, depth - 1, false);
            // undo move instead of clone
            best_move = if score > best_score {
                best_score = score;
                (row, column)
            } else {
               best_move 
            }; 

            println!("move {:?} with score {score}", (row, column));
        }

        println!("best {best_move:?} score {best_score} at depth {DEPTH}");
        best_move
    }

    fn alphabeta_mm(&self, board: &mut Board, depth: u8, is_max: bool) -> i16 {
        
        if depth == 0 || board.calculate_game_status() != flag::STATUS_CONTESTABLE {
            return self.evaluate(board);
        }

        if is_max {
            let mut score = i16::MIN;
            for (row, column) in ValidMoveIterator::new(board.valid_moves_bitfield()) {
                let mut board = board.clone();
                board.do_move(row, column, self.ai_shape);
                score = score.max(self.alphabeta_mm(&mut board, depth - 1, false))
            }
            score
        } else {
            let mut score = i16::MAX;
            for (row, column) in ValidMoveIterator::new(board.valid_moves_bitfield()) {
                let mut board = board.clone();
                board.do_move(row, column, self.opponent_shape);
                score = score.min(self.alphabeta_mm(&mut board, depth - 1, true))
            }
            score
        }
    }

    /// Winning = `i16::MAX`, Drawing = 0, Losing = `i16::MIN`
    /// ± score for:
    /// * won/lost miniboards
    /// * centre-control: ai_shape in centre of miniboard and [near-]winning centre miniboard (#4)
    /// * pointing to near-won miniboards?
    /// * near-won/lost miniboards/full board
    ///     * continuous relative +/- 10
    ///     * unconnected relative +/- 5
    ///     * interrupted/broken relative +/- 7 
    ///
    /// Returns the evaluation of the board state 
    pub fn evaluate(&self, board: &Board) -> i16 {
        // should the main board produce and return miniboard statuses or the AI?
        // for aishape won miniboards += score_mult
        // for oppshape won miniboards -= score_mult

        let mut score = 0;

        // Miniboards won 
        score += ((board.get_miniboard_win_count_of(self.ai_shape) as i16)
               - (board.get_miniboard_win_count_of(self.opponent_shape)) as i16)
               * SCORE_UNIT * MINIBOARD_WIN_COUNT;

        //println!("@ mbs won {score}");
        // centre-control: board-wide and in individual miniboards
        let centre_status = board.get_status_of(4);
        score += (((centre_status == self.ai_shape) as i16) 
               - ((centre_status == self.opponent_shape) as i16))
               * SCORE_UNIT * CENTRE_MB_CONTROL;

        //println!("@ centre mb {score}");

        // crude individual miniboard check
        // todo create bitmask of combined miniboards?
        const CENTRE_CELL_OFFSET: u8 = 8;   // (col 1 + row 1 * 3) * 2
        for miniboard in 0..9 {
            let cells = board.get_miniboard_cells(miniboard);
            let mask = 0b11 << CENTRE_CELL_OFFSET;
            let centre_cell = ((cells & mask) >> CENTRE_CELL_OFFSET) as u8;

            score += (((centre_cell == self.ai_shape) as i16)
                   - ((centre_cell == self.opponent_shape) as i16))
                   * SCORE_UNIT * CENTRE_CELL_CONTROL;
        }

        //println!("@ centre cell {score}");

        //TODO lines: continuous, unconnected  for cells of miniboards, and miniboards of the game
        // continuous pattern strong for 2:  normal winning line: 10 10 10 -> 10 10 [00] OR [00] 10 10 
        // continuous pattern weak for 2:  normal winning line: 10 10 10 -> 10 10 [01] OR [01] 10 10 
        // unconnected pattern strong for 2:  normal winning line: 10 10 10 -> 10 [00] 10   
        // unconnected pattern weak for 2:  normal winning line: 10 10 10 -> 10 [01] 10   
        // bit mask in then bitshift by offset 
        // loop over winning lines and put each pattern offset by the index * 2

        // TODO: 
        // todo: do cells of every miniboard 
        // implicitly biases corners and centre miniboard due to increased intersectionability
        for line in WINNING_LINES {
            let broken_items = [line[0], line[2]];
            let broken_line = broken_items.iter();

            let continous_broken_lines = line[..2]
                .iter()
                .chain(line[1..].iter().rev())
                .chain(broken_line);

            //println!("\n{continous_broken_lines:?}");
            let mut pattern = 0u16;
            let mut pattern_ai = 0u16;
            let mut pattern_opp = 0u16;
            for (i, (row, column)) in continous_broken_lines.enumerate() {
                let miniboard = column + row * 3;
                let status = board.get_status_of(miniboard);

                let offset = i * 2; 
                //println!("{i} -> {offset} - {row} {column} -- {status:02b}");
                pattern |= (status as u16) << offset;
                pattern_ai |= (self.ai_shape as u16) << offset;
                pattern_opp |= (self.opponent_shape as u16) << offset;
            }

            // we split the pattern by left/right bitshifting to compare the first/last two
            // winning line coordinates 
            // e.g. winning line [(0, 0), (1, 0), (2, 0)] -> [(0, 0), (1, 0), (1, 0), (2, 0)]
            // the masked in miniboard status are for 2 i.e. 0b10
            // this could be: 1010_1000 & 0b1111 = 0000_1010 -- forms a near-won line
            // this could be: 1010_1000 & 0b1111 << 4 = 1000_0000 -- does not form a near-won line

            let ai_continous = (pattern_ai & 0b1111 == pattern & 0b1111
                || pattern_ai & (0b1111 << 4) == pattern & (0b1111 << 4))
                as i16;

            let opp_continous = (pattern_opp & 0b1111 == pattern & 0b1111
                || pattern_opp & (0b1111 << 4) == pattern & (0b1111 << 4))
                as i16;

            let ai_unconnected = ((pattern_ai & (0b1111 << 8))  == (pattern & (0b1111 << 8))) as i16;
            let opp_unconnected = ((pattern_opp & (0b1111 << 8))  == (pattern & (0b1111 << 8))) as i16;

            score +=
                ((ai_continous + ai_unconnected) - (opp_continous + opp_unconnected)) * SCORE_UNIT * NEAR_WON_LINES;

            //println!("{pattern:016b} {pattern_ai:016b} {score} con {ai_continous} unconn {ai_unconnected} {line:?}");
        }

        //println!("@ continuous/unconnected mb lines {score}");

        // win/lose -- find way to not need to clone board -- must not use &mut board ideally
        let game_status = board.calculate_game_status();
        if game_status == self.opponent_shape {
            score = i16::MIN;
        } else if game_status == self.ai_shape {
            score = i16::MAX;
        }

        //println!("final {score}");

        score
    }
}

    //function alphabeta(node, depth, α, β, maximizingPlayer) is
    //    if depth == 0 or node is terminal then
    //        return the heuristic value of node
    //    if maximizingPlayer then
    //        value := −∞
    //        for each child of node do
    //            value := max(value, alphabeta(child, depth − 1, α, β, FALSE))
    //            α := max(α, value)
    //            if value ≥ β then
    //                break (* β cutoff *)
    //        return value
    //    else
    //        value := +∞
    //        for each child of node do
    //            value := min(value, alphabeta(child, depth − 1, α, β, TRUE))
    //            β := min(β, value)
    //            if value ≤ α then
    //                break (* α cutoff *)
    //        return value
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
    assert_eq!(score, SCORE_UNIT * CENTRE_MB_CONTROL + SCORE_UNIT * CENTRE_CELL_CONTROL + SCORE_UNIT * MINIBOARD_WIN_COUNT); 
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

    let _ = board.calculate_game_status();
    let score = ai.evaluate(&board);
    assert_eq!(
        score,
        ((ai_mbs_won - opp_mbs_won)
        * SCORE_UNIT 
        * MINIBOARD_WIN_COUNT)
    );

    board.reset();

    // near won/lost continous/unconnected lines 
    println!("continuous_line test");
    board.do_move(3, 0, 2);
    board.set_status_of(5, 2);
    board.set_status_of(8, 2);

    board.set_status_of(0, 2);
    board.set_status_of(3, 2);

    let score = ai.evaluate(&board);
    assert_eq!(
        score, 
        SCORE_UNIT * NEAR_WON_LINES * 4 
    );
    board.reset();

    board.do_move(0, 0, 2);
    board.set_status_of(0, 2);
    board.set_status_of(6, 2);

    board.set_status_of(1, 1);
    board.set_status_of(7, 1);

    board.set_status_of(2, 2);
    board.set_status_of(8, 2);

    let score = ai.evaluate(&board);

    // corners * 2?
    assert_eq!(score, 5 * SCORE_UNIT * NEAR_WON_LINES);
}

#[test]
fn ai_minimax() {
    let mut board = Board::new();
    let ai = AI::default();
    
    board.do_move(2, 4, 1);
    let aimove = ai.calculate_move(&board, 8);
    println!("{aimove:?}");
}
