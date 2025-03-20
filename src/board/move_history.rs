type Move = (u8, u8);
/// Stores the `Board`'s history of moves
/// This enables undoing moves.
/// Uses separate arrays/vecs to track the row affected, and its index.
/// Snapshots the relevant board state immediately before `board.do_move` to surgically revert changes
#[derive(Debug, Clone)]
pub struct MoveHistory {
    i: u8,
    cell: [Move; 81],
    last_move: [Move; 81],
    miniboard_index: [u8; 81],
    miniboard_status: [u8; 81],
    miniboard_move_count: [u16; 81], // both x/o player, and total counts
    xo_miniboard_win_count: [u8; 81],
}

// Surgically store the state that is about change including the affected cell,
// miniboard status & idx, player & total move count, last move, xo miniboard win count
impl MoveHistory {
    pub fn new() -> Self {
        let cell = [(0, 0); 81];
        let last_move = [(0, 0); 81];
        let miniboard_index = [0; 81];
        let miniboard_status = [0; 81];
        let miniboard_move_count = [0; 81];
        let xo_miniboard_win_count = [0; 81];

        Self {
            i: 0,
            cell,
            last_move,
            miniboard_index,
            miniboard_status,
            miniboard_move_count,
            xo_miniboard_win_count,
        }
    }

    pub fn add(
        &mut self,
        cell: Move,
        last_move: Move,
        mb_index: u8,
        mb_status: u8,
        mb_move_count: u16,
        xo_mb_win_count: u8,
    ) {
        self.i += 1;

        self.cell[self.i as usize] = cell;
        self.last_move[self.i as usize] = last_move;
        self.miniboard_index[self.i as usize] = mb_index;
        self.miniboard_status[self.i as usize] = mb_status;
        self.miniboard_move_count[self.i as usize] = mb_move_count;
        self.xo_miniboard_win_count[self.i as usize] = xo_mb_win_count;
    }

    // TODO: refactor into simpler type
    // Ensure zeroed values return if we try to undo on the first move
    pub fn pop(&mut self) -> Option<(Move, Move, u8, u8, u16, u8)> {
        let i = self.i;
        if i == 0 {
            None
        } else {
            self.i -= 1;
            Some((
                self.cell[i as usize],
                self.last_move[i as usize],
                self.miniboard_index[i as usize],
                self.miniboard_status[i as usize],
                self.miniboard_move_count[i as usize],
                self.xo_miniboard_win_count[i as usize],
            ))
        }
    }

    pub fn reset(&mut self) {
        self.i = 0;
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
        let mut board = Board::new(true);

        /* 2 moves: same row, but 2 miniboards of 4 and 5 */
        board.do_move(4, 4, 2);
        board.do_move(4, 8, 2);
        assert_eq!(board.main_board[4], 0b000010001000000100000001000000000);
        assert_eq!(board.main_board[5], 0b10001000000 << 18);

        // partially equivalent to
        assert_eq!(board.get_player_move_count_of(4, 2), 1);
        assert_eq!(board.get_player_move_count_of(5, 2), 1);

        board.undo_move();
        assert_eq!(board.main_board[4], 0b00010001000000_000000001000000000);
        assert_eq!(board.main_board[5], 0b0);
        //println!("{board}");

        board.undo_move();
        assert_eq!(board.main_board[4], 0b0);
        //println!("2 moves\n{board}");

        // multiple moves: different rows and miniboards
        /* undo winning move and continue */
        board.set_status_of(0, 1);
        board.set_status_of(1, 1);

        board.do_move(2, 6, 1);
        board.do_move(2, 7, 1);
        board.do_move(2, 8, 1);

        assert_eq!(board.get_status_of(0), 1);
        assert_eq!(board.get_status_of(1), 1);
        assert_eq!(board.get_status_of(2), 1);

        assert_eq!(board.get_player_move_count_of(2, 1), 3);
        assert_eq!(board.get_total_move_count_of(2), 3);
        assert_eq!(board.calculate_game_status(), flag::STATUS_X_WIN);

        let wincount_before = board.get_miniboard_win_count_of(1);
        //println!("{board}");

        board.undo_move();

        //println!("{board}");
        assert_eq!(board.calculate_game_status(), flag::STATUS_CONTESTABLE);
        assert_eq!(board.get_miniboard_win_count_of(1), wincount_before - 1);
        assert_eq!(board.get_status_of(2), 0);
        assert_eq!(board.get_total_move_count_of(2), 2);
        assert_eq!(board.get_player_move_count_of(2, 1), 2);

        /* interlaced do/undo moves */
        assert_ne!(board.move_history.as_ref().unwrap().i, 0);
        board.reset();
        assert_eq!(board.move_history.as_ref().unwrap().i, 0);
        //assert_eq!(board.move_history.last_move[0].0, flag::NEW_GAME);

        board.do_move(7, 0, 2);
        assert_eq!(board.get_cell(7, 0), 2);
        board.undo_move();
        assert_eq!(board.get_cell(7, 0), 0);

        board.do_move(7, 0, 1);
        board.do_move(7, 1, 1);
        board.do_move(7, 2, 1);
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

        // full
        board.reset();
        board.do_move(3, 0, 1);
        board.do_move(3, 1, 1);
        board.do_move(3, 2, 1);

        board.do_move(3, 3, 1);
        board.do_move(3, 4, 1);
        board.do_move(3, 5, 1);

        board.do_move(3, 6, 1);
        board.do_move(3, 7, 1);
        board.do_move(3, 8, 1);

        assert_eq!(board.calculate_game_status(), flag::STATUS_X_WIN);

        board.undo_move();
        assert_eq!(board.calculate_game_status(), flag::STATUS_CONTESTABLE);

        for _ in 0..8 {
            board.undo_move()
        }

        assert_eq!(board.last_move, (0, 0));
        assert_eq!(board.xo_miniboard_win_count, 0);
        assert_eq!(board.main_board.iter().sum::<u32>(), 0);

        // try undoing more than moves have been made
        board.undo_move();
        board.undo_move();
        board.undo_move();
        assert_eq!(board.main_board.iter().sum::<u32>(), 0);

        // then make moves again
        board.do_move(8, 8, 2);
        board.do_move(8, 7, 2);
        board.do_move(8, 6, 2);
        assert_eq!(board.get_status_of(8), 2);
    }
}
