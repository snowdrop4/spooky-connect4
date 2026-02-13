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
pub fn encode_game_planes(game: &Game) -> (Vec<f32>, usize, usize, usize) {
    let perspective = game.turn();
    let opponent = perspective.opposite();
    let width = game.width();
    let height = game.height();
    let num_planes = TOTAL_INPUT_PLANES;
    let total_size = num_planes * height * width;
    let mut data = vec![0.0f32; total_size];

    // Helper to set a value in the flat array
    let set_plane_value =
        |data: &mut Vec<f32>, plane: usize, row: usize, col: usize, value: f32| {
            data[plane * height * width + row * width + col] = value;
        };

    // Plane for current player's pieces
    for row in 0..height {
        for col in 0..width {
            let pos = Position::new(col, row);
            if let Some(player) = game.board().get_piece(&pos) {
                if player == perspective {
                    set_plane_value(&mut data, 0, row, col, 1.0);
                }
            }
        }
    }

    // Plane for opponent's pieces
    for row in 0..height {
        for col in 0..width {
            let pos = Position::new(col, row);
            if let Some(player) = game.board().get_piece(&pos) {
                if player == opponent {
                    set_plane_value(&mut data, 1, row, col, 1.0);
                }
            }
        }
    }

    // Historical positions (planes 2 to HISTORY_LENGTH * PIECE_PLANES - 1) are zeros

    // Color plane (last plane)
    let color_plane = HISTORY_LENGTH * PIECE_PLANES;
    let color_value = if perspective == Player::Red { 1.0 } else { 0.0 };
    for row in 0..height {
        for col in 0..width {
            set_plane_value(&mut data, color_plane, row, col, color_value);
        }
    }

    (data, num_planes, height, width)
}

/// Encode a move as an action index for the policy head
pub fn encode_move(move_: &Move) -> usize {
    move_.col
}

