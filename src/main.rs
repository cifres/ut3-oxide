mod ai;
mod board;

use ai::AI;
use board::{Board, bitflag, move_tracker::TrackedBoard};
use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    let mut args = std::env::args();
    _ = args.next(); // program name
    if args.len() == 0 {
        player_vs_ai();
    } else {
        ai_vs_ai(&mut args);
    }

    Ok(())
}

fn ai_vs_ai(args: &mut std::env::Args) {
    let mut board = Board::new();
    let depth_1: u8 = args
        .next()
        .expect("should've gotten arg for AI X")
        .parse()
        .expect("AI X's depth should be a valid number between 0–255");

    let depth_2: u8 = args
        .next()
        .expect("should've gotten arg for AI O")
        .parse()
        .expect("AI O's depth should be a valid number between 0–255");

    let iterations: u16 = args
        .next()
        .expect("should've gotten number of iterations")
        .parse()
        .expect("iterations should be a number between 0–65535");

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
            let (_, (row, column)) = ai_x.calculate_move_par(&board, depth_1);
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
            let (_, (row, column)) = ai_o.calculate_move_par(&board, depth_2);
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
        "wr @ depth {depth_1} X: {wrx:.2}% ({wins}/{total})\nwr @ depth {depth_2} O: {wro:.2}% ({losses}/{total})
        — total games: {total}
        — average total turns: {average_turns:.2} of total {total_turns}
        — draws: {drawrate:.2}% ({draws}/{total}),"
    );
}

fn player_vs_ai() {
    let ai_x = AI::new(bitflag::X_PLAYER);
    let mut tracked = TrackedBoard::new(Board::new());

    println!("{:#}", tracked.board);

    loop {
        let Some((row, column)) = ask_move(&mut tracked) else {
            print!("\x1B[2J\x1B[1;1H");
            println!("{:#}", tracked.board);
            println!("invalid!");
            let (lr, lc) = tracked.board.last_move;
            println!(
                "{lr} {lc} move in -> {}",
                Board::move_corresponding_miniboard(lr, lc)
            );
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

        // println!("{board:#}");
        // ai move
        let now = std::time::Instant::now();
        let (eval, (row, column)) = ai_x.calculate_move_par(&tracked.board, 5);
        let elapsed = now.elapsed();
        tracked.do_move(row, column, 2);

        print!("\x1B[2J\x1B[1;1H");
        println!("{:#}", tracked.board);
        println!(
            "{row} {column} move in -> {} ({} ms | {} μs)",
            Board::move_corresponding_miniboard(row, column),
            elapsed.as_millis(),
            elapsed.as_micros(),
        );

        // let eval = ai_x.evaluate(board);
        let norm = normalise(eval as f32, -2300., 2300.);
        println!("score: {eval} -> {:.2}", norm);
        eval_bar(norm, 30);

        let game_status = tracked.board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            _ = io::stdout().flush();
            println!("Game over: result {game_status}");
            tracked.board.display_mb_statuses();
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
    //println!("\n{input}");

    //let [row, col, ..] = input
    //    .split_whitespace()
    //    .map(|sn| sn.parse().expect("Should have parsed the number {sn}"))
    //    .collect::<Vec<u8>>()[..] else { return None };

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

fn eval_bar(norm: f32, width: u8) {
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
    eval_bar(0.3, 30);
    eval_bar(0.4, 30);
    eval_bar(0.05, 30);
}
