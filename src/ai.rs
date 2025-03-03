use crate::board::{Board, flag};

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
        println!("Ai created as {ai_shape}");
        let opponent_shape = if ai_shape == flag::O_PLAYER { flag::X_PLAYER } else { flag::O_PLAYER };
        Self { ai_shape, opponent_shape }
    }

    pub fn calculate_move(board: &Board) -> (u8, u8) {
        let mut board = board.clone();
        let status = board.calculate_game_status(); 
        println!("status from ai: {status}");
        todo!()
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
        // should the miniboard produce and return miniboard statuses or the AI?
        // for aishape won miniboards += score_mult
        // for oppshape won miniboards -= score_mult

        // basic multiplier and weight adjustment for what is valued
        const SCORE_UNIT: i16 = 10;
        const CENTRE_CELL_CONTROL: i16 = 2;
        const MINIBOARD_WIN_COUNT: i16 = 1;
        const CENTRE_MB_CONTROL: i16 = 3;

        let mut score = 0;

        // Miniboards won 
        score += ((board.get_miniboard_win_count_of(self.ai_shape)
               - board.get_miniboard_win_count_of(self.opponent_shape)) as i16)
               * SCORE_UNIT * MINIBOARD_WIN_COUNT;

        // centre-control: board-wide and in individual miniboards
        let centre_status = board.get_status_of(4);
        score += (((centre_status == self.ai_shape) as i16) 
               - ((centre_status == self.opponent_shape) as i16))
               * SCORE_UNIT * CENTRE_MB_CONTROL;

        //println!("centre control {score} status {centre_status}");
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

        //TODO lines: continuous, unconnected -> broken

        println!("{score}");

        score
    }

    fn minimax(board: &Board, depth: u8) -> (u8, u8) {
        // default 6
        let depth = if depth > 0 { depth } else { 6 };
        todo!()
    }
}

#[test]
fn ai_evaluate() {
    let ai = AI::default();
    let mut board = Board::new();        
    
    let score = ai.evaluate(&board);
    assert_eq!(score, 0);

    board.set_status_of(4, 2);

    let score = ai.evaluate(&board);
    assert_eq!(score, 30);
}
