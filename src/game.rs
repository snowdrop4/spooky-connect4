use crate::board::{Board, STANDARD_COLS, STANDARD_ROWS};
use crate::outcome::GameOutcome;
use crate::player::Player;
use crate::position::Position;
use crate::r#move::Move;

#[derive(Debug)]
pub struct Game {
    board: Board,
    current_player: Player,
    move_history: Vec<Move>,
    is_over: bool,
    outcome: Option<GameOutcome>,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Game {
            board: Board::new(width, height),
            current_player: Player::Red,
            move_history: Vec::new(),
            is_over: false,
            outcome: None,
        }
    }

    pub fn standard() -> Self {
        Self::new(STANDARD_COLS, STANDARD_ROWS)
    }

    pub fn width(&self) -> usize {
        self.board.width()
    }

    pub fn height(&self) -> usize {
        self.board.height()
    }

    pub fn get_piece(&self, pos: &Position) -> Option<i8> {
        self.board.get_piece(pos).map(|p| p as i8)
    }

    pub fn set_piece(&mut self, pos: &Position, player: Option<Player>) {
        self.board.set_piece(pos, player)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn turn(&self) -> Player {
        self.current_player
    }

    pub fn is_over(&self) -> bool {
        self.is_over
    }

    pub fn outcome(&self) -> Option<GameOutcome> {
        self.outcome
    }

    pub fn move_history(&self) -> &[Move] {
        &self.move_history
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        if self.is_over {
            return Vec::new();
        }

        let mut moves = Vec::new();
        for col in 0..self.board.width() {
            if !self.board.is_column_full(col) {
                // Calculate where the piece would land
                let row = self.board.column_height(col);
                moves.push(Move::new(col, row));
            }
        }
        moves
    }

    pub fn is_legal_move(&self, move_: &Move) -> bool {
        if self.is_over {
            return false;
        }

        if move_.col >= self.board.width() {
            return false;
        }

        // Check if the column is not full and the row matches where the piece would land
        !self.board.is_column_full(move_.col) && move_.row == self.board.column_height(move_.col)
    }

    pub fn make_move(&mut self, move_: &Move) -> bool {
        if !self.is_legal_move(move_) {
            return false;
        }

        // Drop the piece
        if let Some(row) = self.board.drop_piece(move_.col, self.current_player) {
            let pos = Position::new(move_.col, row);
            self.move_history.push(*move_);

            // Check for win
            if self.board.check_win(&pos, self.current_player) {
                self.is_over = true;
                self.outcome = Some(match self.current_player {
                    Player::Red => GameOutcome::RedWin,
                    Player::Yellow => GameOutcome::YellowWin,
                });
            }
            // Check for draw
            else if self.board.is_board_full() {
                self.is_over = true;
                self.outcome = Some(GameOutcome::Draw);
            }

            // Switch player (always, even if game is over)
            self.current_player = self.current_player.opposite();
            true
        } else {
            false
        }
    }

    pub fn unmake_move(&mut self) -> bool {
        if let Some(last_move) = self.move_history.pop() {
            // Clear the piece at the position
            let pos = Position::new(last_move.col, last_move.row);
            self.board.set_piece(&pos, None);

            // Reset game state
            self.is_over = false;
            self.outcome = None;
            self.current_player = self.current_player.opposite();

            true
        } else {
            false
        }
    }
}

