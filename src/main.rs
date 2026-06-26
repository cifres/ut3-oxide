#![warn(clippy::pedantic)]

mod ai;
mod board;

use ai::AI;
use board::{Board, bitflag, move_tracker::TrackedBoard};
use std::io::{self, Write};

const SYMBOL: [&str; 2] = ["\x1B[0;34mX\x1B[0m", "\x1B[0;31mO\x1B[0m"];
const HELP_TEXT: &str =
"
Usage: ut3_oxide <COMMAND>

Commands:
  help                                               Display this help message
  tutorial                                           Tutorial mode for beginners to learn the rules
  pvp                                                Player versus Player mode
  pva  [depth=5]                                     Player versus AI mode at a given AI optional [depth]
  ava  <depth_x> <depth_o> <n> [show=0] [delay=500]  AI X versus AI O with a depth for each, repeated <n> times
                                                     [show] `0` to hide or `1` display the game -- optional
                                                     [delay=500] in milliseconds between turns -- optional

Examples:
  ut3_oxide pvp                                      Player versus Player
  ut3_oxide pva 9                                    Player versus AI at depth 9
  ut3_oxide ava 5 5 10                               AI X versus AI O at depth 5 playing 10 games
  ut3_oxide ava 7 7 1 1                              AI X versus AI O at depth 7 playing 1 game, print their moves
  ut3_oxide ava 7 7 2 1 900                          AI X versus AI O at depth 7 playing 1 game, print their moves every 900ms
";

fn main() {
    let mut args = std::env::args();
    _ = args.next(); // program name

    let Some(cmd_name) = args.next() else {
        println!("{HELP_TEXT}");
        return;
    };

    match cmd_name.as_str() {
        "pvp" => player_vs_player(),
        "pva" => player_vs_ai(&mut args),
        "ava" => ai_vs_ai(&mut args),
        "tutorial" => tutorial(),
        "help" => println!("{HELP_TEXT}"),
        unknown => println!("Command `{unknown}` is not recognised.\n{HELP_TEXT}"),
    }
}

fn tutorial() {
    let questions: [(&str, (u8, u8), &str); 6] = [

        (
            "Welcome to the UT3 Tutorial!",
            (0, 0),
            ""
        ),
        (
            "1. Like regular tic-tac-toe, you win by getting a three-in-a-line\nTo form a line \
            here enter the coordinates for the \x1b[31m3rd row \x1b[34m5th column\x1b[0m like \
            \"\x1b[31mrow\x1b[0m\x1b[34mcol\x1b[0m\"",
            (3, 5),
            "Try: \x1b[34m3\x1b[0m\x1b[31m5\x1b[0m",
        ),
        (
            concat!("Excellent! You won a \x1b[34mminiboard\x1b[0m\n\n\
            2. Time to understand the core mechanic!\nIn UT3, your ", uline!("move"), " sends your \
            opponent to a corresponding ", uline!("miniboard"), " and vice versa.\nSo, the \
            \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m ", uline!("move"), " in any ", uline!("miniboard"), " \
            sends your opponent to the \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m \
            ", uline!("miniboard"), ".\n\n\
            Notice how your previous ", uline!("move"), " sent your opponent \
            to the \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m ", uline!("miniboard"), ".\nThe green \
            \x1b[1;32m_\x1b[0m's, and \x1b[32mcoordinates\x1b[0m on the sides highlight valid moves"),
            (0, 0),
            "",
        ),

        (
            "3. Quiz time! Which coordinates will send your oppoent to the \
            \x1b[34mcentre-\x1b[31mleft\x1b[0m \x1b[4mminiboard\x1b[0m?",
            (1, 6),
            "Hint: in the \x1b[34mtop\x1b[0m-\x1b[31mright\x1b[0m miniboard, look for the \
            coordinates on the side that line up with the \x1b[34mcentre-\x1b[31mleft\x1b[0m \
                cell",
        ),
        (
            concat!("4. Time for the final rule! \x1b[31mO\x1b[0m will now play in the middle-centre \
            cell. But, the \x1b[34mmiddle\x1b[0m-\x1b[31mcentre\x1b[0m ", uline!("miniboard"), " is \
            won already.\nWhat do you think will happen?"),
            (0, 0),
            "",
        ),
        (
            "Notice how you can make a move anywhere now! Remember, green \x1b[1;32m_\x1b[0m's \
            highlight valid moves",
            (0, 0),
            ""
        ),

    ];

    let mut tracked = TrackedBoard::new(Board::new());
    let mut buffer = String::with_capacity(2);

    // Pre tutorial setup
    tracked.board.do_move(3, 3, bitflag::X_PLAYER);
    tracked.board.do_move(1, 1, bitflag::O_PLAYER);
    tracked.board.do_move(3, 4, bitflag::X_PLAYER);
    tracked.board.do_move(1, 4, bitflag::O_PLAYER);

    for (i, &(question, answer, hint)) in questions.iter().enumerate() {
        print!("\x1B[2J\x1B[1;1H");
        println!("{:#}", tracked.board);
        buffer.clear();
        println!("{question}\n");

        // FIXME: error prone if question order changes
        if i == 4 {
            tracked.board.do_move(4, 1, bitflag::O_PLAYER);
        }

        loop {
            // If there's no hint, this isn't a question to be answered
            if hint.is_empty() {
                print!("Press Enter to continue ");
                _ = io::stdout().flush();
                _ = io::stdin().read_line(&mut buffer);
                break;
            }

            if let Some((row, col)) = ask_move(&mut tracked, &mut buffer, &mut false, true)
                && (row, col) == answer
            {
                tracked.board.do_move(row, col, bitflag::X_PLAYER);
                break;
            }

            print!("\x1B[2J\x1B[1;1H");
            println!("{:#}", tracked.board);
            println!("{question}\n");
            println!("{hint}");
        }
    };

    print!("\x1B[2J\x1B[1;1H");
    println!("{:#}", tracked.board);
    println!(
        "Congratulations on completing the tutorial!\nYou've learned how to win a miniboard and \
        what happens when you are sent to a won/full miniboard"
    );
}

