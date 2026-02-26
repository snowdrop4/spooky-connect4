pub mod bitboard;
pub mod board;
pub mod encode;
pub mod game;
pub mod r#move;
pub mod outcome;
pub mod player;
pub mod position;

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
    use crate::bitboard::nw_for_board;
    use crate::board::Board;
    use crate::encode;
    use crate::game::Game;
    use crate::outcome::GameOutcome;
    use crate::player::Player;
    use crate::position::Position;
    use crate::r#move::Move;

    // -----------------------------------------------------------------------
    // Enum dispatch via paste! for Game<NW> and Board<NW>
    // -----------------------------------------------------------------------

    macro_rules! define_dispatch {
        ($($nw:literal),*) => {
            paste::paste! {
                #[derive(Clone, Debug)]
                enum GameInner {
                    $( [<Nw $nw>](Game<$nw>), )*
                }

                #[derive(Clone, Debug)]
                enum BoardInner {
                    $( [<Nw $nw>](Board<$nw>), )*
                }

                macro_rules! dispatch_game {
                    ($self_:expr, $g:ident => $body:expr) => {
                        match $self_ {
                            $( GameInner::[<Nw $nw>]($g) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_game_mut {
                    ($self_:expr, $g:ident => $body:expr) => {
                        match $self_ {
                            $( GameInner::[<Nw $nw>]($g) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_board {
                    ($self_:expr, $b:ident => $body:expr) => {
                        match $self_ {
                            $( BoardInner::[<Nw $nw>]($b) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_board_mut {
                    ($self_:expr, $b:ident => $body:expr) => {
                        match $self_ {
                            $( BoardInner::[<Nw $nw>]($b) => $body, )*
                        }
                    };
                }

                fn make_game_inner(width: u8, height: u8) -> GameInner {
                    let nw = nw_for_board(width, height);
                    match nw {
                        $( $nw => GameInner::[<Nw $nw>](Game::new(width, height)), )*
                        _ => unreachable!("NW out of range: {}", nw),
                    }
                }

                fn make_board_inner(width: u8, height: u8) -> BoardInner {
                    let nw = nw_for_board(width, height);
                    match nw {
                        $( $nw => BoardInner::[<Nw $nw>](Board::new(width, height)), )*
                        _ => unreachable!("NW out of range: {}", nw),
                    }
                }

                macro_rules! game_to_board_inner {
                    ($game_inner:expr) => {
                        match $game_inner {
                            $( GameInner::[<Nw $nw>](g) => BoardInner::[<Nw $nw>](*g.board()), )*
                        }
                    };
                }
            }
        }
    }

    define_dispatch!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

    // -----------------------------------------------------------------------
    // PyBoard
    // -----------------------------------------------------------------------

    #[pyclass(name = "Board")]
    #[derive(Clone)]
    pub struct PyBoard {
        inner: BoardInner,
    }

    #[pymethods]
    impl PyBoard {
        #[new]
        pub fn new(width: usize, height: usize) -> PyResult<Self> {
            if !(4..=32).contains(&width) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 4 and 32",
                ));
            }
            if !(4..=32).contains(&height) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 4 and 32",
                ));
            }
            Ok(PyBoard {
                inner: make_board_inner(width as u8, height as u8),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyBoard {
                inner: make_board_inner(7, 6),
            }
        }

        pub fn width(&self) -> usize {
            dispatch_board!(&self.inner, b => b.width() as usize)
        }

        pub fn height(&self) -> usize {
            dispatch_board!(&self.inner, b => b.height() as usize)
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col as u8, row as u8);
            dispatch_board!(&self.inner, b => b.get_piece(&pos).map(|p| p as i8))
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col as u8, row as u8);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            dispatch_board_mut!(&mut self.inner, b => b.set_piece(&pos, player))
        }

        pub fn clear(&mut self) {
            dispatch_board_mut!(&mut self.inner, b => b.clear())
        }

        pub fn is_board_full(&self) -> bool {
            dispatch_board!(&self.inner, b => {
                let geo = crate::bitboard::BoardGeometry::new(b.width(), b.height());
                b.is_board_full(&geo)
            })
        }

        pub fn is_column_full(&self, col: usize) -> bool {
            dispatch_board!(&self.inner, b => {
                let geo = crate::bitboard::BoardGeometry::new(b.width(), b.height());
                b.is_column_full(col as u8, &geo)
            })
        }

        pub fn column_height(&self, col: usize) -> usize {
            dispatch_board!(&self.inner, b => {
                let geo = crate::bitboard::BoardGeometry::new(b.width(), b.height());
                b.column_height(col as u8, &geo) as usize
            })
        }

        pub fn __str__(&self) -> String {
            dispatch_board!(&self.inner, b => b.to_string())
        }

        pub fn __repr__(&self) -> String {
            let w = self.width();
            let h = self.height();
            format!("Board(width={}, height={})", w, h)
        }
    }

    // -----------------------------------------------------------------------
    // PyGame
    // -----------------------------------------------------------------------

    #[pyclass(name = "Game")]
    pub struct PyGame {
        inner: GameInner,
    }

    #[pymethods]
    impl PyGame {
        #[new]
        pub fn new(width: usize, height: usize) -> PyResult<Self> {
            if !(4..=32).contains(&width) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 4 and 32",
                ));
            }
            if !(4..=32).contains(&height) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 4 and 32",
                ));
            }
            Ok(PyGame {
                inner: make_game_inner(width as u8, height as u8),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyGame {
                inner: make_game_inner(7, 6),
            }
        }

        pub fn width(&self) -> usize {
            dispatch_game!(&self.inner, g => g.width() as usize)
        }

        pub fn height(&self) -> usize {
            dispatch_game!(&self.inner, g => g.height() as usize)
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col as u8, row as u8);
            dispatch_game!(&self.inner, g => g.get_piece(&pos))
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col as u8, row as u8);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            dispatch_game_mut!(&mut self.inner, g => g.set_piece(&pos, player))
        }

        pub fn turn(&self) -> i8 {
            dispatch_game!(&self.inner, g => g.turn() as i8)
        }

        pub fn is_over(&self) -> bool {
            dispatch_game!(&self.inner, g => g.is_over())
        }

        // ---------------------------------------------------------------------
        // Unified Game Protocol Methods
        // ---------------------------------------------------------------------

        pub fn legal_action_indices(&self) -> Vec<usize> {
            dispatch_game!(&self.inner, g => {
                g.legal_moves()
                    .into_iter()
                    .map(|m| encode::encode_move(&m))
                    .collect()
            })
        }

        pub fn apply_action(&mut self, action: usize) -> bool {
            dispatch_game_mut!(&mut self.inner, g => {
                if let Some(move_) = encode::decode_move(action, g) {
                    g.make_move(&move_)
                } else {
                    false
                }
            })
        }

        pub fn action_size(&self) -> usize {
            dispatch_game!(&self.inner, g => g.width() as usize)
        }

        pub fn board_shape(&self) -> (usize, usize) {
            dispatch_game!(&self.inner, g => (g.height() as usize, g.width() as usize))
        }

        pub fn input_plane_count(&self) -> usize {
            encode::TOTAL_INPUT_PLANES
        }

        pub fn reward_absolute(&self) -> f32 {
            dispatch_game!(&self.inner, g => {
                g.outcome()
                    .map(|o| o.encode_winner_absolute())
                    .unwrap_or(0.0)
            })
        }

        pub fn reward_from_perspective(&self, perspective: i8) -> f32 {
            dispatch_game!(&self.inner, g => {
                g.outcome()
                    .map(|o| {
                        o.encode_winner_from_perspective(
                            Player::from_int(perspective).expect("Invalid perspective"),
                        )
                    })
                    .unwrap_or(0.0)
            })
        }

        pub fn name(&self) -> String {
            dispatch_game!(&self.inner, g => format!("connect4_{}x{}", g.width(), g.height()))
        }

        pub fn outcome(&self) -> Option<PyGameOutcome> {
            dispatch_game!(&self.inner, g => g.outcome().map(|o| PyGameOutcome { outcome: o }))
        }

        pub fn legal_moves(&self) -> Vec<PyMove> {
            dispatch_game!(&self.inner, g => {
                g.legal_moves()
                    .into_iter()
                    .map(|m| PyMove { move_: m })
                    .collect()
            })
        }

        pub fn is_legal_move(&self, move_: &PyMove) -> bool {
            dispatch_game!(&self.inner, g => g.is_legal_move(&move_.move_))
        }

        pub fn make_move(&mut self, move_: &PyMove) -> bool {
            dispatch_game_mut!(&mut self.inner, g => g.make_move(&move_.move_))
        }

        pub fn unmake_move(&mut self) -> bool {
            dispatch_game_mut!(&mut self.inner, g => g.unmake_move())
        }

        pub fn board(&self) -> PyBoard {
            PyBoard {
                inner: game_to_board_inner!(&self.inner),
            }
        }

        pub fn clone(&self) -> PyGame {
            PyGame {
                inner: self.inner.clone(),
            }
        }

        pub fn __hash__(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            dispatch_game!(&self.inner, g => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                g.board().hash(&mut hasher);
                (g.turn() as i8).hash(&mut hasher);
                hasher.finish()
            })
        }

        // ---------------------------------------------------------------------
        // Encoding/decoding
        // ---------------------------------------------------------------------

        pub fn encode_game_planes(&self) -> (Vec<f32>, usize, usize, usize) {
            dispatch_game!(&self.inner, g => encode::encode_game_planes(g))
        }

        pub fn decode_action(&self, action: usize) -> Option<PyMove> {
            dispatch_game!(&self.inner, g => {
                encode::decode_move(action, g).map(|move_| PyMove { move_ })
            })
        }

        // ---------------------------------------------------------------------
        // Dunder Methods
        // ---------------------------------------------------------------------

        pub fn __str__(&self) -> String {
            dispatch_game!(&self.inner, g => g.to_string())
        }

        pub fn __repr__(&self) -> String {
            dispatch_game!(&self.inner, g => {
                format!(
                    "Game(width={}, height={}, turn={:?}, over={})",
                    g.width(),
                    g.height(),
                    g.turn(),
                    g.is_over()
                )
            })
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
                move_: Move::new(col as u8, row as u8),
            }
        }

        pub fn col(&self) -> usize {
            self.move_.col as usize
        }

        pub fn row(&self) -> usize {
            self.move_.row as usize
        }

        // ---------------------------------------------------------------------
        // Encoding/decoding
        // ---------------------------------------------------------------------

        pub fn encode(&self) -> usize {
            encode::encode_move(&self.move_)
        }

        #[staticmethod]
        pub fn decode(data: usize, game: &PyGame) -> PyResult<Self> {
            dispatch_game!(&game.inner, g => {
                match encode::decode_move(data, g) {
                    Some(mv) => Ok(PyMove { move_: mv }),
                    _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "invalid move",
                    )),
                }
            })
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
            format!("GameOutcome({})", self.outcome)
        }

        pub fn __eq__(&self, other: &PyGameOutcome) -> bool {
            self.outcome == other.outcome
        }
    }
}
