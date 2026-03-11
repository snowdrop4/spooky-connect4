use super::*;

#[pyclass(name = "GameOutcome")]
#[derive(Clone, Copy, Debug)]
pub struct PyGameOutcome {
    outcome: GameOutcome,
}

impl PyGameOutcome {
    pub(in crate::python) fn from_outcome(outcome: GameOutcome) -> Self {
        PyGameOutcome { outcome }
    }
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
