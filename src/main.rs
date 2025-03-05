mod board;
mod ai;

use ai::AI;
use board::Board;
//use board::{flag::{MINIBOARD_MOVE_COUNT, MINIBOARD_STATUS}, Board};

fn main() {
    let mut board = Board::new();
    let ai = AI::default();
    
    board.do_move(2, 4, 1);
    let aimove = ai.calculate_move_par(&board, 7);
    println!("{aimove:?}");
}


fn ask_move() {
    
}
