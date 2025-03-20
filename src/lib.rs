pub mod board;
pub mod ai;

#[cfg(test)]
mod tests {
    use crate::board::{flag, Board, iterator};
    use crate::ai::AI;

    #[test]
    fn set_meta_data() {
        let mut board = Board::default();

        board.do_move(3, 3, 1);
        board.do_move(3, 4, 1);
        board.do_move(3, 5, 1);

        board.do_move(4, 3, 1);
        board.do_move(4, 4, 1);
        board.do_move(4, 5, 1);

        board.do_move(5, 3, 2);
        board.do_move(5, 4, 2);
        board.do_move(5, 5, 2);

        let miniboard = Board::move_miniboard(3, 3);
        let movecount = board.get_total_move_count_of(miniboard);
        let left_miniboard_movecount = board.get_total_move_count_of(miniboard - 1);
        let right_miniboard_movecount = board.get_total_move_count_of(miniboard + 1);

        assert_eq!(movecount, 9, "move count was {movecount} for minboard {miniboard} when it should be 9");
        assert_eq!(left_miniboard_movecount, 0);
        assert_eq!(right_miniboard_movecount, 0);
    }
    /// validate moves by ensuring that invalidity if:
    /// 1) cell is occupied
    /// 2) miniboard is 'uncontested' i.e. won by X or O, or drawn
    /// 3) miniboard coords don't correspond to previous move
    #[test]
    fn is_valid_move_test() {
        let mut board = Board::default();

        // 1) cell is occupied
        board.do_move(5, 5, 1);
        assert!(!board.is_valid_move(5, 5));

        assert!(!board.is_valid_move(1, 7));

        assert!(board.is_valid_move(6, 8));
        assert!(board.is_valid_move(6, 7));

        // 2) corresponding miniboard is uncontestable, so all other valid miniboards and cells are playable
        // provided that the selected miniboard and cell are contestable and empty respectively.

        let corresponding_miniboard = Board::move_corresponding_miniboard(5, 5);
        let uncontestable_corresponding_miniboard = corresponding_miniboard;
        let uncontestable_miniboard = 3;
        let occuipied_cell = (0, 2);

        board.set_cell(occuipied_cell.0, occuipied_cell.1, 2);
        board.set_status_of(uncontestable_corresponding_miniboard, flag::STATUS_X_WIN);
        board.set_status_of(uncontestable_miniboard, flag::STATUS_X_WIN);

        let validcells = board.valid_moves_bitfield();
        //println!("validmask = {validmask:b}");

        for row in 0..9 {
            for column in 0..9 {
                let move_miniboard = Board::move_miniboard(row, column);
                if (row, column) == (5, 5)
                    || (row, column) == occuipied_cell 
                    || move_miniboard == uncontestable_miniboard
                    || move_miniboard == uncontestable_corresponding_miniboard
                {
                    assert!(!board.is_valid_move(row, column), "{row} {column} was valid but shouldn't be");
                    let pos = column + row * 9;
                    let validcell = (validcells >> pos) & 1;
                    assert!(validcell == 0, "{row} {column} at {pos} was valid but shouldn't be");
                    continue;
                }

                assert!(board.is_valid_move(row, column), "{row} {column} was invalid but should be valid");
                let pos = column + row * 9;
                let isvalidcell = (validcells >> pos) & 1;
                assert!(isvalidcell == 1, "{row} {column} at {pos} was valid but should be valid");
            }
        }

        board.reset();

        // 3) miniboard coordinate correspondence/matching
        let (_row, _column) = (2, 2);
        let corresponding_miniboard = Board::move_corresponding_miniboard(_row, _column);
        assert_eq!(corresponding_miniboard, 8);

        assert!(board.is_valid_move(_row, _column));
        board.do_move(_row, _column, 1);

        assert!(board.is_valid_move(7, 7));
        board.reset();

        // 4) exception of uncontestable minboard to play any other valid miniboard
        let (_row, _column) = (1, 1);
        let move_corresponding = Board::move_corresponding_miniboard(_row, _column);
        board.do_move(_row, _column, 1);
        board.set_status_of(move_corresponding, flag::STATUS_X_WIN);

        for row in 0..9 {
            for col in 0..9 {
                if Board::move_miniboard(row, col) == move_corresponding
                    || (row, col) == (_row, _column)
                {
                    println!("{row}, {col}");
                    assert!(!board.is_valid_move(row, col));
                    continue;
                }

                assert!(board.is_valid_move(row, col));
            }
        }
    }

