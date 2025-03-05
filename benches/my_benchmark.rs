use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use std::hint::black_box;

use ut3_oxide::ai::AI;
use ut3_oxide::board::{flag, Board, ValidMoveIterator};

fn winrate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("AI-performance");
    // single call speed
    let mut board = Board::new();
    let ai = AI::default();
    let mut rng = rand::prelude::SmallRng::seed_from_u64(7);

    for i in 0..16 {
        let validbitfield = board.valid_moves_bitfield();
        let (row, column) = ValidMoveIterator::new(validbitfield)
            .choose(&mut rng)
            .expect("should be able to get random move");

        let player = if i % 2 == 0 {
            flag::X_PLAYER
        } else {
            flag::O_PLAYER
        };
        board.do_move(row, column, player);
    }

    group.bench_function("single-call-performance", |b| {
        b.iter(|| {
            black_box(ai.calculate_move_par(&board, 6));
        });
    });

    // winrate
    let iterations = 10;
    let mut wins = 0u8;
    let mut draws = 0u8;
    let mut loses = 0u8;
    let duration = std::time::Duration::from_secs(7);

    group.sample_size(10).measurement_time(duration);
    group.bench_function("winrate", |b| {
        b.iter(|| {
            for _ in 0..black_box(iterations) {
                let status = ai_playout();
                if status == flag::O_PLAYER {
                    wins += 1;
                } else if status == flag::STATUS_DRAW {
                    draws += 1;
                } else if status == flag::X_PLAYER {
                    loses += 1;
                }
            }
        });
    });

    group.finish();
    let wr = (wins as f64 / (wins + loses + draws) as f64) * 100.0;
    let total = wins + loses + draws;
    println!("wr {wr:.3}% -> {wins}/{total} — loses: {loses} draws: {draws}");
}

fn ai_playout() -> u8 {
    let mut board = Board::new();
    let ai = AI::default();
    //let mut rng = rand::rng();
    let mut rng = rand::prelude::SmallRng::seed_from_u64(7);

    loop {
        // random move
        let validbitfield = board.valid_moves_bitfield();
        let (row, column) = ValidMoveIterator::new(validbitfield)
            .choose(&mut rng)
            .expect("should be able to get random move");

        board.do_move(row, column, flag::X_PLAYER);

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return game_status;
        }

        // ai move
        let (row, column) = ai.calculate_move_par(&board, 6);
        board.do_move(row, column, flag::O_PLAYER);

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return game_status;
        }
    }
}

criterion_group!(benches, winrate_benchmark);
criterion_main!(benches);
