mod ai;
mod board;

use ai::AI;
use board::{flag::STATUS_CONTESTABLE, Board};
use std::io;

use crate::board::flag;
//use board::{flag::{MINIBOARD_MOVE_COUNT, MINIBOARD_STATUS}, Board};

use std::env;

fn main() -> Result<(), std::io::Error> {
    let mut board = Board::new();
    let ai_o = AI::default();
    let ai_x = AI::new(flag::X_PLAYER);

    //println!("{board:#}");
    //board.do_move(2, 4, 1);
    //let aimove = ai_o.calculate_move_par(&board, 6);
    //println!("{aimove:?}");
    //println!("{board:#}");

    //loop {
    //    let (row, column) = ask_move();
    //    if !board.is_valid_move(row, column) {
    //        println!("{board:#}");
    //        println!("invalid!");
    //        continue;
    //    }
    //
    //    board.do_move(row, column, 1);
    //    let game_status = board.calculate_game_status();
    //    if game_status != STATUS_CONTESTABLE {
    //        println!("Game over: result {game_status}");
    //        break;
    //    }
    //    // println!("{board:#}");
    //    // ai move
    //    let (row, column) = ai.calculate_move_par(&board, 6);
    //    board.do_move(row, column, 2);
    //
    //    print!("\x1B[2J\x1B[1;1H");
    //    println!("{board:#}");
    //    println!(
    //        "{row} {column} move in -> {}",
    //        Board::move_corresponding_miniboard(row, column)
    //    );
    //
    //    let game_status = board.calculate_game_status();
    //    if game_status != STATUS_CONTESTABLE {
    //        println!("Game over: result {game_status}");
    //        break;
    //    }
    //}
    let mut args = env::args();
    let _ = args.next();
    let depth_1: u8 = args
        .next()
        .expect("should've gotten arg for AI")
        .parse()
        .expect("couldn't parse");

    let depth_2: u8 = args
        .next()
        .expect("should've gotten arg for AI")
        .parse()
        .expect("couldn't parse");

    loop {
        let (row, column) = ai_x.calculate_move_par(&board, depth_1);
        board.do_move(row, column, flag::X_PLAYER);
        
        if board.calculate_game_status() != STATUS_CONTESTABLE {
            println!("{board:#}");
            println!("x win");
            std::process::exit(1);
        }

        let (row, column) = ai_o.calculate_move_par(&board, depth_2);
        board.do_move(row, column, flag::O_PLAYER);

        if board.calculate_game_status() != STATUS_CONTESTABLE {
            println!("{board:#}");
            println!("o win");
            std::process::exit(2);
        }
    }
}

fn ask_depth() -> u8 {
    let mut depth = String::new();
    io::stdin()
        .read_line(&mut depth)
        .expect("couldn't read input depth");

    depth
        .trim_end()
        .parse()
        .expect("should've parsed {depth} as a u8")
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
