use crate::bitboard::BoardGeometry;
use crate::board::Board;
use crate::outcome::GameOutcome;
use crate::player::Player;
use crate::position::Position;
use crate::r#move::Move;

#[derive(Debug)]
pub struct Game<const NW: usize> {
    board: Board<NW>,
    geo: BoardGeometry<NW>,
    current_player: Player,
    move_history: Vec<Move>,
    is_over: bool,
    outcome: Option<GameOutcome>,
}

impl<const NW: usize> Game<NW> {
    pub fn new(width: u8, height: u8) -> Self {
        Game {
            board: Board::new(width, height),
            geo: BoardGeometry::new(width, height),
            current_player: Player::Red,
            move_history: Vec::new(),
            is_over: false,
            outcome: None,
        }
    }

    pub fn width(&self) -> u8 {
        self.board.width()
    }

    pub fn height(&self) -> u8 {
        self.board.height()
    }

    pub fn get_piece(&self, pos: &Position) -> Option<i8> {
        self.board.get_piece(pos).map(|p| p as i8)
    }

    pub fn set_piece(&mut self, pos: &Position, player: Option<Player>) {
        self.board.set_piece(pos, player)
    }

    pub fn board(&self) -> &Board<NW> {
        &self.board
    }

    pub fn geo(&self) -> &BoardGeometry<NW> {
        &self.geo
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
            if !self.board.is_column_full(col, &self.geo) {
                let row = self.board.column_height(col, &self.geo);
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

        !self.board.is_column_full(move_.col, &self.geo)
            && move_.row == self.board.column_height(move_.col, &self.geo)
    }

    pub fn make_move(&mut self, move_: &Move) -> bool {
        if !self.is_legal_move(move_) {
            return false;
        }

        if let Some(row) = self
            .board
            .drop_piece(move_.col, self.current_player, &self.geo)
        {
            self.move_history.push(Move::new(move_.col, row));

            // Check for win
            if self.board.check_win(self.current_player, &self.geo) {
                self.is_over = true;
                self.outcome = Some(match self.current_player {
                    Player::Red => GameOutcome::RedWin,
                    Player::Yellow => GameOutcome::YellowWin,
                });
            }
            // Check for draw
            else if self.board.is_board_full(&self.geo) {
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
            let pos = Position::new(last_move.col, last_move.row);
            self.board.set_piece(&pos, None);

            self.is_over = false;
            self.outcome = None;
            self.current_player = self.current_player.opposite();

            true
        } else {
            false
        }
    }
}

impl<const NW: usize> Clone for Game<NW> {
    fn clone(&self) -> Self {
        Game {
            board: self.board,
            geo: self.geo,
            current_player: self.current_player,
            move_history: self.move_history.clone(),
            is_over: self.is_over,
            outcome: self.outcome,
        }
    }
}

impl<const NW: usize> std::fmt::Display for Game<NW> {
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
    use crate::bitboard::nw_for_board;
    use crate::board::{STANDARD_COLS, STANDARD_ROWS};

    type StdGame = Game<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>;

    fn standard_game() -> StdGame {
        StdGame::new(STANDARD_COLS, STANDARD_ROWS)
    }

    #[test]
    fn test_new_game() {
        let game = standard_game();
        assert_eq!(game.turn(), Player::Red);
        assert!(!game.is_over());
        assert!(game.outcome().is_none());
    }

    #[test]
    fn test_legal_moves_initial() {
        let game = standard_game();
        let moves = game.legal_moves();
        assert_eq!(moves.len(), STANDARD_COLS as usize);
    }

    #[test]
    fn test_make_move() {
        let mut game = standard_game();
        let move_ = Move::new(0, 0);

        assert!(game.is_legal_move(&move_));
        assert!(game.make_move(&move_));
        assert_eq!(game.turn(), Player::Yellow);
    }

    #[test]
    fn test_make_invalid_move() {
        let mut game = standard_game();
        let move_ = Move::new(10, 0); // Invalid column

        assert!(!game.is_legal_move(&move_));
        assert!(!game.make_move(&move_));
    }

    #[test]
    fn test_unmake_move() {
        let mut game = standard_game();
        let move_ = Move::new(0, 0);

        game.make_move(&move_);
        assert_eq!(game.turn(), Player::Yellow);

        assert!(game.unmake_move());
        assert_eq!(game.turn(), Player::Red);
        assert_eq!(game.move_history().len(), 0);
    }

    #[test]
    fn test_vertical_win() {
        let mut game = standard_game();

        for i in 0..3u8 {
            let red_move = Move::new(0, i);
            game.make_move(&red_move);

            let yellow_move = Move::new(1, i);
            game.make_move(&yellow_move);
        }

        let winning_move = Move::new(0, 3);
        game.make_move(&winning_move);

        assert!(game.is_over());
        assert_eq!(game.outcome(), Some(GameOutcome::RedWin));
    }

    #[test]
    fn test_horizontal_win() {
        let mut game = standard_game();

        for col in 0..3u8 {
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
        let mut game = standard_game();

        let pattern: Vec<u8> = vec![
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

        assert!(game.board().is_board_full(&game.geo));
        assert!(game.is_over());
        assert_eq!(game.outcome(), Some(GameOutcome::Draw));
    }

    #[test]
    fn test_clone() {
        let mut game = standard_game();
        let move_ = Move::new(0, 0);
        game.make_move(&move_);

        let cloned = game.clone();
        assert_eq!(cloned.turn(), game.turn());
        assert_eq!(cloned.is_over(), game.is_over());
        assert_eq!(cloned.move_history().len(), game.move_history().len());
    }

    #[test]
    fn test_move_history() {
        let mut game = standard_game();

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
        let mut game = standard_game();

        for i in 0..STANDARD_ROWS {
            let move_ = Move::new(0, i);
            game.make_move(&move_);
        }

        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), STANDARD_COLS as usize - 1);
        assert!(legal_moves.iter().all(|m| m.col != 0));
    }

    #[test]
    fn test_legal_moves_when_game_over() {
        let mut game = standard_game();

        for i in 0..3u8 {
            game.make_move(&Move::new(0, i));
            game.make_move(&Move::new(1, i));
        }
        game.make_move(&Move::new(0, 3));

        assert!(game.is_over());
        assert_eq!(game.legal_moves().len(), 0);
    }

    #[test]
    fn test_is_legal_move_after_column_full() {
        let mut game = standard_game();

        for i in 0..STANDARD_ROWS {
            game.make_move(&Move::new(0, i));
        }

        let move_ = Move::new(0, 0);
        assert!(!game.is_legal_move(&move_));
    }

    #[test]
    fn test_multiple_unmakes() {
        let mut game = standard_game();

        for i in 0..5u8 {
            let col = i % STANDARD_COLS;
            let legal = game.legal_moves();
            let m = legal.iter().find(|m| m.col == col).unwrap();
            game.make_move(m);
        }

        assert_eq!(game.move_history().len(), 5);

        for _ in 0..5 {
            assert!(game.unmake_move());
        }

        assert_eq!(game.move_history().len(), 0);
        assert_eq!(game.turn(), Player::Red);
        assert!(!game.is_over());
    }

    #[test]
    fn test_unmake_when_empty() {
        let mut game = standard_game();
        assert!(!game.unmake_move());
    }
}
