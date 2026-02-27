use crate::game::Game;
use crate::player::Player;
use crate::position::Position;
use crate::r#move::Move;

/// Number of planes for piece positions (1 for RED + 1 for YELLOW)
const PIECE_PLANES: usize = 1 + 1;

/// Number of positions in the game history to encode
pub const HISTORY_LENGTH: usize = 8;

/// Number of constant planes (1 for current player color)
const CONSTANT_PLANES: usize = 1;

/// Total number of input planes for the neural network
pub const TOTAL_INPUT_PLANES: usize = (HISTORY_LENGTH * PIECE_PLANES) + CONSTANT_PLANES;

/// Encode the full game state into a flat f32 array for efficient transfer to Python/numpy
/// Returns (flat_data, num_planes, height, width), where flat_data is in row-major order
pub fn encode_game_planes<const NW: usize>(game: &mut Game<NW>) -> (Vec<f32>, usize, usize, usize) {
    let perspective = game.turn();
    let width = game.width() as usize;
    let height = game.height() as usize;
    let num_planes = TOTAL_INPUT_PLANES;
    let board_size = height * width;
    let total_size = num_planes * board_size;
    let mut data = vec![0.0f32; total_size];

    let history = game.move_history();
    let history_len = history.len();
    let steps_back = (HISTORY_LENGTH - 1).min(history_len);

    let moves_to_replay: Vec<Move> = history[(history_len - steps_back)..].to_vec();

    // T=0: current position
    fill_connect4_planes(&mut data, game, perspective, 0, width, height);

    // T=1..steps_back: walk backward through history
    for t in 1..=steps_back {
        game.unmake_move();
        fill_connect4_planes(&mut data, game, perspective, t, width, height);
    }

    // Replay saved moves to restore game state
    for mv in &moves_to_replay {
        game.make_move(mv);
    }

    // Color plane (last plane)
    let color_plane = HISTORY_LENGTH * PIECE_PLANES;
    let color_value = if perspective == Player::Red { 1.0 } else { 0.0 };
    let color_offset = color_plane * board_size;
    for i in 0..board_size {
        data[color_offset + i] = color_value;
    }

    (data, num_planes, height, width)
}

fn fill_connect4_planes<const NW: usize>(
    data: &mut [f32],
    game: &Game<NW>,
    perspective: Player,
    t: usize,
    width: usize,
    height: usize,
) {
    let opponent = perspective.opposite();
    let board_size = height * width;
    let own_offset = t * PIECE_PLANES * board_size;
    let opp_offset = (t * PIECE_PLANES + 1) * board_size;

    for row in 0..height {
        for col in 0..width {
            let pos = Position::new(col as u8, row as u8);
            if let Some(player) = game.board().get_piece(&pos) {
                let idx = row * width + col;
                if player == perspective {
                    data[own_offset + idx] = 1.0;
                } else if player == opponent {
                    data[opp_offset + idx] = 1.0;
                }
            }
        }
    }
}

/// Encode a move as an action index for the policy head
pub fn encode_move(move_: &Move) -> usize {
    move_.col as usize
}

