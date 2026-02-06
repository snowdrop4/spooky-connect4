pub mod board;
pub mod encode;
pub mod game;
pub mod r#move;
pub mod outcome;
pub mod player;
pub mod position;

#[cfg(feature = "serde")]
pub mod serde_support;

#[cfg(feature = "python")]
extern crate pyo3;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule(gil_used = false)]
fn spooky_connect4(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use player::Player;
    use python_bindings::*;
    m.add_class::<PyBoard>()?;
    m.add_class::<PyGame>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<PyGameOutcome>()?;
    m.add("RED", Player::Red as i8)?;
    m.add("YELLOW", Player::Yellow as i8)?;
    m.add("TOTAL_INPUT_PLANES", encode::TOTAL_INPUT_PLANES)?;
    Ok(())
}

#[cfg(feature = "python")]
mod python_bindings {
    use super::*;
    use crate::board::Board;
    use crate::encode;
    use crate::game::Game;
    use crate::outcome::GameOutcome;
    use crate::player::Player;
    use crate::position::Position;
    use crate::r#move::Move;

    #[pyclass(name = "Board")]
    #[derive(Clone)]
    pub struct PyBoard {
        board: Board,
    }

    #[pymethods]
    impl PyBoard {
        #[new]
        pub fn new(width: usize, height: usize) -> PyResult<Self> {
            if width < 4 || width > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 4 and 32",
                ));
            }
            if height < 4 || height > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 4 and 32",
                ));
            }
            Ok(PyBoard {
                board: Board::new(width, height),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyBoard {
                board: Board::standard(),
            }
        }

        pub fn width(&self) -> usize {
            self.board.width()
        }

        pub fn height(&self) -> usize {
            self.board.height()
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col, row);
            self.board.get_piece(&pos).map(|p| p as i8)
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col, row);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            self.board.set_piece(&pos, player)
        }

        pub fn clear(&mut self) {
            self.board.clear()
        }

        pub fn is_board_full(&self) -> bool {
            self.board.is_board_full()
        }

        pub fn is_column_full(&self, col: usize) -> bool {
            self.board.is_column_full(col)
        }

        pub fn column_height(&self, col: usize) -> usize {
            self.board.column_height(col)
        }

        pub fn __str__(&self) -> String {
            self.board.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!(
                "Board(width={}, height={})",
                self.board.width(),
                self.board.height()
            )
        }
    }

    #[pyclass(name = "Game")]
    pub struct PyGame {
        game: Game,
    }

    #[pymethods]
    impl PyGame {
        #[new]
        pub fn new(width: usize, height: usize) -> PyResult<Self> {
            if width < 4 || width > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 4 and 32",
                ));
            }
            if height < 4 || height > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 4 and 32",
                ));
            }
            Ok(PyGame {
                game: Game::new(width, height),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyGame {
                game: Game::standard(),
            }
        }

        pub fn width(&self) -> usize {
            self.game.board().width()
        }

        pub fn height(&self) -> usize {
            self.game.board().height()
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col, row);
            self.game.get_piece(&pos).map(|p| p as i8)
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col, row);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            self.game.set_piece(&pos, player)
        }

        pub fn turn(&self) -> i8 {
            self.game.turn() as i8
        }

        pub fn is_over(&self) -> bool {
            self.game.is_over()
        }

        // ---------------------------------------------------------------------
        // Unified Game Protocol Methods
        // ---------------------------------------------------------------------

        pub fn legal_action_indices(&self) -> Vec<usize> {
            self.game
                .legal_moves()
                .into_iter()
                .map(|m| encode::encode_move(&m))
                .collect()
        }

        pub fn apply_action(&mut self, action: usize) -> bool {
            if let Some(move_) = encode::decode_move(action, &self.game) {
                self.game.make_move(&move_)
            } else {
                false
            }
        }

        pub fn action_size(&self) -> usize {
            self.game.board().width()
        }

        pub fn board_shape(&self) -> (usize, usize) {
            (self.game.board().height(), self.game.board().width())
        }

        pub fn input_plane_count(&self) -> usize {
            encode::TOTAL_INPUT_PLANES
        }

        pub fn reward_absolute(&self) -> f32 {
            self.game
                .outcome()
                .map(|o| o.encode_winner_absolute())
                .unwrap_or(0.0)
        }

        pub fn reward_from_perspective(&self, perspective: i8) -> f32 {
            self.game
                .outcome()
                .map(|o| {
                    o.encode_winner_from_perspective(
                        Player::from_int(perspective).expect("Invalid perspective"),
                    )
                })
                .unwrap_or(0.0)
        }

        pub fn name(&self) -> String {
            format!(
                "connect4_{}x{}",
                self.game.board().width(),
                self.game.board().height()
            )
        }

        pub fn outcome(&self) -> Option<PyGameOutcome> {
            self.game.outcome().map(|o| PyGameOutcome { outcome: o })
        }

        pub fn legal_moves(&self) -> Vec<PyMove> {
            self.game
                .legal_moves()
                .into_iter()
                .map(|m| PyMove { move_: m })
                .collect()
        }

        pub fn is_legal_move(&self, move_: &PyMove) -> bool {
            self.game.is_legal_move(&move_.move_)
        }

        pub fn make_move(&mut self, move_: &PyMove) -> bool {
            self.game.make_move(&move_.move_)
        }

        pub fn unmake_move(&mut self) -> bool {
            self.game.unmake_move()
        }

        pub fn board(&self) -> PyBoard {
            PyBoard {
                board: self.game.board().clone(),
            }
        }

        pub fn clone(&self) -> PyGame {
            PyGame {
                game: self.game.clone(),
            }
        }

        pub fn __hash__(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();

            self.game.board().hash(&mut hasher);
            (self.game.turn() as i8).hash(&mut hasher);

            hasher.finish()
        }

        // ---------------------------------------------------------------------
        // Encoding/decoding
        // ---------------------------------------------------------------------

        pub fn encode_game_planes(&self) -> (Vec<f32>, usize, usize, usize) {
            encode::encode_game_planes(&self.game)
        }

        pub fn decode_action(&self, action: usize) -> Option<PyMove> {
            // Find the legal move that matches this action
            encode::decode_move(action, &self.game).map(|move_| PyMove { move_: move_ })
        }

        // ---------------------------------------------------------------------
        // Dunder Methods
        // ---------------------------------------------------------------------

        pub fn __str__(&self) -> String {
            self.game.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!(
                "Game(width={}, height={}, turn={:?}, over={})",
                self.game.board().width(),
                self.game.board().height(),
                self.game.turn(),
                self.game.is_over()
            )
        }
    }

    #[pyclass(name = "Move")]
    #[derive(Clone, Debug)]
    pub struct PyMove {
        move_: Move,
    }

    #[pymethods]
    impl PyMove {
        #[new]
        pub fn new(col: usize, row: usize) -> Self {
            PyMove {
                move_: Move::new(col, row),
            }
        }

        pub fn col(&self) -> usize {
            self.move_.col
        }

        pub fn row(&self) -> usize {
            self.move_.row
        }

        // ---------------------------------------------------------------------
        // Encoding/decoding
        // ---------------------------------------------------------------------

        pub fn encode(&self) -> usize {
            encode::encode_move(&self.move_)
        }

        #[staticmethod]
        pub fn decode(data: usize, game: &PyGame) -> PyResult<Self> {
            match encode::decode_move(data, &game.game) {
                Some(mv) => Ok(PyMove { move_: mv }),
                _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "invalid move",
                )),
            }
        }

        // ---------------------------------------------------------------------
        // Dunder Methods
        // ---------------------------------------------------------------------

        pub fn __str__(&self) -> String {
            format!("col {}", self.move_.col)
        }

        pub fn __repr__(&self) -> String {
            format!("Move(col={}, row={})", self.move_.col, self.move_.row)
        }

        pub fn __eq__(&self, other: &PyMove) -> bool {
            self.move_ == other.move_
        }

        pub fn __hash__(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            self.move_.col.hash(&mut hasher);
            self.move_.row.hash(&mut hasher);
            hasher.finish()
        }
    }

    #[pyclass(name = "GameOutcome")]
    #[derive(Clone, Copy, Debug)]
    pub struct PyGameOutcome {
        outcome: GameOutcome,
    }

    #[pymethods]
    impl PyGameOutcome {
        pub fn winner(&self) -> Option<i8> {
            self.outcome.winner().map(|player| player as i8)
        }

        pub fn encode_winner_absolute(&self) -> f32 {
            self.outcome.encode_winner_absolute()
        }

        pub fn encode_winner_from_perspective(&self, perspective: i8) -> f32 {
            self.outcome.encode_winner_from_perspective(
                Player::from_int(perspective).expect("Unrecognized perspective"),
            )
        }

        pub fn is_draw(&self) -> bool {
            self.outcome.is_draw()
        }

        pub fn name(&self) -> String {
            self.outcome.to_string()
        }

        pub fn __str__(&self) -> String {
            self.outcome.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!("GameOutcome({})", self.outcome.to_string())
        }

        pub fn __eq__(&self, other: &PyGameOutcome) -> bool {
            self.outcome == other.outcome
        }
    }
}
