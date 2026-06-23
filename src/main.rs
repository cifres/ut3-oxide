#![warn(clippy::pedantic)]

// TODO: (h)int, tutorial?

mod ai;
mod board;

use ai::AI;
use board::{Board, bitflag, move_tracker::TrackedBoard};
use std::io::{self, Write};

const SYMBOL: [&str; 2] = ["\x1B[0;34mX\x1B[0m", "\x1B[0;31mO\x1B[0m"];

fn main() {
    let mut args = std::env::args();
    _ = args.next(); // program name

    match args.len() {
        0 => player_vs_player(),
        1 => player_vs_ai(&mut args),
        _ => ai_vs_ai(&mut args), // It deals with any number of arguments
    }
}

fn player_vs_player() {
    let mut turn: u8 = 0;
    let mut is_game_over = false;
    let mut tracked = TrackedBoard::new(Board::new());
    let mut input_buffer = String::with_capacity(2);

    print!("\x1B[2J\x1B[1;1H");
    println!("{:#}", tracked.board);

    loop {
        println!("{}'s turn", SYMBOL[usize::from(turn % 2)]);

        if let Some((row, column)) =
            ask_move(&mut tracked, &mut input_buffer, &mut is_game_over, false)
            && tracked.board.is_valid_move(row, column)
        {
            tracked.do_move(row, column, turn % 2 + 1);
            turn += 1;
        } else {
            print!("\x1B[2J\x1B[1;1H");
            println!("{:#}", tracked.board);
            match input_buffer.trim() {
                "u" => turn = turn.saturating_sub(1),
                "r" => turn = 0,
                _ => println!("\x1b[41m invalid! \x1b[0m"),
            }

            continue;
        }

        print!("\x1B[2J\x1B[1;1H");
        println!("{:#}", tracked.board);

        let status = tracked.board.calculate_game_status();
        if status != bitflag::STATUS_CONTESTABLE {
            is_game_over = true;
            match status {
                bitflag::STATUS_O_WIN | bitflag::STATUS_X_WIN => {
                    let mv_cnt = (0..9)
                        .map(|n| tracked.board.get_player_move_count_of(n, status))
                        .sum::<u8>();

                    println!(
                        "{} \x1b[1mWins in {mv_cnt} moves\x1b[0m!",
                        SYMBOL[usize::from(status - 1)]
                    );
                }
                bitflag::STATUS_DRAW => println!("Draw!"),
                _ => unreachable!(),
            }
        }
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

            let game_status = board.calculate_game_status();

            // println!("\x1B[2J\x1B[1;1H");
            // println!("{board:#}");
            // _ = std::io::stdout().flush();
            // std::thread::sleep(std::time::Duration::from_millis(750));

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
        _ => unreachable!(),
    };

    let ai_o = AI::new(bitflag::O_PLAYER);
    let mut tracked = TrackedBoard::new(Board::new());
    let mut is_game_over = false;
    let mut input_buffer = String::with_capacity(2);

    let mut is_player_turn = true;
    let mut ai_messages = String::with_capacity(760);
    let mut ai_eval_buffer = String::with_capacity(40);

    print!("\x1B[2J\x1B[1;1H");
    println!("Welcome! You may (u)ndo, (r)eset or (q)uit any time.");
    println!("{:#}", tracked.board);

    loop {
        if is_player_turn {
            if let Some((row, column)) =
                ask_move(&mut tracked, &mut input_buffer, &mut is_game_over, true)
                && tracked.board.is_valid_move(row, column)
            {
                // TOOD (h)int
                tracked.do_move(row, column, bitflag::X_PLAYER);
            } else {
                print!("\x1B[2J\x1B[1;1H");
                println!("{:#}", tracked.board);

                if !matches!(input_buffer.trim(), "u" | "r") {
                    println!("\x1b[41m invalid! \x1b[0m");
                }

                continue;
            }
        } else {
            // ai move
            let now = std::time::Instant::now();
            let (eval, (row, column)) = ai_o.calculate_move_par(&tracked.board, depth);
            let elapsed = now.elapsed();
            tracked.do_move(row, column, bitflag::O_PLAYER);

            let norm = normalise(eval as f32, -5700., 5700.);

            get_eval_bar(norm, 30, &mut ai_eval_buffer);
            ai_messages.push_str(format!("Score: {eval} -> {norm:.2}\n{ai_eval_buffer}").as_str());

            // Adapatively select display units
            let duration = match elapsed.as_micros() {
                micros @ 0..=1000 => format!("{micros} μs"),
                micros_to_ms @ 1001.. => format!("{}~ ms", micros_to_ms / 1000),
            };

            ai_messages.push_str(
                format!("\n\nAI @ depth {depth} in {duration}: ({row}, {column})\n").as_str(),
            );

            println!("end {}", ai_messages.capacity());
        }

        // Clear and redraw
        print!("\x1B[2J\x1B[1;1H");
        println!("{:#}", tracked.board);

        println!("{ai_messages}");
        ai_messages.clear();

        let game_status = tracked.board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE || is_game_over {
            is_game_over = true;
            _ = io::stdout().flush();
            match game_status {
                bitflag::STATUS_O_WIN | bitflag::STATUS_X_WIN => {
                    let mv_cnt = (0..9)
                        .map(|n| tracked.board.get_player_move_count_of(n, game_status))
                        .sum::<u8>();

                    println!(
                        "{} \x1b[1mWins in {mv_cnt} moves\x1b[0m!",
                        SYMBOL[usize::from(game_status - 1)]
                    );
                }
                bitflag::STATUS_DRAW => println!("Draw!"),
                _ => unreachable!(),
            }
        }

        is_player_turn = !is_player_turn;
    }
}

fn ask_move(
    tracked: &mut TrackedBoard,
    input_buffer: &mut String,
    is_game_over: &mut bool,
    is_p_v_ai: bool,
) -> Option<(u8, u8)> {
    if *is_game_over {
        print!("Game over! (u)ndo, (r)eset, (q)uit? ");
    } else {
        print!("Enter a move as \"rowcol\" like \"43\": ");
    }
    _ = io::stdout().flush();

    input_buffer.clear();

    io::stdin()
        .read_line(input_buffer)
        .expect("Couldn't get the input");

    // Parse undo, reset, and quit
    let input = input_buffer.trim();
    if input == "u" {
        *is_game_over = false;
        // undo twice for the Player versus AI
        println!("Move undone!");
        tracked.undo_move();
        if is_p_v_ai {
            tracked.undo_move();
        }

        return None;
    } else if input == "r" {
        *is_game_over = false;

        println!("Game reset!");
        tracked.reset();
        return None;
    } else if input == "q" {
        println!("See ya! :)");
        std::process::exit(0);
    }

    // Otherwise parse a move
    if input.len() != 2 || *is_game_over {
        return None;
    }

    // E.g. "47" -> (4, 7)
    let split_at = input.split_at(1);
    match (split_at.0.parse(), split_at.1.parse()) {
        (Ok(row @ 0..=8), Ok(col @ 0..=8)) => Some((row, col)),
        _ => None,
    }
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

fn get_eval_bar(norm: f32, width: u8, buffer: &mut String) {
    const BASE_BAR: &str = "\x1b[34m█\x1b[0m";
    let base = BASE_BAR.repeat(width as usize);
    *buffer = base.replacen(
        BASE_BAR,
        "\x1b[31m█\x1b[0m",
        (norm * width as f32).round() as usize,
    );
    // println!("{s}");
}

#[test]
fn evalbar() {
    // get_eval_bar(0.3, 30);
    // get_eval_bar(0.4, 30);
    // get_eval_bar(0.05, 30);
}
