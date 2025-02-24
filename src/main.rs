mod board;

use board::{flag::{MINIBOARD_MOVE_COUNT, MINIBOARD_STATUS}, Board};

fn main() -> std::fmt::Result {
    let mut board = Board::new();
    //board.main_board[0] = 4_294_119_914;


    board.is_valid_move(5, 5);
    board.do_move(5, 5, 1);
    board.is_valid_move(5, 5);

    board.is_valid_move(5, 4);
    board.do_move(5, 4, 1);

    board.is_valid_move(5, 8);
    board.do_move(5, 8, 2);

    board.do_move(0, 0, 1);
    board.do_move(0, 1, 1);
    board.do_move(0, 2, 2);

    board.do_move(4, 4, 1);
    board.do_move(6, 7, 2);
    board.do_move(5, 3, 2);

    //println!("{}", board);
    println!("{:#}", board);
    board.reset();
    board.do_move(3, 3, 2);
    board.do_move(3, 4, 2);
    board.do_move(3, 5, 2);

    board.do_move(3, 3, 2);
    board.do_move(4, 4, 2);
    board.do_move(5, 5, 2);

    board._check_miniboard_status(4);

    //for i in 0..9 {
    //
    //    println!("{i} -> {}", board.get_meta_data(4, board::flag::MINIBOARD_MOVE_COUNT, board::flag:: MOVE_COUNT_BIT_SIZE));
    //    board.do_move(5, 5, 1);
    //    println!("{i} -> {}", board.get_meta_data(4, board::flag::MINIBOARD_MOVE_COUNT, board::flag::MOVE_COUNT_BIT_SIZE));
    //    println!();
    //    //let get = board.get_meta_data(4, MINIBOARD_MOVE_COUNT);
    //    //board.set_meta_data(4, board::flag::MINIBOARD_MOVE_COUNT, get + 1);
    //    //println!("{:b}", board._get_row_metadata(4));
    //}
    //println!("metadata:{:14b}", board.get_row_metadata(0));

    board.reset();
    Ok(())
}
