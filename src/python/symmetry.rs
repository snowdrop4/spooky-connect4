use numpy::ndarray::{s, Array1, Array2, Array4};
use numpy::{
    IntoPyArray, PyArray1, PyArray2, PyArray4, PyReadonlyArray1, PyReadonlyArray2, PyReadonlyArray4,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn validate_batch_shapes(
    states_shape: &[usize],
    policies_shape: &[usize],
    values_shape: &[usize],
    opponent_policies_shape: &[usize],
    opponent_policy_masks_shape: &[usize],
) -> PyResult<(usize, usize, usize, usize)> {
    if states_shape.len() != 4 {
        return Err(PyValueError::new_err(
            "states must have shape [batch, planes, height, width]",
        ));
    }
    if policies_shape.len() != 2 {
        return Err(PyValueError::new_err(
            "policies must have shape [batch, actions]",
        ));
    }
    if values_shape.len() != 1 {
        return Err(PyValueError::new_err("values must have shape [batch]"));
    }
    if opponent_policies_shape.len() != 2 {
        return Err(PyValueError::new_err(
            "opponent_policies must have shape [batch, actions]",
        ));
    }
    if opponent_policy_masks_shape.len() != 1 {
        return Err(PyValueError::new_err(
            "opponent_policy_masks must have shape [batch]",
        ));
    }

    let sample_count = states_shape[0];
    let plane_count = states_shape[1];
    let height = states_shape[2];
    let width = states_shape[3];

    if policies_shape[0] != sample_count
        || values_shape[0] != sample_count
        || opponent_policies_shape[0] != sample_count
        || opponent_policy_masks_shape[0] != sample_count
    {
        return Err(PyValueError::new_err(
            "all inputs must have the same batch dimension",
        ));
    }

    if policies_shape[1] != width || opponent_policies_shape[1] != width {
        return Err(PyValueError::new_err(
            "connect4 policies must have one action per column",
        ));
    }

    Ok((sample_count, plane_count, height, width))
}

#[pyfunction]
pub fn augment_symmetries<'py>(
    py: Python<'py>,
    states: PyReadonlyArray4<'py, f32>,
    policies: PyReadonlyArray2<'py, f32>,
    values: PyReadonlyArray1<'py, f32>,
    opponent_policies: PyReadonlyArray2<'py, f32>,
    opponent_policy_masks: PyReadonlyArray1<'py, f32>,
) -> PyResult<(
    Bound<'py, PyArray4<f32>>,
    Bound<'py, PyArray2<f32>>,
    Bound<'py, PyArray1<f32>>,
    Bound<'py, PyArray2<f32>>,
    Bound<'py, PyArray1<f32>>,
)> {
    let states = states.as_array();
    let policies = policies.as_array();
    let values = values.as_array();
    let opponent_policies = opponent_policies.as_array();
    let opponent_policy_masks = opponent_policy_masks.as_array();

    let (sample_count, plane_count, height, width) = validate_batch_shapes(
        states.shape(),
        policies.shape(),
        values.shape(),
        opponent_policies.shape(),
        opponent_policy_masks.shape(),
    )?;

    let augmented_sample_count = sample_count * 2;

    let mut augmented_states =
        Array4::<f32>::zeros((augmented_sample_count, plane_count, height, width));
    augmented_states
        .slice_mut(s![0..sample_count, .., .., ..])
        .assign(&states);

    let mut augmented_policies = Array2::<f32>::zeros((augmented_sample_count, width));
    augmented_policies
        .slice_mut(s![0..sample_count, ..])
        .assign(&policies);

    let mut augmented_values = Array1::<f32>::zeros(augmented_sample_count);
    augmented_values
        .slice_mut(s![0..sample_count])
        .assign(&values);
    augmented_values
        .slice_mut(s![sample_count..])
        .assign(&values);

    let mut augmented_opponent_policies = Array2::<f32>::zeros((augmented_sample_count, width));
    augmented_opponent_policies
        .slice_mut(s![0..sample_count, ..])
        .assign(&opponent_policies);

    let mut augmented_opponent_policy_masks = Array1::<f32>::zeros(augmented_sample_count);
    augmented_opponent_policy_masks
        .slice_mut(s![0..sample_count])
        .assign(&opponent_policy_masks);
    augmented_opponent_policy_masks
        .slice_mut(s![sample_count..])
        .assign(&opponent_policy_masks);

    for sample_idx in 0..sample_count {
        let mirrored_sample_idx = sample_count + sample_idx;
        for plane_idx in 0..plane_count {
            for row_idx in 0..height {
                for col_idx in 0..width {
                    let mirrored_col_idx = width - 1 - col_idx;
                    augmented_states[[mirrored_sample_idx, plane_idx, row_idx, mirrored_col_idx]] =
                        states[[sample_idx, plane_idx, row_idx, col_idx]];
                }
            }
        }

        for action_idx in 0..width {
            let mirrored_action_idx = width - 1 - action_idx;
            augmented_policies[[mirrored_sample_idx, mirrored_action_idx]] =
                policies[[sample_idx, action_idx]];
            augmented_opponent_policies[[mirrored_sample_idx, mirrored_action_idx]] =
                opponent_policies[[sample_idx, action_idx]];
        }
    }

    Ok((
        augmented_states.into_pyarray(py),
        augmented_policies.into_pyarray(py),
        augmented_values.into_pyarray(py),
        augmented_opponent_policies.into_pyarray(py),
        augmented_opponent_policy_masks.into_pyarray(py),
    ))
}
