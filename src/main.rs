mod ai;
mod board;

use ai::AI;
use board::{flag::STATUS_CONTESTABLE, Board};
use std::io::{self, Write};

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
            println!("\x1B[2J\x1B[1;1H");
            println!("{board:#}");
            println!("Invalid!");

            let (row, column) = board.last_move;
            println!("Make a move in miniboard {}", Board::move_corresponding_miniboard(row, column));

            continue;
        }

        board.do_move(row, column, 1);
        let game_status = board.calculate_game_status();
        if game_status != STATUS_CONTESTABLE {
            println!("Game over: result {game_status}");
            break;
        }

        // ai move
        let (row, column) = ai.calculate_move_par(&board, 7);
        board.do_move(row, column, 2);

        println!("\x1B[2J\x1B[1;1H");
        println!("{board:#}");
        println!(
            "{row} {column} move in -> {}",
            Board::move_corresponding_miniboard(row, column)
        );

        let game_status = board.calculate_game_status();
        if game_status != STATUS_CONTESTABLE {
            println!("Game over: result {game_status}");
            break;
        }
    }
}

fn ask_move() -> (u8, u8) {
    print!("Enter a move as \"row col\" like \"4 3\": ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Couldn't get the input");

    //println!("\n{input}");

    let moves = input
        .split_whitespace()
        .map(|sn| sn.parse().expect("Should have parsed the number {sn}"))
        .collect::<Vec<u8>>();

    (moves[0], moves[1])
}
