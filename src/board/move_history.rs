use std::hint;

/// Stores the `Board`'s history of moves
/// This enables undoing moves.
/// Uses separate arrays/vecs to track the row affected, and its index.
#[derive(Debug, Clone)]
pub struct MoveHistory {
    move_row_affected: Vec<u32>,
    move_row_affected_index: Vec<u8>,
    miniboard_affected: Vec<u32>,
    miniboard_affected_index: Vec<u8>,
    last_move: Vec<(u8, u8)>,
    xo_miniboard_win_count: Vec<u8>,
}

impl MoveHistory {
    pub fn new() -> Self {
        let mut move_row_affected = Vec::with_capacity(81);
        let mut move_row_affected_index = Vec::with_capacity(81);
        let mut miniboard_affected = Vec::with_capacity(81);
        let mut miniboard_affected_index = Vec::with_capacity(81);
        let mut last_move = Vec::with_capacity(81);
        let mut xo_miniboard_win_count = Vec::with_capacity(81);

        move_row_affected.push(0);
        move_row_affected_index.push(0);
        miniboard_affected.push(0);
        miniboard_affected_index.push(0);
        last_move.push((0, 0));
        xo_miniboard_win_count.push(0);

        Self {
            move_row_affected,
            move_row_affected_index,
            miniboard_affected,
            miniboard_affected_index,
            last_move,
            xo_miniboard_win_count,
        }
    }

    pub fn add(
        &mut self,
        move_row: u32,
        row_index: u8,
        miniboard_row: u32,
        miniboard_index: u8,
        last_move: (u8, u8),
        win_count: u8,
    ) {
        self.move_row_affected.push(move_row);
        self.move_row_affected_index.push(row_index);
        self.miniboard_affected.push(miniboard_row);
        self.miniboard_affected_index.push(miniboard_index);
        self.last_move.push(last_move);
        self.xo_miniboard_win_count.push(win_count);
    }

    // TODO: refactor into simpler type
    pub fn pop(&mut self) -> Option<(u32, u8, u32, u8, (u8, u8), u8)> {
        let row = self.move_row_affected.pop();
        let row_index = self.move_row_affected_index.pop();
        let miniboard = self.miniboard_affected.pop();
        let miniboard_index = self.miniboard_affected_index.pop();
        let last_move = self.last_move.pop();
        let win_count = self.xo_miniboard_win_count.pop();

        Some((
            row?,
            row_index?,
            miniboard?,
            miniboard_index?,
            last_move?,
            win_count?,
        ))
    }

    pub fn reset(&mut self) {
        self.move_row_affected.clear();
        self.move_row_affected_index.clear();
        self.miniboard_affected.clear();
        self.miniboard_affected_index.clear();
        self.last_move.clear();
        self.xo_miniboard_win_count.clear();

        self.move_row_affected.push(0);
        self.move_row_affected_index.push(0);
        self.miniboard_affected.push(0);
        self.miniboard_affected_index.push(0);
        self.last_move.push((0, 0));
        self.xo_miniboard_win_count.push(0);
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{flag, Board};

    #[test]
    fn undo_move() {
        let mut board = Board::new();

        /* 2 moves: same row, but 2 miniboards of 4 and 5 */
        board.do_move(4, 4, 2);
        board.do_move(4, 8, 2);
        assert_eq!(board.main_board[4], 0b000010001000000100000001000000000);
        assert_eq!(board.main_board[5], 0b10001000000 << 18);

        // equivalent to
        assert_eq!(board.get_player_move_count_of(4, 2), 1);
        assert_eq!(board.get_player_move_count_of(5, 2), 1);

        board.undo_move();
        assert_eq!(board.main_board[4], 0b00010001000000_000000001000000000);
        assert_eq!(board.main_board[5], 0b0);
        println!("{board}");

        board.undo_move();
        assert_eq!(board.main_board[4], 0b0);
        println!("2 moves\n{board}");

        // multiple moves: different rows and miniboards
        //board.do_move(4, 3, 2);
        //board.do_move(4, 3, 2);

        /* undo winning move and continue */
        board.set_status_of(0, 1);
        board.set_status_of(1, 1);

        board.do_move(2, 6, 1);
        board.do_move(2, 7, 1);
        board.do_move(2, 8, 1);

        assert_eq!(board.get_player_move_count_of(2, 1), 3);
        assert_eq!(board.get_total_move_count_of(2), 3);
        assert_eq!(board.get_total_move_count_of(2), 3);
        assert_eq!(board.calculate_game_status(), flag::STATUS_X_WIN);

        let wincount_before = board.get_miniboard_win_count_of(1);

        board.undo_move();
        assert_eq!(board.calculate_game_status(), flag::STATUS_CONTESTABLE);
        assert_eq!(board.get_miniboard_win_count_of(1), wincount_before - 1);

        /* interlaced do/undo moves */
        assert_ne!(board.move_history.miniboard_affected.len(), 0);
        board.reset();
        assert_eq!(board.move_history.miniboard_affected.len(), 1);
        assert_eq!(board.move_history.last_move[0].0, flag::NEW_GAME);

        board.do_move(7, 0, 2);
        assert_eq!(board.get_cell(7, 0), 2);
        board.undo_move();
        assert_eq!(board.get_cell(7, 0), 0);

        board.do_move(7, 0, 1);
        board.do_move(7, 1, 1);
        board.undo_move();

        board.do_move(7, 1, 2);
        board.do_move(7, 0, 2);
        board.do_move(7, 2, 2);
        assert_eq!(board.get_status_of(6), 2);

        board.undo_move();
        board.do_move(8, 0, 1);
        board.do_move(8, 1, 1);
        board.do_move(8, 2, 1);
        assert_eq!(board.get_status_of(6), 1);
    }
}
