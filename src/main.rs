mod ai;
mod board;

use ai::AI;
use board::{flag::STATUS_CONTESTABLE, Board};
use std::io;
//use board::{flag::{MINIBOARD_MOVE_COUNT, MINIBOARD_STATUS}, Board};

fn main() {
    let mut board = Board::new();
    let ai = AI::default();

    println!("{board:#}");
    //board.do_move(2, 4, 1);
    //let aimove = ai.calculate_move_par(&board, 7);
    //println!("{aimove:?}");

    loop {
        let (row, column) = ask_move();
        if !board.is_valid_move(row, column) {
            println!("{board:#}");
            println!("invalid!");
            continue;
        }

        board.do_move(row, column, 1);
        let game_status = board.calculate_game_status();
        if game_status != STATUS_CONTESTABLE {
            println!("Game over: result {game_status}");
            break;
        }
        // println!("{board:#}");
        // ai move
        let (row, column) = ai.calculate_move_par(&board, 6);
        board.do_move(row, column, 2);

        print!("\x1B[2J\x1B[1;1H");
        println!("{board:#}");
        println!("{row} {column} move in -> {}", Board::move_corresponding_miniboard(row, column));

        let game_status = board.calculate_game_status();
        if game_status != STATUS_CONTESTABLE {
            println!("Game over: result {game_status}");
            break;
        }
    }
}

fn ask_move() -> (u8, u8) {
    print!("Enter a move as <row col> like 0 1: ");
    let mut input = String::new();
    println!();
    io::stdin()
        .read_line(&mut input)
        .expect("Couldn't get the input");

    println!("\n{input}");
    
    let moves = input
        .split_whitespace()
        .map(|sn| {
            println!("{sn}");
            sn.parse().expect("Should have parsed the number {sn}")
        })
        .collect::<Vec<u8>>();

    (moves[0], moves[1])
}