    #[test]
    fn get_cells() {
        let mut board = Board::default();

        // fill miniboard 4 by filling rows and columns 3, 4, and 5.
        board.set_cell(3, 3, 1);
        board.set_cell(3, 4, 2);
        board.set_cell(3, 5, 1);
 
        board.set_cell(4, 3, 1);
        board.set_cell(4, 4, 1);
        board.set_cell(4, 5, 1);

        board.set_cell(5, 3, 2);
        board.set_cell(5, 4, 2);
        board.set_cell(5, 5, 2);

        let cells = board.get_miniboard_cells(4);

        println!("{cells:032b} = {cells}");

        assert_eq!(cells, 0b101010_010101_011001);
    }

    #[test]
    fn check_miniboard_status() {
        let mut board = Board::default();

        /* row-wise horizontal line: 0 */
        board.do_move(0, 0, 2);
        assert!(!board.calculate_miniboard_status(0));
        assert_eq!(board.get_status_of(0), flag::STATUS_CONTESTABLE);

        board.do_move(0, 1, 2);
        board.do_move(0, 2, 2);

        let miniboard = Board::move_miniboard(0, 0);
        let win = board.calculate_miniboard_status(miniboard);
        let winner = board.get_status_of(miniboard);
        assert!(win);
        assert_eq!(winner, flag::STATUS_O_WIN);
        board.reset();

        // row: 1  
        board.do_move(1, 3, 1);
        board.do_move(1, 4, 1);
        board.do_move(1, 5, 1);

        let miniboard = Board::move_miniboard(1, 3);
        let win = board.calculate_miniboard_status(miniboard);
        let winner = board.get_status_of(miniboard);
        assert!(win);
        assert_eq!(winner, flag::STATUS_X_WIN);
        board.reset();

        // row: 2  
        // x win
        board.do_move(2, 6, 1);
        board.do_move(2, 7, 1);
        board.do_move(2, 8, 1);

        let miniboard = Board::move_miniboard(2, 6);
        let win = board.calculate_miniboard_status(miniboard);
        let winner = board.get_status_of(miniboard);
        assert!(win);
        assert_eq!(winner, flag::STATUS_X_WIN);

        // o win
        board.do_move(2, 3, 2);
        board.do_move(2, 4, 2);
        board.do_move(2, 5, 2);

        let miniboard = Board::move_miniboard(2, 3);
        let win = board.calculate_miniboard_status(miniboard);
        let winner = board.get_status_of(miniboard);
        assert!(win);
        assert_eq!(winner, flag::STATUS_O_WIN);

        board.reset();

        /* column wise, vertical line: 0 */
        board.do_move(3, 0, 2);
        board.do_move(4, 0, 2);
        board.do_move(5, 0, 2);

        let miniboard = Board::move_miniboard(3, 0);
        let win = board.calculate_miniboard_status(miniboard);
        assert!(win);
        board.reset();
        
        // column: 1
        board.do_move(3, 4, 1);
        board.do_move(4, 4, 1);
        board.do_move(5, 4, 1);

        let miniboard = Board::move_miniboard(3, 4);
        let win = board.calculate_miniboard_status(miniboard);
        let winner = board.get_status_of(miniboard);
        assert!(win);
        assert_eq!(winner, flag::STATUS_X_WIN);
        board.reset();

        // column: 2
        board.do_move(3, 8, 2);
        board.do_move(4, 8, 2);
        board.do_move(5, 8, 2);

        let miniboard = Board::move_miniboard(3, 8);
        let win = board.calculate_miniboard_status(miniboard);
        let winner = board.get_status_of(miniboard);
        assert!(win);
        assert_eq!(winner, flag::STATUS_O_WIN);
        board.reset();

        // diagonal: 0
        // A winning line found with 9 moves is still a win, not a draw
        board.do_move(6, 6, 1);
        board.do_move(6, 7, 2);
        board.do_move(6, 8, 1);

        board.do_move(7, 6, 2);
        board.do_move(7, 7, 1);
        board.do_move(7, 8, 2);

        board.do_move(8, 6, 2);
        board.do_move(8, 7, 1);
        board.do_move(8, 8, 2);

        let miniboard = Board::move_miniboard(6, 6);
        let status_changed = board.calculate_miniboard_status(miniboard);
        let status = board.get_status_of(miniboard);
        println!("movecount: {:?}", board.get_total_move_count_of(miniboard));
        assert!(status_changed);
        assert_eq!(status, flag::STATUS_DRAW);

        // 9 moves is a draw but ensure priority to win checking
        // by changing last 3 rows to make x win and have 9 moves made 
        //board.set_player_move_count_of(miniboard, 3, 1);
        //board.set_player_move_count_of(miniboard, 3, 2);
        board.set_total_move_count_of(miniboard, 6);
        board.set_status_of(miniboard, flag::STATUS_CONTESTABLE);
        board.do_move(8, 6, 1);
        board.do_move(8, 7, 2);
        board.do_move(8, 8, 1);
        
        let move_count = board.get_total_move_count_of(miniboard);
        let status_changed = board.calculate_miniboard_status(miniboard);
        let status = board.get_status_of(miniboard);
        assert_eq!(move_count, 9);
        assert!(status_changed);
        assert_eq!(status, flag::STATUS_X_WIN);
        board.reset();

        /* diagonal line */
        // diagonal: 1
        board.do_move(6, 2, 1);
        board.do_move(7, 1, 1);
        board.do_move(8, 0, 1);

        let miniboard = Board::move_miniboard(6, 2);
        let win = board.calculate_miniboard_status(miniboard);
        let winner = board.get_status_of(miniboard);
        assert!(win);
        assert_eq!(winner, flag::STATUS_X_WIN);
        board.reset();

        /* combined test*/
        // contestable board with Xs and Os is won by X;
        let miniboard = Board::move_miniboard(3, 4);
        board.do_move(3, 3, 1);
        board.do_move(3, 4, 2);
        board.do_move(3, 5, 2);
        assert!(!board.calculate_miniboard_status(miniboard));

        board.do_move(4, 3, 2);
        board.do_move(4, 4, 1);
        assert!(!board.calculate_miniboard_status(miniboard));

        board.do_move(5, 5, 1);
        assert!(board.calculate_miniboard_status(miniboard));
        assert_eq!(board.get_status_of(miniboard), flag::STATUS_X_WIN);
        board.reset();

        /* move count > 6 == win test */
        board.do_move(0, 0, 1);
        board.do_move(0, 1, 1);
        board.do_move(1, 0, 1);

        board.do_move(1, 2, 1);
        board.do_move(2, 1, 1);
        board.do_move(2, 2, 1);

        // player 2 makes a move but it makes no difference
        board.do_move(1, 1, 2);

        board.do_move(0, 2, 1);
        assert_eq!(board.get_player_move_count_of(0, flag::X_PLAYER), 7);
        assert_eq!(board.get_status_of(0), flag::STATUS_X_WIN);

    }