/// Decode an action index back to a move
/// Returns the column number and row where the piece would land
pub fn decode_move(action: usize, game: &Game) -> Option<Move> {
    if action >= game.width() {
        return None;
    }

    // Calculate the row where the piece would land
    let row = game.board().column_height(action);

    if row >= game.height() {
        return None;
    }

    Some(Move::new(action, row))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let game = Game::standard();
        let (data, num_planes, height, width) = encode_game_planes(&game);

        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, game.height());
        assert_eq!(width, game.width());
        assert_eq!(data.len(), num_planes * height * width);

        // First two planes (current and opponent pieces) should be zeros for empty board
        for plane in 0..PIECE_PLANES {
            for row in 0..height {
                for col in 0..width {
                    assert_eq!(get_plane_value(&data, plane, row, col, height, width), 0.0);
                }
            }
        }
    }

    #[test]
    fn test_encode_game() {
        let game = Game::standard();
        let (data, num_planes, height, width) = encode_game_planes(&game);

        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, game.height());
        assert_eq!(width, game.width());
        assert_eq!(data.len(), num_planes * height * width);
    }

    #[test]
    fn test_encode_decode_move() {
        let game = Game::standard();

        for col in 0..game.width() {
            let move_ = Move::new(col, 0);
            let encoded = encode_move(&move_);
            assert_eq!(encoded, col);

            let decoded = decode_move(encoded, &game).unwrap();
            assert_eq!(decoded.col, col);
            assert_eq!(decoded.row, 0); // Empty board, so pieces land at row 0
        }
    }

    #[test]
    fn test_encode_game_with_pieces() {
        let mut game = Game::standard();

        // Drop a red piece in column 0
        let move1 = Move::new(0, 0);
        game.make_move(&move1);

        // Drop a yellow piece in column 1
        let move2 = Move::new(1, 0);
        game.make_move(&move2);

        // Now it's Red's turn again, encode from Red's perspective
        let (data, _num_planes, height, width) = encode_game_planes(&game);

        // Check red piece plane (row 0, columns 0 and 1)
        assert_eq!(get_plane_value(&data, 0, 0, 0, height, width), 1.0); // Red piece at (row 0, col 0)
        assert_eq!(get_plane_value(&data, 0, 0, 1, height, width), 0.0); // No red piece at (row 0, col 1)

        // Check yellow piece plane
        assert_eq!(get_plane_value(&data, 1, 0, 0, height, width), 0.0); // No yellow piece at (row 0, col 0)
        assert_eq!(get_plane_value(&data, 1, 0, 1, height, width), 1.0); // Yellow piece at (row 0, col 1)
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
                    let mut game = Game::standard();
                    let max_moves = 42;

                    for _move_num in 0..max_moves {
                        if game.is_over() {
                            break;
                        }

                        let legal_moves = game.legal_moves();
                        if legal_moves.is_empty() {
                            break;
                        }

                        let (data, num_planes, height, width) = encode_game_planes(&game);
                        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
                        assert_eq!(height, game.height());
                        assert_eq!(width, game.width());
                        assert_eq!(data.len(), num_planes * height * width);

                        for move_ in &legal_moves {
                            let action = encode_move(move_);
                            assert!(
                                action < game.width(),
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
    fn test_fuzz_encoding_all_columns() {
        let game = Game::standard();
        // Test that all columns can be encoded and decoded
        for col in 0..game.width() {
            let move_ = Move::new(col, 0);

            let action = encode_move(&move_);
            assert_eq!(action, col);

            let decoded = decode_move(action, &game);
            assert!(decoded.is_some());
            assert_eq!(decoded.unwrap().col, col);
        }
    }

    #[test]
    fn test_encoding_consistency() {
        use rand::prelude::IndexedRandom;
        use rand::SeedableRng;

        let mut game = Game::standard();
        let mut rng = rand::rngs::StdRng::seed_from_u64(123);

        // Play several moves
        for _ in 0..20 {
            if game.is_over() {
                break;
            }

            let legal_moves = game.legal_moves();
            if legal_moves.is_empty() {
                break;
            }

            // Encode state multiple times - should be consistent
            let encoding1 = encode_game_planes(&game);
            let encoding2 = encode_game_planes(&game);
            assert_eq!(encoding1, encoding2, "Encoding should be deterministic");

            // Make a move
            let chosen_move = legal_moves.choose(&mut rng).unwrap();
            game.make_move(chosen_move);
        }
    }

    #[test]
    fn test_encoding_after_undo() {
        let mut game = Game::standard();

        // Encode initial state
        let initial_encoding = encode_game_planes(&game);

        // Make a move
        let move1 = Move::new(0, 0);
        game.make_move(&move1);

        // Make another move
        let move2 = Move::new(1, 0);
        game.make_move(&move2);

        // Undo both moves
        game.unmake_move();
        game.unmake_move();

        // Encoding should match initial state
        let final_encoding = encode_game_planes(&game);
        assert_eq!(
            initial_encoding, final_encoding,
            "Encoding after undo should match initial state"
        );
    }

    #[test]
    fn test_plane_sizes() {
        let game = Game::standard();
        let (data, num_planes, height, width) = encode_game_planes(&game);

        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, game.height());
        assert_eq!(width, game.width());
        assert_eq!(data.len(), num_planes * height * width);
    }

    #[test]
    fn test_encoding_different_positions() {
        let mut game1 = Game::standard();
        let mut game2 = Game::standard();

        // Play different moves
        game1.make_move(&Move::new(0, 0));
        game2.make_move(&Move::new(1, 0));

        let encoding1 = encode_game_planes(&game1);
        let encoding2 = encode_game_planes(&game2);

        // Encodings should be different
        assert_ne!(
            encoding1, encoding2,
            "Different positions should have different encodings"
        );
    }

    #[test]
    fn test_invalid_action_decoding() {
        let game = Game::standard();

        // Test out of bounds action
        assert!(decode_move(game.width(), &game).is_none());
        assert!(decode_move(game.width() + 1, &game).is_none());
        assert!(decode_move(100, &game).is_none());
    }

    #[test]
    fn test_encoding_full_column() {
        let mut game = Game::standard();

        // Fill column 0
        for i in 0..game.height() {
            let move_ = Move::new(0, i);
            game.make_move(&move_);
        }

        // Column 0 should not be in legal moves
        let legal_moves = game.legal_moves();
        assert!(legal_moves.iter().all(|m| m.col != 0));

        // Trying to decode action 0 should return None because column is full
        let decoded = decode_move(0, &game);
        assert!(decoded.is_none(), "Decoding full column should return None");
    }

    #[test]
    fn test_encode_arbitrary_board_size_10x8() {
        let game = Game::new(10, 8);

        // Test dimensions
        assert_eq!(game.width(), 10);
        assert_eq!(game.height(), 8);

        // Test encoding produces correct sized planes
        let (data, num_planes, height, width) = encode_game_planes(&game);
        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, 8);
        assert_eq!(width, 10);
        assert_eq!(data.len(), num_planes * height * width);

        // Test move encoding/decoding
        for col in 0..10 {
            let move_ = Move::new(col, 0);
            let encoded = encode_move(&move_);
            assert_eq!(encoded, col);

            let decoded = decode_move(encoded, &game).unwrap();
            assert_eq!(decoded.col, col);
        }

        // Test out of bounds
        assert!(decode_move(10, &game).is_none());
    }

    #[test]
    fn test_encode_arbitrary_board_size_5x5() {
        let game = Game::new(5, 5);

        // Test dimensions
        assert_eq!(game.width(), 5);
        assert_eq!(game.height(), 5);

        // Test encoding produces correct sized planes
        let (data, num_planes, height, width) = encode_game_planes(&game);
        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, 5);
        assert_eq!(width, 5);
        assert_eq!(data.len(), num_planes * height * width);

        // Fill the board and test encoding still works
        let mut test_game = game.clone();
        for _ in 0..25 {
            let legal_moves = test_game.legal_moves();
            if legal_moves.is_empty() {
                break;
            }
            test_game.make_move(&legal_moves[0]);
        }

        let (data2, num_planes2, height2, width2) = encode_game_planes(&test_game);
        assert_eq!(num_planes2, TOTAL_INPUT_PLANES);
        assert_eq!(data2.len(), num_planes2 * height2 * width2);
    }

    #[test]
    fn test_encode_different_board_sizes_different_encodings() {
        let game1 = Game::new(7, 6);
        let game2 = Game::new(10, 8);

        let (data1, num_planes1, height1, width1) = encode_game_planes(&game1);
        let (data2, num_planes2, height2, width2) = encode_game_planes(&game2);

        // Different board sizes should produce different dimensions
        assert_eq!(num_planes1, TOTAL_INPUT_PLANES);
        assert_eq!(num_planes2, TOTAL_INPUT_PLANES);

        // Check dimensions
        assert_eq!(height1, 6);
        assert_eq!(width1, 7);
        assert_eq!(data1.len(), num_planes1 * 6 * 7);

        assert_eq!(height2, 8);
        assert_eq!(width2, 10);
        assert_eq!(data2.len(), num_planes2 * 8 * 10);
    }
}
