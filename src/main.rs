#![warn(clippy::pedantic)]

mod ai;
mod board;

use ai::AI;
use board::{Board, bitflag, move_tracker::TrackedBoard};
use std::io::{self, Write};

fn main() {
    let mut args = std::env::args();
    _ = args.next(); // program name

    if args.len() == 1 {
        player_vs_ai(&mut args);
    } else {
        ai_vs_ai(&mut args);
    }
}

fn ai_vs_ai(args: &mut std::env::Args) {
    let mut board = Board::new();

    let args_len = args.len();

    let [depth_ai_x, depth_ai_o, iterations, ..]: [u8] = args
        .take(3)
        .filter_map(|arg| arg.parse().ok())
        .collect::<Vec<u8>>()[..]
    else {
        eprintln!("Expected 3 arguments found {args_len}.");
        eprintln!("Expected format: ./ut3_oxide <depth for ai_x> <depth for ai_o> <iterations>");
        eprintln!("Try `./ut3_oxide 5 5 10` ");
        return;
    };

    let ai_x = AI::new(bitflag::X_PLAYER);
    let ai_o = AI::new(bitflag::O_PLAYER);

    let mut wins = 0u16;
    let mut draws = 0u16;
    let mut losses = 0u16;
    let mut total_turns = 0u32;
    let mut status;

    for _ in 0..iterations {
        loop {
            //let now = std::time::Instant::now();
            let (_, (row, column)) = ai_x.calculate_move_par(&board, depth_ai_x);
            //let elapsed = now.elapsed();
            //println!("x: {:.2}", elapsed.as_micros());
            board.do_move(row, column, bitflag::X_PLAYER);
            total_turns += 1;
            let game_status = board.calculate_game_status();
            if game_status != bitflag::STATUS_CONTESTABLE {
                status = game_status;
                break;
            }

            // ai move
            //let now = std::time::Instant::now();
            let (_, (row, column)) = ai_o.calculate_move_par(&board, depth_ai_o);
            //let elapsed = now.elapsed();
            //println!("o: {:.2}", elapsed.as_micros());
            board.do_move(row, column, 2);
            total_turns += 1;

            //println!("\x1B[2J\x1B[1;1H");
            //println!("{board:#}");
            //println!(
            //    "{row} {column} move in -> {}",
            //    Board::move_corresponding_miniboard(row, column)
            //);

            let game_status = board.calculate_game_status();
            if game_status != bitflag::STATUS_CONTESTABLE {
                status = game_status;
                break;
            }
        }

        if status == bitflag::X_PLAYER {
            wins += 1;
        } else if status == bitflag::STATUS_DRAW {
            draws += 1;
        } else if status == bitflag::O_PLAYER {
            losses += 1;
        }

        board.reset();
        //println!("{board:#}");
        //println!("Game over: result [{status}]");
        //println!("X = {} — O = {}", bitflag::X_PLAYER, bitflag::O_PLAYER);
    }

    let total = wins + losses + draws;
    let average_turns = total_turns as f32 / total as f32;
    let wrx = (wins as f64 / total as f64) * 100.0;
    let wro = (losses as f64 / total as f64) * 100.0;
    let drawrate = (draws as f64 / total as f64) * 100.0;

    println!(
        "wr @ depth {depth_ai_x} X: {wrx:.2}% ({wins}/{total})\nwr @ depth {depth_ai_o} O: {wro:.2}% ({losses}/{total})
        — total games: {total}
        — average total turns: {average_turns:.2} of total {total_turns}
        — draws: {drawrate:.2}% ({draws}/{total}),"
    );
}

fn player_vs_ai(args: &mut std::env::Args) {

    let depth = match args.next() {
        Some(d) => d.parse().unwrap_or(5),
        None => 5,
    };

    println!("\nAI search depth: {depth}...");

    let ai_x = AI::new(bitflag::X_PLAYER);
    let mut tracked = TrackedBoard::new(Board::new());

    println!("{:#}", tracked.board);

    loop {
        let Some((row, column)) = ask_move(&mut tracked) else {
            print!("\x1B[2J\x1B[1;1H");
            println!("{:#}", tracked.board);
            println!("invalid!");
            // let (lr, lc) = tracked.board.last_move;
            // println!(
            //     "{lr} {lc} move in -> {}",
            //     Board::move_corresponding_miniboard(lr, lc)
            // );
            continue;
        };

        if !tracked.board.is_valid_move(row, column) {
            print!("\x1B[2J\x1B[1;1H");
            println!("{:#}", tracked.board);
            println!("invalid!");
            continue;
        }

        tracked.do_move(row, column, 1);
        let game_status = tracked.board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            _ = io::stdout().flush();
            println!("Game over: result {game_status}");
            tracked.board.display_mb_statuses();
            break;
        }

        // ai move
        let now = std::time::Instant::now();
        let (eval, (row, column)) = ai_x.calculate_move_par(&tracked.board, depth);
        let elapsed = now.elapsed();
        tracked.do_move(row, column, 2);

        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        println!("{:#}", tracked.board);

        let norm = normalise(eval as f32, -2300., 2300.);
        println!("Score: {eval} -> {norm:.2}\n");
        print_eval_bar(norm, 30);
        println!();

        println!(
            "AI: ({row}, {column}) — ({} ms | {} μs)",
            elapsed.as_millis(),
            elapsed.as_micros(),
        );

        let game_status = tracked.board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            _ = io::stdout().flush();
            println!("Game over: result {game_status}");
            break;
        }
    }
}

fn ask_move(tracked: &mut TrackedBoard) -> Option<(u8, u8)> {
    print!("Enter a move as \"rowcol\" like \"43\": ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Couldn't get the input");

    let input = input.trim_end();
    if input == "u" {
        println!("Move undone!");
        // undo twice for the AI and the player
        tracked.undo_move();
        tracked.undo_move();
        return None;
    } else if input == "r" {
        println!("Game reset!");
        tracked.reset();
        return None;
    } else if input == "q" {
        println!("See ya! :)");
        std::process::exit(0);
    }

    if input.len() != 2 {
        return None;
    }

    let split_at = input.split_at(1);
    let (row, col): (u8, u8) = (
        split_at.0.parse().unwrap_or(9),
        split_at.1.parse().unwrap_or(9),
    );

    if row == 9 || col == 9 {
        return None;
    }

    Some((row, col))
}

fn normalise(n: f32, min: f32, max: f32) -> f32 {
    let n = n.clamp(min, max);
    (n - min) / (max - min)
}

#[test]
fn norm() {
    println!("{:.2}", normalise(100., 1., 200.));
    println!("{:.2}", normalise(100., -50., 2000.));
}

fn print_eval_bar(norm: f32, width: u8) {
    const BASE_BAR: &str = "\x1b[34m█\x1b[0m";
    let base = BASE_BAR.repeat(width as usize);
    let s = base.replacen(
        BASE_BAR,
        "\x1b[31m█\x1b[0m",
        (norm * width as f32).round() as usize,
    );
    println!("{s}");
}

#[test]
fn evalbar() {
    print_eval_bar(0.3, 30);
    print_eval_bar(0.4, 30);
    print_eval_bar(0.05, 30);
}
