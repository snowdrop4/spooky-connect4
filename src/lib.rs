pub mod bitboard;
pub mod board;
pub mod encode;
pub mod game;
mod limits;
pub mod r#move;
pub mod outcome;
pub mod player;
pub mod position;

#[cfg(feature = "python")]
extern crate pyo3;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
#[pymodule(gil_used = false)]
fn spooky_connect4(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use player::Player;
    use python::*;
    m.add_class::<PyBoard>()?;
    m.add_class::<PyGame>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<PyGameOutcome>()?;
    m.add_function(wrap_pyfunction!(augment_symmetries, m)?)?;
    m.add("RED", Player::Red as i8)?;
    m.add("YELLOW", Player::Yellow as i8)?;
    m.add("TOTAL_INPUT_PLANES", encode::TOTAL_INPUT_PLANES)?;
    Ok(())
}
