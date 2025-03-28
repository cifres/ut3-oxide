use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use ut3_oxide::{
    ai::AI,
    board::{flag, Board},
};

const DEPTH: u8 = 5;

fn winrate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("AI-performance");

    // single call speed
    let mut board = Board::new(true);
    //let mut board = Board::default();
    let ai = AI::default();
    let rng = rand::prelude::SmallRng::seed_from_u64(7);

    do_random_moves(&mut board, rng);

    // 298.42 μs @ depth 5
    // 246.42 μs @ depth 5
    // 211.28 μs @ depth 5 inline only no (always)
    // 125.14 μs @ depth 5
    // PC 124.14 μs @ depth 5 26/03/2025
    // Laptop 182 μs @ depth 5 -- current best
    // PC 118~ μs @ depth 5 -- current best
    group.bench_function("single-call-performance", |b| {
        b.iter(|| {
            ai.calculate_move_par(&board, DEPTH);
        });
    });

    let aix = AI::new(flag::X_PLAYER);
    let aio = AI::default();
    board.reset();
    group.bench_function("single-ai-v-ai-playout", |b| {
        b.iter(|| ai_vs_ai(board.clone(), &aix, &aio));
    });

    let rng = rand::prelude::SmallRng::seed_from_u64(7);
    group.bench_function("single-playout", |b| {
        b.iter(|| ai_single(board.clone(), &aio, rng.clone()));
    });

    // winrate
    let mut wins = 0u16;
    let mut draws = 0u16;
    let mut losses = 0u16;
    let mut total_turns = 0u32;
    //let mut total_moves = 0u32;
    //let mut running_avg_mov = Vec::new();
    //let duration = std::time::Duration::from_secs(30);

    // group.sample_size(10).measurement_time(duration);
    //group.measurement_time(duration);
    group.bench_function("winrate", |b| {
        b.iter(|| {
            let (status, turns, moves) = ai_playout();
            //let (status, turns) = ai_vs_ai(board.clone(), &aix, &aio);
            total_turns += turns as u32;
            //total_moves += moves as u32;
            //running_avg_mov.push(moves as f32 / turns as f32);
            if status == flag::O_PLAYER {
                wins += 1;
            } else if status == flag::STATUS_DRAW {
                draws += 1;
            } else if status == flag::X_PLAYER {
                losses += 1;
            }
            status
        });
    });

    group.finish();

    if total_turns == 0 {
        return;
    }

    let total = wins + losses + draws;
    let average_turns = total_turns as f32 / total as f32;
    let wro = (wins as f64 / total as f64) * 100.0;
    let wrx = (losses as f64 / total as f64) * 100.0;
    let drawper = (draws as f64 / total as f64) * 100.0;
    //let average_possible_moves = total_moves as f32 / total_turns as f32;
    //let running_avg = running_avg_mov.iter().sum::<f32>() / running_avg_mov.len() as f32;
    //let max = running_avg_mov.clone().into_iter().reduce(f32::max).unwrap_or(0.);
    //let min = running_avg_mov.into_iter().reduce(f32::min).unwrap_or(0.);
    println!(
        "wr @ depth {DEPTH}: wins (O's wins) {wro:.2}% -> {wins}/{total} 
        — losses (X's wins): {wrx:.2}% {losses}/{total}
        — draws: {drawper:.2}% {draws}/{total}
        — average total turns: {average_turns:.2} of {total_turns}",
        /* — average possible moves: {average_possible_moves} of {total_moves} — {running_avg} {min}..{max} */
    );
}

fn do_random_moves(board: &mut Board, mut rng: SmallRng) {
    for i in 0..17 {
        let (row, column) = board
            .valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        let player = if i % 2 == 0 {
            flag::X_PLAYER
        } else {
            flag::O_PLAYER
        };
        board.do_move(row, column, player);
    }
}

fn ai_single(mut board: Board, ai: &AI, mut rng: SmallRng) -> u8 {
    loop {
        // random move
        //let validbitfield = board.valid_moves_bitfield();
        //let (row, column) = iterator::ValidMoveIterator::new(validbitfield)
        let (row, column) = board
            .valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        // let (row, column) = aix.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, flag::X_PLAYER);

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return game_status;
        }

        // ai move
        let (row, column) = ai.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, flag::O_PLAYER);

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return game_status;
        }
    }
}

fn ai_vs_ai(mut board: Board, aix: &AI, aio: &AI) -> (u8, u8) {
    let mut turns = 0;
    loop {
        let (row, column) = aix.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, flag::X_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return (game_status, turns);
        }

        // ai move
        let (row, column) = aio.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, flag::O_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return (game_status, turns);
        }
    }
}

fn ai_playout() -> (u8, u8, u16) {
    let mut board = Board::new(true);
    //let mut board = Board::default();
    let ai = AI::default();
    let mut turns = 0;
    let mut possible_moves = 0;

    //let mut rng = rand::rng();
    let mut rng = rand::prelude::SmallRng::seed_from_u64(7);

    loop {
        // random move

        //possible_moves += (board.valid_moves().collect::<Vec<(u8, u8)>>().len()) as u16;
        let (row, column) = board
            .valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        // let (row, column) = aix.calculate_move_par(&board, DEPTH);
        //valid_moves = board.valid_moves() 
        board.do_move(row, column, flag::X_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return (game_status, turns, possible_moves);
        }

        // ai move
        //possible_moves += (board.valid_moves().collect::<Vec<(u8, u8)>>().len()) as u16;
        let (row, column) = ai.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, flag::O_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return (game_status, turns, possible_moves);
        }
    }
}

criterion_group!(benches, winrate_benchmark);
criterion_main!(benches);
