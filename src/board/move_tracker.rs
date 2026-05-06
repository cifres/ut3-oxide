use super::Board;

pub struct TrackedBoard {
    pub board: Board,
    history: Vec<Board>,
}

impl TrackedBoard {
    pub fn new(board: Board) -> Self {
        TrackedBoard {
            board,
            history: Vec::with_capacity(54), // Approx avg max moves per game
        }
    }

    pub fn do_move(&mut self, row: u8, column: u8, player: u8) {
        self.history.push(self.board.clone());
        self.board.do_move(row, column, player);
    }

    pub fn undo_move(&mut self) -> bool {
        let Some(board) = self.history.pop() else {
            eprintln!("No history to undo from");
            return false;
        };

        self.board = board;
        true
    }

    pub fn reset(&mut self) {
        self.board.reset();
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Board, bitflag};
    use super::TrackedBoard;

    #[test]
    fn undo_move() {
        let board = Board::default();
        let mut tracked = TrackedBoard::new(board);

        // ensure undo on the first move doesn't err
        tracked.undo_move();
        tracked.undo_move();

        /* 2 moves: same row, but 2 miniboards of 4 and 5 */
        tracked.do_move(4, 4, 2);
        tracked.do_move(4, 8, 2);
        assert_eq!(
            tracked.board.main_board[4],
            0b00010001000000_100000001000000000
        );
        assert_eq!(tracked.board.main_board[5], 0b10001000000 << 18);

        // partially equivalent to
        assert_eq!(tracked.board.get_player_move_count_of(4, 2), 1);
        assert_eq!(tracked.board.get_player_move_count_of(5, 2), 1);

        tracked.undo_move();
        assert_eq!(
            tracked.board.main_board[4],
            0b00010001000000_000000001000000000
        );
        assert_eq!(tracked.board.main_board[5], 0b0);
        //println!("{board}");

        tracked.undo_move();
        assert_eq!(tracked.board.main_board[4], 0b0);
        //println!("2 moves\n{board}");

        // multiple moves: different rows and miniboards
        /* undo winning move and continue */
        tracked.board.set_status_of(0, 1);
        tracked.board.set_status_of(1, 1);

        tracked.do_move(2, 6, 1);
        tracked.do_move(2, 7, 1);
        tracked.do_move(2, 8, 1);

        assert_eq!(tracked.board.get_status_of(0), 1);
        assert_eq!(tracked.board.get_status_of(1), 1);
        assert_eq!(tracked.board.get_status_of(2), 1);

        assert_eq!(tracked.board.get_player_move_count_of(2, 1), 3);
        assert_eq!(tracked.board.get_total_move_count_of(2), 3);
        assert_eq!(tracked.board.calculate_game_status(), bitflag::STATUS_X_WIN);

        let wincount_before = tracked.board.get_miniboard_win_count_of(1);
        //println!("{board}");

        tracked.undo_move();

        //println!("{board}");
        assert_eq!(
            tracked.board.calculate_game_status(),
            bitflag::STATUS_CONTESTABLE
        );
        assert_eq!(
            tracked.board.get_miniboard_win_count_of(1),
            wincount_before - 1
        );
        assert_eq!(tracked.board.get_status_of(2), 0);
        assert_eq!(tracked.board.get_total_move_count_of(2), 2);
        assert_eq!(tracked.board.get_player_move_count_of(2, 1), 2);

        /* interlaced do/undo moves */
        tracked.reset();
        //assert_eq!(tracked.board.move_history.last_move[0].0, flag::NEW_GAME);

        tracked.do_move(7, 0, 2);
        assert_eq!(tracked.board.get_cell(7, 0), 2);
        tracked.undo_move();
        assert_eq!(tracked.board.get_cell(7, 0), 0);

        tracked.do_move(7, 0, 1);
        tracked.do_move(7, 1, 1);
        tracked.do_move(7, 2, 1);
        tracked.undo_move();

        tracked.do_move(7, 1, 2);
        tracked.do_move(7, 0, 2);
        tracked.do_move(7, 2, 2);
        assert_eq!(tracked.board.get_status_of(6), 2);

        tracked.undo_move();
        tracked.do_move(8, 0, 1);
        tracked.do_move(8, 1, 1);
        tracked.do_move(8, 2, 1);
        assert_eq!(tracked.board.get_status_of(6), 1);

        // full
        tracked.reset();
        tracked.do_move(3, 0, 1);
        tracked.do_move(3, 1, 1);
        tracked.do_move(3, 2, 1);

        tracked.do_move(3, 3, 1);
        tracked.do_move(3, 4, 1);
        tracked.do_move(3, 5, 1);

        tracked.do_move(3, 6, 1);
        tracked.do_move(3, 7, 1);
        tracked.do_move(3, 8, 1);
        assert_eq!(tracked.board.calculate_game_status(), bitflag::STATUS_X_WIN);

        tracked.undo_move();
        assert_eq!(
            tracked.board.calculate_game_status(),
            bitflag::STATUS_CONTESTABLE
        );

        for _ in 0..8 {
            tracked.undo_move();
        }

        assert!(tracked.board.is_first_move());
        assert_eq!(tracked.board.last_move, (0, 0));
        assert_eq!(tracked.board.xo_miniboard_win_count, 0);
        assert_eq!(tracked.board.main_board.iter().sum::<u32>(), 0);

        // try undoing more than moves have been made
        tracked.undo_move();
        tracked.undo_move();
        tracked.undo_move();
        assert_eq!(tracked.board.main_board.iter().sum::<u32>(), 0);

        // then make moves again
        tracked.do_move(8, 8, 2);
        tracked.do_move(8, 7, 2);
        tracked.do_move(8, 6, 2);
        assert_eq!(tracked.board.get_status_of(8), 2);
    }
}
