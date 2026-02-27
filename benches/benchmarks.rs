use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::IndexedRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use spooky_connect4::bitboard::nw_for_board;
use spooky_connect4::encode::encode_game_planes;
use spooky_connect4::game::Game;
use std::hint::black_box;

type StdGame = Game<{ nw_for_board(7, 6) }>;
type Game9x9 = Game<{ nw_for_board(9, 9) }>;
type Game19x19 = Game<{ nw_for_board(19, 19) }>;

/// Play ~10 random moves on a fresh game to create a realistic mid-game position.
fn setup_midgame_7x6() -> StdGame {
    let mut game = StdGame::new(7, 6);
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..10 {
        let moves = game.legal_moves();
        if moves.is_empty() {
            break;
        }
        let mv = moves.choose(&mut rng).unwrap();
        game.make_move(mv);
    }
    game
}

fn setup_midgame_9x9() -> Game9x9 {
    let mut game = Game9x9::new(9, 9);
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..10 {
        let moves = game.legal_moves();
        if moves.is_empty() {
            break;
        }
        let mv = moves.choose(&mut rng).unwrap();
        game.make_move(mv);
    }
    game
}

fn setup_midgame_19x19() -> Game19x19 {
    let mut game = Game19x19::new(19, 19);
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..10 {
        let moves = game.legal_moves();
        if moves.is_empty() {
            break;
        }
        let mv = moves.choose(&mut rng).unwrap();
        game.make_move(mv);
    }
    game
}

// ---------------------------------------------------------------------------
// Microbenchmarks
// ---------------------------------------------------------------------------

fn bench_legal_moves_9x9(c: &mut Criterion) {
    let game = setup_midgame_9x9();
    c.bench_function("legal_moves_9x9", |b| {
        b.iter(|| black_box(game.legal_moves()))
    });
}

fn bench_legal_moves_19x19(c: &mut Criterion) {
    let game = setup_midgame_19x19();
    c.bench_function("legal_moves_19x19", |b| {
        b.iter(|| black_box(game.legal_moves()))
    });
}

fn bench_make_move(c: &mut Criterion) {
    let game = setup_midgame_9x9();
    let moves = game.legal_moves();
    c.bench_function("make_move", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                black_box(g.make_move(moves.first().unwrap()));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_make_unmake(c: &mut Criterion) {
    let game = setup_midgame_9x9();
    let moves = game.legal_moves();
    c.bench_function("make_unmake", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                g.make_move(moves.first().unwrap());
                black_box(g.unmake_move());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_encode_game_planes_9x9(c: &mut Criterion) {
    let game = setup_midgame_9x9();
    c.bench_function("encode_game_planes_9x9", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| black_box(encode_game_planes(&mut g)),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_encode_game_planes_19x19(c: &mut Criterion) {
    let game = setup_midgame_19x19();
    c.bench_function("encode_game_planes_19x19", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| black_box(encode_game_planes(&mut g)),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_outcome(c: &mut Criterion) {
    let game = setup_midgame_9x9();
    c.bench_function("outcome", |b| b.iter(|| black_box(game.outcome())));
}

// ---------------------------------------------------------------------------
// Integration benchmarks
// ---------------------------------------------------------------------------

fn bench_random_playout_9x9(c: &mut Criterion) {
    c.bench_function("random_playout_9x9", |b| {
        b.iter(|| {
            let mut game = Game9x9::new(9, 9);
            while !game.is_over() {
                let moves = game.legal_moves();
                game.make_move(moves.first().unwrap());
            }
            black_box(game.outcome())
        })
    });
}

fn bench_random_playout_19x19(c: &mut Criterion) {
    c.bench_function("random_playout_19x19", |b| {
        b.iter(|| {
            let mut game = Game19x19::new(19, 19);
            while !game.is_over() {
                let moves = game.legal_moves();
                game.make_move(moves.first().unwrap());
            }
            black_box(game.outcome())
        })
    });
}

fn bench_self_play_step(c: &mut Criterion) {
    let game = setup_midgame_9x9();
    c.bench_function("self_play_step", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                let moves = g.legal_moves();
                let _planes = encode_game_planes(&mut g);
                g.make_move(moves.first().unwrap());
                black_box(&g);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(100_000);
    targets =
        bench_legal_moves_9x9,
        bench_legal_moves_19x19,
        bench_make_move,
        bench_make_unmake,
        bench_encode_game_planes_9x9,
        bench_encode_game_planes_19x19,
        bench_outcome,
        bench_self_play_step,
);
criterion_group!(
    name = playouts;
    config = Criterion::default().sample_size(10_000);
    targets =
        bench_random_playout_9x9,
        bench_random_playout_19x19,
);
criterion_main!(benches, playouts);