/// Decode an action index back to a move
/// Returns the column number and row where the piece would land
pub fn decode_move<const NW: usize>(action: usize, game: &Game<NW>) -> Option<Move> {
    if action >= game.width() as usize {
        return None;
    }

    let col = action as u8;
    let row = game.board().column_height(col, game.geo());

    if row >= game.height() {
        return None;
    }

    Some(Move::new(col, row))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::nw_for_board;
    use crate::board::{STANDARD_COLS, STANDARD_ROWS};

    type StdGame = Game<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>;

    fn standard_game() -> StdGame {
        StdGame::new(STANDARD_COLS, STANDARD_ROWS)
    }

    fn get_plane_value(
        data: &[f32],
        plane: usize,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
    ) -> f32 {
        data[plane * height * width + row * width + col]
    }

    #[test]
    fn test_encode_game_empty() {
        let mut game = standard_game();
        let (data, num_planes, height, width) = encode_game_planes(&mut game);

        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, game.height() as usize);
        assert_eq!(width, game.width() as usize);
        assert_eq!(data.len(), num_planes * height * width);

        for plane in 0..PIECE_PLANES {
            for row in 0..height {
                for col in 0..width {
                    assert_eq!(get_plane_value(&data, plane, row, col, height, width), 0.0);
                }
            }
        }
    }

    #[test]
    fn test_encode_decode_move() {
        let game = standard_game();

        for col in 0..game.width() {
            let move_ = Move::new(col, 0);
            let encoded = encode_move(&move_);
            assert_eq!(encoded, col as usize);

            let decoded = decode_move(encoded, &game).unwrap();
            assert_eq!(decoded.col, col);
            assert_eq!(decoded.row, 0);
        }
    }

    #[test]
    fn test_encode_game_with_pieces() {
        let mut game = standard_game();

        let move1 = Move::new(0, 0);
        game.make_move(&move1);

        let move2 = Move::new(1, 0);
        game.make_move(&move2);

        let (data, _num_planes, height, width) = encode_game_planes(&mut game);

        assert_eq!(get_plane_value(&data, 0, 0, 0, height, width), 1.0);
        assert_eq!(get_plane_value(&data, 0, 0, 1, height, width), 0.0);

        assert_eq!(get_plane_value(&data, 1, 0, 0, height, width), 0.0);
        assert_eq!(get_plane_value(&data, 1, 0, 1, height, width), 1.0);
    }

    #[test]
    fn test_fuzz_encoding_random_games() {
        use rand::prelude::IndexedRandom;
        use rand::SeedableRng;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc;
        use std::thread;

        let num_games = 5_000;
        let num_threads = num_cpus::get();
        let games_per_thread = num_games / num_threads;

        let total_moves_played = Arc::new(AtomicU64::new(0));
        let total_moves_tested = Arc::new(AtomicU64::new(0));

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let moves_played = Arc::clone(&total_moves_played);
            let moves_tested = Arc::clone(&total_moves_tested);

            let handle = thread::spawn(move || {
                let mut rng = rand::rngs::StdRng::seed_from_u64(thread_id as u64);
                let mut thread_moves_played = 0u64;
                let mut thread_moves_tested = 0u64;

                for _game_num in 0..games_per_thread {
                    let mut game = standard_game();
                    let max_moves = 42;

                    for _move_num in 0..max_moves {
                        if game.is_over() {
                            break;
                        }

                        let legal_moves = game.legal_moves();
                        if legal_moves.is_empty() {
                            break;
                        }

                        let (data, num_planes, height, width) = encode_game_planes(&mut game);
                        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
                        assert_eq!(height, game.height() as usize);
                        assert_eq!(width, game.width() as usize);
                        assert_eq!(data.len(), num_planes * height * width);

                        for move_ in &legal_moves {
                            let action = encode_move(move_);
                            assert!(
                                action < game.width() as usize,
                                "Invalid action {} for move col {}",
                                action,
                                move_.col
                            );

                            let decoded = decode_move(action, &game);
                            assert!(decoded.is_some(), "Failed to decode action {}", action);

                            let decoded_move = decoded.unwrap();
                            assert_eq!(
                                decoded_move.col, move_.col,
                                "Decoded column {} doesn't match original {}",
                                decoded_move.col, move_.col
                            );

                            thread_moves_tested += 1;
                        }

                        let chosen_move = legal_moves.choose(&mut rng).unwrap();
                        let success = game.make_move(chosen_move);
                        assert!(success, "Failed to make move col {}", chosen_move.col);

                        thread_moves_played += 1;
                    }
                }

                moves_played.fetch_add(thread_moves_played, Ordering::Relaxed);
                moves_tested.fetch_add(thread_moves_tested, Ordering::Relaxed);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_moves_played = total_moves_played.load(Ordering::Relaxed);
        let final_moves_tested = total_moves_tested.load(Ordering::Relaxed);

        println!(
            "\nConnect4 Encoding Fuzz Test:\n  Games: {}\n  Threads: {}\n  Moves played: {}\n  Moves tested: {}",
            num_games, num_threads, final_moves_played, final_moves_tested
        );

        assert!(final_moves_played > 0, "No moves were played");
        assert!(final_moves_tested > 0, "No moves were tested");
    }

    #[test]
    fn test_encoding_consistency() {
        use rand::prelude::IndexedRandom;
        use rand::SeedableRng;

        let mut game = standard_game();
        let mut rng = rand::rngs::StdRng::seed_from_u64(123);

        for _ in 0..20 {
            if game.is_over() {
                break;
            }

            let legal_moves = game.legal_moves();
            if legal_moves.is_empty() {
                break;
            }

            let encoding1 = encode_game_planes(&mut game);
            let encoding2 = encode_game_planes(&mut game);
            assert_eq!(encoding1, encoding2, "Encoding should be deterministic");

            let chosen_move = legal_moves.choose(&mut rng).unwrap();
            game.make_move(chosen_move);
        }
    }

    #[test]
    fn test_encoding_after_undo() {
        let mut game = standard_game();

        let initial_encoding = encode_game_planes(&mut game);

        let move1 = Move::new(0, 0);
        game.make_move(&move1);

        let move2 = Move::new(1, 0);
        game.make_move(&move2);

        game.unmake_move();
        game.unmake_move();

        let final_encoding = encode_game_planes(&mut game);
        assert_eq!(
            initial_encoding, final_encoding,
            "Encoding after undo should match initial state"
        );
    }

    #[test]
    fn test_encoding_different_positions() {
        let mut game1 = standard_game();
        let mut game2 = standard_game();

        game1.make_move(&Move::new(0, 0));
        game2.make_move(&Move::new(1, 0));

        let encoding1 = encode_game_planes(&mut game1);
        let encoding2 = encode_game_planes(&mut game2);

        assert_ne!(
            encoding1, encoding2,
            "Different positions should have different encodings"
        );
    }

    #[test]
    fn test_invalid_action_decoding() {
        let game = standard_game();

        assert!(decode_move(game.width() as usize, &game).is_none());
        assert!(decode_move(game.width() as usize + 1, &game).is_none());
        assert!(decode_move(100, &game).is_none());
    }

    #[test]
    fn test_encoding_full_column() {
        let mut game = standard_game();

        for i in 0..game.height() {
            let move_ = Move::new(0, i);
            game.make_move(&move_);
        }

        let legal_moves = game.legal_moves();
        assert!(legal_moves.iter().all(|m| m.col != 0));

        let decoded = decode_move(0, &game);
        assert!(decoded.is_none(), "Decoding full column should return None");
    }

    #[test]
    fn test_encode_arbitrary_board_size_10x8() {
        let mut game = Game::<{ nw_for_board(10, 8) }>::new(10, 8);

        assert_eq!(game.width(), 10);
        assert_eq!(game.height(), 8);

        let (data, num_planes, height, width) = encode_game_planes(&mut game);
        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, 8);
        assert_eq!(width, 10);
        assert_eq!(data.len(), num_planes * height * width);

        for col in 0..10u8 {
            let move_ = Move::new(col, 0);
            let encoded = encode_move(&move_);
            assert_eq!(encoded, col as usize);

            let decoded = decode_move(encoded, &game).unwrap();
            assert_eq!(decoded.col, col);
        }

        assert!(decode_move(10, &game).is_none());
    }

    #[test]
    fn test_encode_arbitrary_board_size_5x5() {
        let mut game = Game::<{ nw_for_board(5, 5) }>::new(5, 5);

        assert_eq!(game.width(), 5);
        assert_eq!(game.height(), 5);

        let (data, num_planes, height, width) = encode_game_planes(&mut game);
        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, 5);
        assert_eq!(width, 5);
        assert_eq!(data.len(), num_planes * height * width);

        let mut test_game = game.clone();
        for _ in 0..25 {
            let legal_moves = test_game.legal_moves();
            if legal_moves.is_empty() {
                break;
            }
            test_game.make_move(&legal_moves[0]);
        }

        let (data2, num_planes2, height2, width2) = encode_game_planes(&mut test_game);
        assert_eq!(num_planes2, TOTAL_INPUT_PLANES);
        assert_eq!(data2.len(), num_planes2 * height2 * width2);
    }

    #[test]
    fn test_encode_different_board_sizes_different_encodings() {
        let mut game1 = Game::<{ nw_for_board(7, 6) }>::new(7, 6);
        let mut game2 = Game::<{ nw_for_board(10, 8) }>::new(10, 8);

        let (data1, num_planes1, height1, width1) = encode_game_planes(&mut game1);
        let (data2, num_planes2, height2, width2) = encode_game_planes(&mut game2);

        assert_eq!(num_planes1, TOTAL_INPUT_PLANES);
        assert_eq!(num_planes2, TOTAL_INPUT_PLANES);

        assert_eq!((height1, width1), (6, 7));
        assert_eq!((height2, width2), (8, 10));

        assert_ne!(data1.len(), data2.len());
    }
}