fn player_vs_player() {
    let mut turn: u8 = 0;
    let mut is_game_over = false;
    let mut tracked = TrackedBoard::new(Board::new());
    let mut input_buffer = String::with_capacity(2);

    print!("\x1B[2J\x1B[1;1H");
    println!("Welcome! You may (u)ndo, (r)eset or (q)uit any time.");
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
    let len = args.len();
    let [depth_ai_x, depth_ai_o, iterations, ..]: [u8] = args
        .take(3)
        .filter_map(|arg| arg.parse().ok())
        .collect::<Vec<u8>>()[..]
    else {
        eprintln!("Expected 3 arguments found {len}.");
        eprintln!("Expected format: ./ut3_oxide <depth_x> <depth_o> <n> [show] [delay]");
        eprintln!("Try `./ut3_oxide 5 5 10` ");
        return;
    };

    let print_game = match args.next() {
        Some(to_show) => to_show.parse().unwrap_or(0) != 0,
        None => false,
    };

    let print_delay = match args.next() {
        Some(n) => n.parse().unwrap_or(500),
        None => 500,
    };

    let mut wins = 0u16;
    let mut draws = 0u16;
    let mut losses = 0u16;
    let mut total_turns = 0u32;
    let mut status;

    let ai_xo = [AI::new(bitflag::X_PLAYER), AI::new(bitflag::O_PLAYER)];
    let ai_xo_depth = [depth_ai_x, depth_ai_o];
    for _ in 0..iterations {
        loop {
            // Due to turns % 2, whoever does the finishing move starts second. Thus they swap
            // who plays first.
            let i = total_turns as usize % 2;
            let ai = ai_xo[i];
            let depth = ai_xo_depth[i];
            let (_, (row, column)) = ai.calculate_move_par(&board, depth);
            board.do_move(row, column, ai.ai_shape);
            total_turns += 1;

            if print_game {
                println!("\x1B[2J\x1B[1;1H");
                println!("{board:#}");
                _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(print_delay));
            }

            let game_status = board.calculate_game_status();
            if game_status != bitflag::STATUS_CONTESTABLE {
                status = game_status;
                break;
            }
        }

        match status {
            bitflag::X_PLAYER => wins += 1,
            bitflag::O_PLAYER => losses += 1,
            bitflag::STATUS_DRAW => draws += 1,
            _ => unreachable!(),
        }

        board.reset();
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

    let ai_o = AI::new(bitflag::O_PLAYER);
    let mut tracked = TrackedBoard::new(Board::new());
    let mut is_game_over = false;
    let mut input_buffer = String::with_capacity(2);

    let mut is_player_turn = true;
    let mut ai_messages = String::with_capacity(760);
    let mut ai_eval_buffer = String::with_capacity(40);
    let ai_x_hinter = AI::new(bitflag::X_PLAYER);

    print!("\x1B[2J\x1B[1;1H");
    println!("Welcome! You may get a (h)int, (u)ndo, (r)eset or (q)uit any time.");
    println!(
        "Enter a move as \"\x1b[31mrow\x1b[0m\x1b[34mcol\x1b[0m\" like \"\x1b[31m4\x1b[0m\x1b[34m3\x1b[0m\""
    );
    println!("{:#}", tracked.board);

    loop {
        if is_player_turn {
            if let Some((row, column)) =
                ask_move(&mut tracked, &mut input_buffer, &mut is_game_over, true)
                && tracked.board.is_valid_move(row, column)
            {
                tracked.do_move(row, column, bitflag::X_PLAYER);
            } else {
                print!("\x1B[2J\x1B[1;1H");
                println!("{:#}", tracked.board);

                // If the input isn't a move and doesn't match any str commands, it's invalid.
                if !match input_buffer.trim() {
                    "u" | "r" => true,
                    "h" if !is_game_over => {
                        let (_, mov) = ai_x_hinter.calculate_move_par(&tracked.board, depth + 2);
                        println!("How about {mov:?}?");
                        true
                    }
                    _ => false,
                } {
                    println!("\x1b[41m Invalid! \x1b[0m");
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
        if game_status != bitflag::STATUS_CONTESTABLE {
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
        print!("Enter a move: ");
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
