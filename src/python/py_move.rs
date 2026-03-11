use super::*;

#[pyclass(name = "Move")]
#[derive(Clone, Debug)]
pub struct PyMove {
    move_: Move,
}

impl PyMove {
    pub(in crate::python) fn from_move(move_: Move) -> Self {
        PyMove { move_ }
    }

    pub(in crate::python) fn as_move(&self) -> Move {
        self.move_
    }
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
