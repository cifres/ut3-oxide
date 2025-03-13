mod ai;
mod board;

use ai::AI;
use board::{flag, Board};
use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    let mut board = Board::new();
    let ai_x = AI::new(flag::X_PLAYER);
    //let ai_o = AI::default();

    //println!("{board:#}");
    // TODO: merge vs ai and AI X vs AI O based on prescence of command line args?
    board.do_move(2, 4, flag::O_PLAYER);
    let (row, column) = ai_x.calculate_move_par(&board, 7);
    board.do_move(row, column, flag::X_PLAYER);
    //println!("{row}, {column}");
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
    //        return Ok(());
    //    }
    //    // println!("{board:#}");
    //    // ai move
    //    let (row, column) = ai_x.calculate_move_par(&board, 7);
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
    //        return Ok(());
    //    }
    //}

    //let mut wins = 0u16;
    //let mut draws = 0u16;
    //let mut losses = 0u16;
    //let mut total_turns = 0u32;
    //let mut status;
    //
    //let mut args = std::env::args();
    //let _ = args.next();
    //let depth_1: u8 = args
    //    .next()
    //    .expect("should've gotten arg for AI X")
    //    .parse()
    //    .expect("couldn't parse");
    //
    //let depth_2: u8 = args
    //    .next()
    //    .expect("should've gotten arg for AI O")
    //    .parse()
    //    .expect("couldn't parse");
    //
    //let iterations: u8 = args
    //    .next()
    //    .expect("should've gotten number of iterations")
    //    .parse()
    //    .expect("couldn't parse iteration count");
    //
    //for _ in 0..iterations {
    //    loop {
    //        let (row, column) = ai_x.calculate_move_par(&board, depth_1);
    //        board.do_move(row, column, flag::X_PLAYER);
    //        total_turns += 1;
    //        let game_status = board.calculate_game_status();
    //        if game_status != flag::STATUS_CONTESTABLE {
    //            status = game_status;
    //            break;
    //        }
    //
    //        // ai move
    //        let (row, column) = ai_o.calculate_move_par(&board, depth_2);
    //        board.do_move(row, column, 2);
    //        total_turns += 1;
    //
    //        //println!("\x1B[2J\x1B[1;1H");
    //        //println!("{board:#}");
    //        //println!(
    //        //    "{row} {column} move in -> {}",
    //        //    Board::move_corresponding_miniboard(row, column)
    //        //);
    //
    //        let game_status = board.calculate_game_status();
    //        if game_status != flag::STATUS_CONTESTABLE {
    //            status = game_status;
    //            break;
    //        }
    //    }
    //
    //    if status == flag::X_PLAYER {
    //        wins += 1;
    //    } else if status == flag::STATUS_DRAW {
    //        draws += 1;
    //    } else if status == flag::O_PLAYER {
    //        losses += 1;
    //    }
    //
    //    board.reset();
    //    //println!("{board:#}");
    //    //println!("Game over: result [{status}]");
    //    //println!("X = {} — O = {}", flag::X_PLAYER, flag::O_PLAYER);
    //}
    //
    //let total = wins + losses + draws;
    //let average_turns = total_turns as f32 / total as f32;
    //let wrx = (wins as f64 / total as f64) * 100.0;
    //let wro = (losses as f64 / total as f64) * 100.0;
    //
    //println!(
    //    "wr @ depth {depth_1} X: {wrx:.2}%\nwr @ depth {depth_2} O: {wro:.2}%
    //    — total games: {total}
    //    — average total turns:  {average_turns:.2} of total {total_turns} 
    //    — draws: {draws}",
    //);

    Ok(())
}

//fn ask_depth() -> u8 {
//    let mut depth = String::new();
//    io::stdin()
//        .read_line(&mut depth)
//        .expect("couldn't read input depth");
//
//    depth
//        .trim_end()
//        .parse()
//        .expect("should've parsed {depth} as a u8")
//}

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
