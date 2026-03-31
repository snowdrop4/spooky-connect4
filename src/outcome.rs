use crate::player::Player;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameOutcome {
    RedWin,
    YellowWin,
    Draw,
}

impl GameOutcome {
    pub fn winner(&self) -> Option<Player> {
        match self {
            GameOutcome::RedWin => Some(Player::Red),
            GameOutcome::YellowWin => Some(Player::Yellow),
            GameOutcome::Draw => None,
        }
    }

    pub fn encode_winner_absolute(&self) -> f32 {
        match self {
            GameOutcome::RedWin => 1.0,
            GameOutcome::YellowWin => -1.0,
            GameOutcome::Draw => 0.0,
        }
    }

    pub fn encode_winner_from_perspective(&self, perspective: Player) -> f32 {
        match perspective {
            Player::Red => match self {
                GameOutcome::RedWin => 1.0,
                GameOutcome::YellowWin => -1.0,
                GameOutcome::Draw => 0.0,
            },
            Player::Yellow => match self {
                GameOutcome::RedWin => -1.0,
                GameOutcome::YellowWin => 1.0,
                GameOutcome::Draw => 0.0,
            },
        }
    }

    pub fn is_draw(&self) -> bool {
        matches!(self, GameOutcome::Draw)
    }
}

impl std::fmt::Display for GameOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameOutcome::RedWin => write!(f, "Red wins"),
            GameOutcome::YellowWin => write!(f, "Yellow wins"),
            GameOutcome::Draw => write!(f, "Draw"),
        }
    }
}
