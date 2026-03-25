use super::*;

#[pyclass(name = "Board")]
#[derive(Clone)]
pub struct PyBoard {
    pub(in crate::python) inner: BoardInner,
}

impl PyBoard {
    pub(in crate::python) fn from_inner(inner: BoardInner) -> Self {
        PyBoard { inner }
    }
}

#[pymethods]
impl PyBoard {
    #[new]
    pub fn new(width: usize, height: usize) -> PyResult<Self> {
        let (width, height) = validate_board_dimensions(width, height)?;
        Ok(PyBoard {
            inner: make_board_inner(width, height),
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
            b.is_column_full(col as u8)
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
