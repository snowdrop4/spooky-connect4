use std::fmt;
use std::hash::{Hash, Hasher};

use crate::bitboard::{nw_for_board, Bitboard, BoardGeometry};
use crate::player::Player;
use crate::position::Position;

pub const STANDARD_COLS: u8 = 7;
pub const STANDARD_ROWS: u8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board<const NW: usize> {
    red: Bitboard<NW>,
    yellow: Bitboard<NW>,
    width: u8,
    height: u8,
}

impl<const NW: usize> Hash for Board<NW> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.red.hash(state);
        self.yellow.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl<const NW: usize> Board<NW> {
    pub fn new(width: u8, height: u8) -> Self {
        Board {
            red: Bitboard::empty(),
            yellow: Bitboard::empty(),
            width,
            height,
        }
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn get_piece(&self, pos: &Position) -> Option<Player> {
        if pos.is_valid(self.width, self.height) {
            let idx = pos.to_index(self.width);
            if self.red.get(idx) {
                Some(Player::Red)
            } else if self.yellow.get(idx) {
                Some(Player::Yellow)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_piece(&mut self, pos: &Position, player: Option<Player>) {
        if pos.is_valid(self.width, self.height) {
            let idx = pos.to_index(self.width);
            self.red.clear(idx);
            self.yellow.clear(idx);
            match player {
                Some(Player::Red) => self.red.set(idx),
                Some(Player::Yellow) => self.yellow.set(idx),
                None => {}
            }
        }
    }

    pub fn clear(&mut self) {
        self.red = Bitboard::empty();
        self.yellow = Bitboard::empty();
    }

    #[inline]
    pub(crate) fn stones_for(&self, player: Player) -> Bitboard<NW> {
        match player {
            Player::Red => self.red,
            Player::Yellow => self.yellow,
        }
    }

    #[inline]
    pub(crate) fn occupied(&self) -> Bitboard<NW> {
        self.red | self.yellow
    }

    /// Set a single bit for a player (no clearing — caller must ensure position is empty).
    #[inline]
    pub(crate) fn set_bit(&mut self, idx: usize, player: Player) {
        match player {
            Player::Red => self.red.set(idx),
            Player::Yellow => self.yellow.set(idx),
        }
    }

    /// Drop a piece into the given column. Returns the row it landed on, or None if column is full.
    pub fn drop_piece(&mut self, col: u8, player: Player, geo: &BoardGeometry<NW>) -> Option<u8> {
        let col_usize = col as usize;
        if col_usize >= self.width as usize {
            return None;
        }

        let occupied = self.occupied();
        // Find the lowest empty row in this column
        let col_mask = geo.column_masks[col_usize];
        let empty_in_col = col_mask.andnot(occupied);

        if let Some(bit_idx) = empty_in_col.lowest_bit_index() {
            let row = (bit_idx / self.width as usize) as u8;
            self.set_bit(bit_idx, player);
            Some(row)
        } else {
            None // Column is full
        }
    }

    /// Get the number of pieces in a column.
    pub fn column_height(&self, col: u8, geo: &BoardGeometry<NW>) -> u8 {
        let col_usize = col as usize;
        if col_usize >= self.width as usize {
            return 0;
        }
        (self.occupied() & geo.column_masks[col_usize]).count() as u8
    }

    /// Check if a column is full.
    pub fn is_column_full(&self, col: u8, _geo: &BoardGeometry<NW>) -> bool {
        let col_usize = col as usize;
        if col_usize >= self.width as usize {
            return true;
        }
        // Check if the top cell in this column is occupied
        let top_idx = (self.height as usize - 1) * self.width as usize + col_usize;
        self.occupied().get(top_idx)
    }

    /// Check if the board is completely full.
    pub fn is_board_full(&self, geo: &BoardGeometry<NW>) -> bool {
        (self.occupied() & geo.top_row_mask) == geo.top_row_mask
    }

    /// Check if the given player has won.
    pub fn check_win(&self, player: Player, geo: &BoardGeometry<NW>) -> bool {
        geo.has_four_in_a_row(&self.stones_for(player))
    }
}

impl Default for Board<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }> {
    fn default() -> Self {
        Self::new(STANDARD_COLS, STANDARD_ROWS)
    }
}

impl<const NW: usize> fmt::Display for Board<NW> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in (0..self.height as usize).rev() {
            write!(f, "|")?;

            for col in 0..self.width as usize {
                let pos = Position::new(col as u8, row as u8);
                let c = if let Some(player) = self.get_piece(&pos) {
                    player.to_char()
                } else {
                    '.'
                };

                write!(f, "{}", c)?;
                write!(f, "|")?;
            }

            writeln!(f)?;
        }

        // Column numbers
        write!(f, " ")?;
        for col in 0..self.width as usize {
            write!(f, "{} ", col)?;
        }
        writeln!(f)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_geo() -> BoardGeometry<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }> {
        BoardGeometry::new(STANDARD_COLS, STANDARD_ROWS)
    }

    #[test]
    fn test_empty_board_creation() {
        let geo = make_geo();
        let board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );
        assert!(!board.is_board_full(&geo));
        for col in 0..STANDARD_COLS {
            assert_eq!(board.column_height(col, &geo), 0);
            assert!(!board.is_column_full(col, &geo));
        }
    }

    #[test]
    fn test_empty_board_creation_10x10() {
        let geo = BoardGeometry::<{ nw_for_board(10, 10) }>::new(10, 10);
        let board = Board::<{ nw_for_board(10, 10) }>::new(10, 10);
        assert!(!board.is_board_full(&geo));
        for col in 0..10 {
            assert_eq!(board.column_height(col, &geo), 0);
            assert!(!board.is_column_full(col, &geo));
        }
    }

    #[test]
    fn test_drop_piece() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );
        let row = board.drop_piece(0, Player::Red, &geo);
        assert_eq!(row, Some(0));

        let pos = Position::new(0, 0);
        assert_eq!(board.get_piece(&pos), Some(Player::Red));
    }

    #[test]
    fn test_drop_multiple_pieces() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        assert_eq!(board.drop_piece(0, Player::Red, &geo), Some(0));
        assert_eq!(board.drop_piece(0, Player::Yellow, &geo), Some(1));
        assert_eq!(board.drop_piece(0, Player::Red, &geo), Some(2));

        assert_eq!(board.column_height(0, &geo), 3);
    }

    #[test]
    fn test_column_full() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        for _ in 0..STANDARD_ROWS {
            board.drop_piece(0, Player::Red, &geo);
        }

        assert!(board.is_column_full(0, &geo));
        assert_eq!(board.drop_piece(0, Player::Red, &geo), None);
    }

    #[test]
    fn test_board_full() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        for col in 0..STANDARD_COLS {
            for _ in 0..STANDARD_ROWS {
                board.drop_piece(col, Player::Red, &geo);
            }
        }

        assert!(board.is_board_full(&geo));
    }

    #[test]
    fn test_horizontal_win() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        for col in 0..4 {
            board.drop_piece(col, Player::Red, &geo);
        }

        assert!(board.check_win(Player::Red, &geo));
    }

    #[test]
    fn test_vertical_win() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        for _ in 0..4 {
            board.drop_piece(0, Player::Red, &geo);
        }

        assert!(board.check_win(Player::Red, &geo));
    }

    #[test]
    fn test_diagonal_win_ascending() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        // Col 0: R
        board.drop_piece(0, Player::Red, &geo);
        // Col 1: Y, R
        board.drop_piece(1, Player::Yellow, &geo);
        board.drop_piece(1, Player::Red, &geo);
        // Col 2: Y, Y, R
        board.drop_piece(2, Player::Yellow, &geo);
        board.drop_piece(2, Player::Yellow, &geo);
        board.drop_piece(2, Player::Red, &geo);
        // Col 3: Y, Y, Y, R
        board.drop_piece(3, Player::Yellow, &geo);
        board.drop_piece(3, Player::Yellow, &geo);
        board.drop_piece(3, Player::Yellow, &geo);
        board.drop_piece(3, Player::Red, &geo);

        assert!(board.check_win(Player::Red, &geo));
    }

    #[test]
    fn test_diagonal_win_descending() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        // Col 0: Y, Y, Y, R
        board.drop_piece(0, Player::Yellow, &geo);
        board.drop_piece(0, Player::Yellow, &geo);
        board.drop_piece(0, Player::Yellow, &geo);
        board.drop_piece(0, Player::Red, &geo);
        // Col 1: Y, Y, R
        board.drop_piece(1, Player::Yellow, &geo);
        board.drop_piece(1, Player::Yellow, &geo);
        board.drop_piece(1, Player::Red, &geo);
        // Col 2: Y, R
        board.drop_piece(2, Player::Yellow, &geo);
        board.drop_piece(2, Player::Red, &geo);
        // Col 3: R
        board.drop_piece(3, Player::Red, &geo);

        assert!(board.check_win(Player::Red, &geo));
    }

    #[test]
    fn test_no_win() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        for col in 0..3 {
            board.drop_piece(col, Player::Red, &geo);
        }

        assert!(!board.check_win(Player::Red, &geo));
    }

    #[test]
    fn test_get_set() {
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );
        let pos = Position::new(3, 2);

        assert_eq!(board.get_piece(&pos), None);

        board.set_piece(&pos, Some(Player::Red));
        assert_eq!(board.get_piece(&pos), Some(Player::Red));

        board.set_piece(&pos, None);
        assert_eq!(board.get_piece(&pos), None);
    }

    #[test]
    fn test_out_of_bounds() {
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        let pos = Position::new(10, 10);
        assert_eq!(board.get_piece(&pos), None);

        board.set_piece(&pos, Some(Player::Red));
        assert_eq!(board.get_piece(&pos), None);
    }

    #[test]
    fn test_invalid_column() {
        let geo = make_geo();
        let mut board = Board::<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>::new(
            STANDARD_COLS,
            STANDARD_ROWS,
        );

        assert_eq!(board.drop_piece(10, Player::Red, &geo), None);
        assert_eq!(board.column_height(10, &geo), 0);
        assert!(board.is_column_full(10, &geo));
    }

    #[test]
    fn test_board_sizes() {
        let size_7x6 = std::mem::size_of::<Board<{ nw_for_board(7, 6) }>>();
        let size_9x9 = std::mem::size_of::<Board<{ nw_for_board(9, 9) }>>();

        // 7x6 (NW=1): 2*8 + 2 = 18 bytes + padding
        assert!(size_7x6 <= 24, "7x6 Board too large: {}", size_7x6);
        // 9x9 (NW=2): 2*16 + 2 = 34 bytes + padding
        assert!(size_9x9 <= 40, "9x9 Board too large: {}", size_9x9);
    }
}