impl Clone for Game {
    fn clone(&self) -> Self {
        Game {
            board: self.board.clone(),
            current_player: self.current_player,
            move_history: self.move_history.clone(),
            is_over: self.is_over,
            outcome: self.outcome,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::standard()
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Game(turn: {}, is_over: {}, outcome: {:?})\n{}",
            self.current_player, self.is_over, self.outcome, self.board
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{STANDARD_COLS, STANDARD_ROWS};

    #[test]
    fn test_new_game() {
        let game = Game::standard();
        assert_eq!(game.turn(), Player::Red);
        assert!(!game.is_over());
        assert!(game.outcome().is_none());
    }

    #[test]
    fn test_legal_moves_initial() {
        let game = Game::standard();
        let moves = game.legal_moves();
        assert_eq!(moves.len(), STANDARD_COLS);
    }

    #[test]
    fn test_make_move() {
        let mut game = Game::standard();
        let move_ = Move::new(0, 0);

        assert!(game.is_legal_move(&move_));
        assert!(game.make_move(&move_));
        assert_eq!(game.turn(), Player::Yellow);
    }

    #[test]
    fn test_make_invalid_move() {
        let mut game = Game::standard();
        let move_ = Move::new(10, 0); // Invalid column

        assert!(!game.is_legal_move(&move_));
        assert!(!game.make_move(&move_));
    }

    #[test]
    fn test_unmake_move() {
        let mut game = Game::standard();
        let move_ = Move::new(0, 0);

        game.make_move(&move_);
        assert_eq!(game.turn(), Player::Yellow);

        assert!(game.unmake_move());
        assert_eq!(game.turn(), Player::Red);
        assert_eq!(game.move_history().len(), 0);
    }

    #[test]
    fn test_vertical_win() {
        let mut game = Game::standard();

        // Red plays column 0 four times
        for i in 0..3 {
            let red_move = Move::new(0, i);
            game.make_move(&red_move);

            let yellow_move = Move::new(1, i);
            game.make_move(&yellow_move);
        }

        // Red's fourth move wins
        let winning_move = Move::new(0, 3);
        game.make_move(&winning_move);

        assert!(game.is_over());
        assert_eq!(game.outcome(), Some(GameOutcome::RedWin));
    }

    #[test]
    fn test_horizontal_win() {
        let mut game = Game::standard();

        // Create horizontal win
        for col in 0..3 {
            let red_move = Move::new(col, 0);
            game.make_move(&red_move);

            let yellow_move = Move::new(col, 1);
            game.make_move(&yellow_move);
        }

        let winning_move = Move::new(3, 0);
        game.make_move(&winning_move);

        assert!(game.is_over());
        assert_eq!(game.outcome(), Some(GameOutcome::RedWin));
    }

    #[test]
    fn test_draw() {
        let mut game = Game::standard();

        // Fill columns in groups of 3, row-by-row within each group.
        // This creates alternating color patterns that avoid 4-in-a-row
        // in all directions (horizontal, vertical, diagonal).
        let pattern = vec![
            0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, // Cols 0-2
            3, 4, 5, 3, 4, 5, 3, 4, 5, 3, 4, 5, 3, 4, 5, 3, 4, 5, // Cols 3-5
            6, 6, 6, 6, 6, 6, // Col 6
        ];

        for &col in &pattern {
            assert!(!game.is_over(), "Game ended early before board was filled");
            let legal_moves = game.legal_moves();
            let move_ = legal_moves.iter().find(|m| m.col == col).cloned();
            let m = move_.expect("Expected column to be playable");
            game.make_move(&m);
        }

        assert!(game.board().is_board_full());
        assert!(game.is_over());
        assert_eq!(game.outcome(), Some(GameOutcome::Draw));
    }

    #[test]
    fn test_clone() {
        let mut game = Game::standard();
        let move_ = Move::new(0, 0);
        game.make_move(&move_);

        let cloned = game.clone();
        assert_eq!(cloned.turn(), game.turn());
        assert_eq!(cloned.is_over(), game.is_over());
        assert_eq!(cloned.move_history().len(), game.move_history().len());
    }

    #[test]
    fn test_move_history() {
        let mut game = Game::standard();

        assert_eq!(game.move_history().len(), 0);

        let move1 = Move::new(0, 0);
        game.make_move(&move1);
        assert_eq!(game.move_history().len(), 1);

        let move2 = Move::new(1, 0);
        game.make_move(&move2);
        assert_eq!(game.move_history().len(), 2);

        game.unmake_move();
        assert_eq!(game.move_history().len(), 1);
    }

    #[test]
    fn test_legal_moves_when_column_full() {
        let mut game = Game::standard();

        // Fill column 0
        for i in 0..STANDARD_ROWS {
            let move_ = Move::new(0, i);
            game.make_move(&move_);
        }

        let legal_moves = game.legal_moves();
        // Should have 6 legal moves (columns 1-6)
        assert_eq!(legal_moves.len(), STANDARD_COLS - 1);
        assert!(legal_moves.iter().all(|m| m.col != 0));
    }

    #[test]
    fn test_legal_moves_when_game_over() {
        let mut game = Game::standard();

        // Create a win
        for i in 0..3 {
            game.make_move(&Move::new(0, i));
            game.make_move(&Move::new(1, i));
        }
        game.make_move(&Move::new(0, 3));

        assert!(game.is_over());
        assert_eq!(game.legal_moves().len(), 0);
    }

    #[test]
    fn test_is_legal_move_after_column_full() {
        let mut game = Game::standard();

        // Fill column 0
        for i in 0..STANDARD_ROWS {
            game.make_move(&Move::new(0, i));
        }

        // Trying to play in column 0 should be illegal
        let move_ = Move::new(0, 0);
        assert!(!game.is_legal_move(&move_));
    }

    #[test]
    fn test_multiple_unmakes() {
        let mut game = Game::standard();

        // Make 5 moves
        for i in 0..5 {
            game.make_move(&Move::new(i % STANDARD_COLS, 0));
        }

        assert_eq!(game.move_history().len(), 5);

        // Unmake all moves
        for _ in 0..5 {
            assert!(game.unmake_move());
        }

        assert_eq!(game.move_history().len(), 0);
        assert_eq!(game.turn(), Player::Red);
        assert!(!game.is_over());
    }

    #[test]
    fn test_unmake_when_empty() {
        let mut game = Game::standard();
        assert!(!game.unmake_move());
    }
}