    #[test]
    fn get_game_status() {
        let mut board = Board::default();
        let ai = AI::default();

        // x win -- miniboards 3, 4, 5 
        board.do_move(3, 0, 1);
        board.do_move(3, 1, 1);
        board.do_move(3, 2, 1);
        board.calculate_miniboard_status(3);
        assert_eq!(board.calculate_game_status(), flag::STATUS_CONTESTABLE);
        ai.evaluate(&board);

        board.do_move(3, 4, 1);
        board.do_move(4, 4, 1);
        board.do_move(5, 4, 1);
        board.calculate_miniboard_status(4);
        ai.evaluate(&board);

        board.do_move(3, 6, 1);
        board.do_move(4, 7, 1);
        board.do_move(5, 8, 1);
        board.calculate_miniboard_status(5);
        ai.evaluate(&board);

        let gamestatus = board.calculate_game_status();
        ai.evaluate(&board);
        assert_eq!(gamestatus, flag::STATUS_X_WIN);

        board.do_move(5, 7, 2);
        let gamestatus = board.calculate_game_status();
        assert_eq!(gamestatus, flag::STATUS_CONTESTABLE);

        // o win
        // pretend x didn't form a line in the 5th miniboard and test 
        board.set_cell(5, 8, 0);
        board.set_status_of(5, flag::STATUS_CONTESTABLE);

        board.do_move(2, 6, 2);
        board.do_move(2, 7, 2);
        board.do_move(2, 8, 2);
        board.calculate_game_status();

        board.do_move(3, 6, 2);
        board.do_move(3, 7, 2);
        board.do_move(3, 8, 2);
        board.calculate_game_status();

        board.do_move(7, 6, 2);
        board.do_move(7, 7, 2);
        board.do_move(7, 8, 2);
        let gamestatus = board.calculate_game_status();
        let o_mbwincount = board.get_miniboard_win_count_of(2);

        assert_eq!(gamestatus, flag::STATUS_O_WIN);
        assert_eq!(o_mbwincount, 3);

        // draw: every cell full, or every miniboard is uncontestable 
        //board.do_move(row, column, xoshape);
        println!("\ndraw test game status");
        board.reset();
        // all boards drawn

        for i in 0..9 {
            board.set_status_of(i, flag::STATUS_DRAW);
        }
        // artificially move 4, 4 to trigger scanning every miniboard
        // because 4, 4 is in miniboard 4 which intersects all other miniboards
        board.do_move(4, 4, 2);
        assert_eq!(board.calculate_game_status(), flag::STATUS_DRAW);

        // all miniboards are uncontestable (won/drawn for X/O not in a winning line)
        board.set_status_of(0, flag::STATUS_X_WIN);
        board.set_status_of(1, flag::STATUS_O_WIN);
        board.set_status_of(2, flag::STATUS_X_WIN);

        board.set_status_of(3, flag::STATUS_X_WIN);
        board.set_status_of(4, flag::STATUS_O_WIN);
        board.set_status_of(5, flag::STATUS_X_WIN);

        board.set_status_of(6, flag::STATUS_O_WIN);
        board.set_status_of(7, flag::STATUS_DRAW);
        board.set_status_of(8, flag::STATUS_O_WIN);

        board.set_miniboard_win_count_of(flag::O_PLAYER, 4);
        board.do_move(4, 4, 2);
        assert!(board.calculate_miniboard_status(4));
        assert_ne!(board.get_status_of(4), flag::STATUS_CONTESTABLE);
        assert_eq!(board.calculate_game_status(), flag::STATUS_DRAW);

        // almost/fully drawn
        board.reset();
        for miniboard in 0..9 {
            board.set_status_of(miniboard, flag::STATUS_DRAW);
        }
        board.do_move(8, 8, 1);
        assert_eq!(board.calculate_game_status(), flag::STATUS_DRAW);

        // almost
        board.set_status_of(8, flag::STATUS_X_WIN);
        board.set_miniboard_win_count_of(flag::X_PLAYER, 1);
        assert_eq!(board.calculate_game_status(), flag::STATUS_DRAW);
    }

