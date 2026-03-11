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
            pub(super) enum GameInner {
                $( [<Nw $nw>](Game<$nw>), )*
            }

            #[derive(Clone, Debug)]
            pub(super) enum BoardInner {
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

            pub(super) fn make_game_inner(width: u8, height: u8) -> GameInner {
                let nw = nw_for_board(width, height);
                match nw {
                    $( $nw => GameInner::[<Nw $nw>](Game::new(width, height)), )*
                    _ => unreachable!("NW out of range: {}", nw),
                }
            }

            pub(super) fn make_board_inner(width: u8, height: u8) -> BoardInner {
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

mod py_board;
mod py_game;
mod py_game_outcome;
mod py_move;

pub use py_board::PyBoard;
pub use py_game::PyGame;
pub use py_game_outcome::PyGameOutcome;
pub use py_move::PyMove;
