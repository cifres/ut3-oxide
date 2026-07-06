use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use ut3_oxide::{
    ai::AI,
    board::{bitflag, Board},
};

const DEPTH: u8 = 5;

fn winrate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("AI-performance");

    // single call speed
    let mut board = Board::new();
    let ai = AI::default();
    let rng = rand::prelude::SmallRng::seed_from_u64(7);
    //let rng = rand::prelude::SmallRng::from_rng(&mut rand::rng());

    do_random_moves(&mut board, rng);

    // 298.42 μs @ depth 5
    // 246.42 μs @ depth 5
    // 211.28 μs @ depth 5 inline only no (always)
    // 125.14 μs @ depth 5
    // PC 124.14 μs @ depth 5 26/03/2025
    // Laptop 167 μs @ depth 5 -- current best
    // PC 118~ μs @ depth 5
    // PC 112~ μs @ depth 5 -- current best
    group.bench_function("single-call-performance", |b| {
        b.iter(|| {
            ai.calculate_move_par(&board, DEPTH);
        });
    });

    let aix = AI::new(bitflag::X_PLAYER);
    let aio = AI::default();
    board.reset();
    group.bench_function("single-ai-v-ai-playout", |b| {
        b.iter(|| ai_vs_ai(board, &aix, &aio));
    });

    let rng = rand::prelude::SmallRng::seed_from_u64(7);
    group.bench_function("single-playout", |b| {
        b.iter(|| ai_single(board, &aio, rng.clone()));
    });

    // winrate: best 99.21% @ depth 5
    let mut wins = 0u16;
    let mut draws = 0u16;
    let mut losses = 0u16;
    let mut total_turns = 0u32;
    //let mut total_moves = 0u32;
    //let mut running_avg_mov = Vec::new();

    //group.sample_size(20_00);
    group.bench_function("winrate", |b| {
        b.iter(|| {
            let (status, turns, _moves) = ai_playout(None);
            //let (status, turns) = ai_vs_ai(board.clone(), &aix, &aio);
            total_turns += turns as u32;
            //total_moves += moves as u32;
            //running_avg_mov.push(moves as f32 / turns as f32);
            match status {
                bitflag::O_PLAYER => wins += 1,
                bitflag::X_PLAYER => losses += 1,
                bitflag::STATUS_DRAW => draws += 1,
                 _ => unreachable!(),
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
        //— average possible moves: {average_possible_moves} of {total_moves} — {running_avg} {min}..{max}",
    );
}

fn do_random_moves(board: &mut Board, mut rng: SmallRng) {
    for i in 0..17 {
        let (row, column) = board
            .valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        let player = if i % 2 == 0 {
            bitflag::X_PLAYER
        } else {
            bitflag::O_PLAYER
        };
        board.do_move(row, column, player);
    }
}

fn ai_single(mut board: Board, ai: &AI, mut rng: SmallRng) -> u8 {
    loop {
        // random move
        let (row, column) = board
            .valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        // let (row, column) = aix.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, bitflag::X_PLAYER);

        // check
        let game_status = board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            return game_status;
        }

        // ai move
        let (_, (row, column)) = ai.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, bitflag::O_PLAYER);

        // check
        let game_status = board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            return game_status;
        }
    }
}

fn ai_vs_ai(mut board: Board, aix: &AI, aio: &AI) -> (u8, u8) {
    let mut turns = 0;
    loop {
        let (_, (row, column)) = aix.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, bitflag::X_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            return (game_status, turns);
        }

        // ai move
        let (_, (row, column)) = aio.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, bitflag::O_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            return (game_status, turns);
        }
    }
}

fn ai_playout(seed: Option<u64>) -> (u8, u8, u16) {
    let mut board = Board::new();
    let ai = AI::default();
    let mut turns = 0;
    let possible_moves = 0;

    let mut rng = if let Some(seed) = seed {
        rand::prelude::SmallRng::seed_from_u64(seed)
    } else {
        rand::prelude::SmallRng::from_rng(&mut rand::rng())
    };

    loop {
        // random move

        //possible_moves += (board.valid_moves().collect::<Vec<(u8, u8)>>().len()) as u16;
        let (row, column) = board
            .valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        // let (row, column) = aix.calculate_move_par(&board, DEPTH);
        //valid_moves = board.valid_moves() 
        board.do_move(row, column, bitflag::X_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return (game_status, turns, possible_moves);
        }

        // ai move
        //possible_moves += (board.valid_moves().collect::<Vec<(u8, u8)>>().len()) as u16;
        let (_, (row, column)) = ai.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, bitflag::O_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != bitflag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return (game_status, turns, possible_moves);
        }
    }
}

criterion_group!(benches, winrate_benchmark);
criterion_main!(benches);