    #[test]
    fn miniboard_win_count() {
        let mut board = Board::default();

        let xwincount = board.get_miniboard_win_count_of(1);
        let owincount = board.get_miniboard_win_count_of(2);
        assert_eq!(xwincount, 0);
        assert_eq!(owincount, 0);

        board.set_miniboard_win_count_of(1, 5);
        assert_eq!(board.get_miniboard_win_count_of(1), 5);
        assert_eq!(board.get_miniboard_win_count_of(2), 0);
        
        // move test
        assert_eq!(board.get_miniboard_win_count_of(2), 0);

        board.do_move(4, 3, 2);
        board.do_move(4, 4, 2);
        board.do_move(4, 5, 2);
        board.calculate_game_status();

        board.do_move(3, 0, 2);
        board.do_move(4, 0, 2);
        board.do_move(5, 0, 2);

        board.calculate_game_status();
        assert_eq!(board.get_miniboard_win_count_of(2), 2);

    }

    #[test]
    fn valid_move_iter() {
        let mut board = Board::default();

        let bitfield = board.valid_moves_bitfield();
        let valid = iterator::ValidMoveIterator::new(bitfield);
        println!("{bitfield}");
        for (row, col) in valid {
            println!("{row} {col} is valid");
        }

        board.do_move(4, 4, 1);
        let bitfield = board.valid_moves_bitfield();
        let valid = iterator::ValidMoveIterator::new(bitfield);
        println!("{bitfield}");
        for (row, col) in valid {
            println!("{row} {col} is valid");
        }
    }

    #[test]
    fn mb_status_iter() {
        use iterator::MiniboardStatusesIterator;
        let mut board = Board::default();

        board.set_status_of(0, 2);
        board.set_status_of(4, 2);
        board.set_status_of(8, 1);

        let mbss = board.get_miniboard_statuses();

        let mbss_iter = MiniboardStatusesIterator::new(mbss);
        for (i, mbs) in mbss_iter.enumerate() {
            match i {
                0 | 4 => assert_eq!(mbs, 2),
                8 => assert_eq!(mbs, 1),
                _ => assert_eq!(mbs, 0)
            }
        }
    }
}
