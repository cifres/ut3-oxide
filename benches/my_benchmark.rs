use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use ut3_oxide::ai::AI;
use ut3_oxide::board::{flag, Board};

const DEPTH: u8 = 5;

fn winrate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("AI-performance");

    // single call speed
    let mut board = Board::new();
    let ai = AI::default();
    let mut rng = rand::prelude::SmallRng::seed_from_u64(7);
    //let mut rng = rand::rng();

    for i in 0..17 {
        let (row, column) = board.valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        let player = if i % 2 == 0 {
            flag::X_PLAYER
        } else {
            flag::O_PLAYER
        };
        board.do_move(row, column, player);
    }

    //group
    //    .significance_level(0.05);
        //group.sample_size(100);
    // 298.42 μs @ depth 5
    // 246.42 μs @ depth 5
    // 211.28 μs @ depth 5 inline only no (always)
    // 362.29 μs @ depth 5 mixed
    group.bench_function("single-call-performance", |b| {
        b.iter(|| {
            ai.calculate_move_par(&board, DEPTH);
        });
    });

    group.bench_function("single-playout", |b| {
        b.iter(ai_playout);
    });

    // winrate
    let mut wins = 0u16;
    let mut draws = 0u16;
    let mut losses = 0u16;
    let mut total_turns = 0u32;
    let duration = std::time::Duration::from_secs(17);

    // group.sample_size(10).measurement_time(duration);
    group.measurement_time(duration);
    group.bench_function("winrate", |b| {
        b.iter(|| {
            let (status, turns) = ai_playout();
            total_turns += turns as u32;
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

    let total = wins + losses + draws;
    let average_turns = total_turns as f32 / total as f32;
    let wr = (wins as f64 / total as f64) * 100.0;
    println!("{total_turns}");
    println!(
        "wr @ depth {DEPTH}: {wr:.2}% -> {wins}/{total}
        — average total turns = {average_turns:.2} 
        — losses: {losses} draws: {draws}",
    );
}

fn ai_playout() -> (u8, u8) {
    let mut board = Board::new();
    let ai = AI::default();
    let mut turns = 0;
    // let aix = AI::new(flag::X_PLAYER);

    //let mut rng = rand::rng();
    let mut rng = rand::prelude::SmallRng::seed_from_u64(7);

    loop {
        // random move
        //let validbitfield = board.valid_moves_bitfield();
        //let (row, column) = iterator::ValidMoveIterator::new(validbitfield)
        let (row, column) = board.valid_moves()
            .choose(&mut rng)
            .expect("should be able to get random move");

        // let (row, column) = aix.calculate_move_par(&board, DEPTH);
        board.do_move(row, column, flag::X_PLAYER);
        turns += 1;

        // check
        let game_status = board.calculate_game_status();
        if game_status != flag::STATUS_CONTESTABLE {
            //println!("Game over: result {game_status}");
            return (game_status, turns);
        }

        // ai move
        let (row, column) = ai.calculate_move_par(&board, DEPTH);
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

criterion_group!(benches, winrate_benchmark);
criterion_main!(benches);
