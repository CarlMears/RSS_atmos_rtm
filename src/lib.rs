//! RTM computation
//!
//! NOTE: this module is intended for the interface between Rust and Python. The
//! real work happens in the other modules, and they do not use `pyo3`, it's
//! only used here.

pub(crate) mod error;
pub(crate) mod rtm;

use std::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};

use error::RtmError;
use log::{debug, info};
use ndarray::{Array2, ArrayView1, Axis};
use numpy::prelude::*;
use numpy::{PyArray2, PyReadonlyArray1, PyReadonlyArray2, ToPyArray};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;
use rtm::{RtmInputs, RtmParameters};

impl From<RtmError> for PyErr {
    fn from(e: RtmError) -> Self {
        match e {
            RtmError::InconsistentInputs => PyValueError::new_err(e.to_string()),
            RtmError::NoSurface => PyValueError::new_err(e.to_string()),
            RtmError::NotContiguous => PyValueError::new_err(e.to_string()),
            RtmError::Cancelled => PyValueError::new_err(e.to_string()),
        }
    }
}

/// Atmospheric parameters.
///
/// This is just a container of multiple numpy arrays, each dimensioned as
/// (`num_points`, `num_freq`).
#[pyclass]
struct AtmoParameters {
    tran: Array2<f32>,
    tb_up: Array2<f32>,
    tb_down: Array2<f32>,
}

/// Implement all the "getters" for the Python properties
#[pymethods]
impl AtmoParameters {
    #[getter]
    fn tran<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f32>> {
        self.tran.to_pyarray(py)
    }

    #[getter]
    fn tb_up<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f32>> {
        self.tb_up.to_pyarray(py)
    }

    #[getter]
    fn tb_down<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f32>> {
        self.tb_down.to_pyarray(py)
    }
}

impl AtmoParameters {
    fn new(num_points: usize, num_freq: usize) -> Self {
        Self {
            tran: Array2::zeros([num_points, num_freq]),
            tb_up: Array2::zeros([num_points, num_freq]),
            tb_down: Array2::zeros([num_points, num_freq]),
        }
    }
}

/// Compute the radiative transfer model for the atmosphere.
///
/// Most of the inputs are numpy arrays and are either 1d or 2d. The `pressure`
/// parameter is the pressure levels in hPa and has shape (`num_levels`, ). It
/// is treated as a constant (i.e., not a function of `num_points`).
///
/// `pressure`: pressure levels, in hPa
///
/// The following are input profiles and have shape (`num_points`,
/// `num_levels`):
///
/// `temperature`: physical temperature in K
///
/// `height`: geometric height above the geoid in m
///
/// `specific_humidity`: specific humidity in kg/kg
///
/// `liquid_content`: liquid water content (from clouds) in kg/kg
///
/// The following are surface parameters and have shape (`num_points`, ):
///
/// `surface_temperature`: 2 meter air temperature in K
///
/// `surface_height`: geopotential height at the surface in m
///
/// `surface_dewpoint`: 2 meter dewpoint in K
///
/// `surface_pressure`: surface pressure in hPa
///
/// The following are RTM parameters and have shape (`num_freq`, ):
///
/// `incidence_angle`: Earth incidence angle in degrees
///
/// `frequency`: microwave frequency in GHz
///
/// The returned atmospheric parameters are each dimensioned as (`num_points`,
/// `num_freq`).
///
/// The number of worker threads is controlled by `num_threads`. It must be a
/// positive integer, or `None` to automatically choose the number of threads.
#[pyfunction]
#[pyo3(signature = (pressure, temperature, height, specific_humidity, liquid_content, surface_temperature, surface_height, surface_dewpoint, surface_pressure, incidence_angle, frequency, num_threads))]
#[allow(clippy::too_many_arguments)]
fn compute_rtm(
    py: Python<'_>,
    pressure: PyReadonlyArray1<'_, f32>,
    temperature: PyReadonlyArray2<'_, f32>,
    height: PyReadonlyArray2<'_, f32>,
    specific_humidity: PyReadonlyArray2<'_, f32>,
    liquid_content: PyReadonlyArray2<'_, f32>,
    surface_temperature: PyReadonlyArray1<'_, f32>,
    surface_height: PyReadonlyArray1<'_, f32>,
    surface_dewpoint: PyReadonlyArray1<'_, f32>,
    surface_pressure: PyReadonlyArray1<'_, f32>,
    incidence_angle: PyReadonlyArray1<'_, f32>,
    frequency: PyReadonlyArray1<'_, f32>,
    num_threads: Option<usize>,
) -> PyResult<AtmoParameters> {
    let num_freq = frequency.len();
    let num_levels = pressure.len();
    let num_points = temperature.shape()[0];

    // Check shapes of all inputs
    {
        let two_dims = &[
            temperature.dims(),
            height.dims(),
            specific_humidity.dims(),
            liquid_content.dims(),
        ];
        let one_dim_points = &[
            surface_temperature.len(),
            surface_height.len(),
            surface_dewpoint.len(),
            surface_pressure.len(),
        ];
        let one_dim_freqs = &[incidence_angle.len(), frequency.len()];

        if two_dims.iter().any(|d| d != &[num_points, num_levels]) {
            return Err(RtmError::InconsistentInputs.into());
        }
        if one_dim_points.iter().any(|&d| d != num_points) {
            return Err(RtmError::InconsistentInputs.into());
        }
        if one_dim_freqs.iter().any(|&d| d != num_freq) {
            return Err(RtmError::InconsistentInputs.into());
        }
    }
    debug!("input shapes are consistent");

    let parameters = RtmParameters::new(frequency.as_slice()?, incidence_angle.as_slice()?)?;

    // Ensure everything is converted and contiguous
    let pressure = pressure.as_slice()?;
    let temperature = temperature.as_array();
    let height = height.as_array();
    let specific_humidity = specific_humidity.as_array();
    let liquid_content = liquid_content.as_array();
    let surface_temperature = surface_temperature.as_slice()?;
    let surface_height = surface_height.as_slice()?;
    let surface_dewpoint = surface_dewpoint.as_slice()?;
    let surface_pressure = surface_pressure.as_slice()?;

    let mut results = Vec::new();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads.unwrap_or(0))
        .build()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    // These atomics keep track of how many points have finished and whether
    // it's time to cancel the computation or not
    let num_completed = AtomicUsize::new(0);
    let cancelled = AtomicBool::new(false);

    info!(
        "Processing atmosphere RTM for {num_points} profiles and {num_freq} frequencies using {} threads",
        pool.current_num_threads()
    );

    pool.in_place_scope(|s| -> Result<(), PyErr> {
        s.spawn(|_| {
            (0..num_points)
                .into_par_iter()
                .map(|point| -> Result<_, RtmError> {
                    if cancelled.load(Ordering::Relaxed) {
                        return Err(RtmError::Cancelled);
                    }

                    let rtm_input = RtmInputs::new(
                        pressure,
                        surface_temperature[point],
                        temperature
                            .index_axis(Axis(0), point)
                            .as_slice()
                            .ok_or(RtmError::NotContiguous)?,
                        surface_height[point],
                        height
                            .index_axis(Axis(0), point)
                            .as_slice()
                            .ok_or(RtmError::NotContiguous)?,
                        surface_dewpoint[point],
                        specific_humidity
                            .index_axis(Axis(0), point)
                            .as_slice()
                            .ok_or(RtmError::NotContiguous)?,
                        liquid_content
                            .index_axis(Axis(0), point)
                            .as_slice()
                            .ok_or(RtmError::NotContiguous)?,
                        surface_pressure[point],
                    )?;

                    Ok(rtm_input.run(&parameters))
                })
                .inspect(|_| {
                    num_completed.fetch_add(1, Ordering::Relaxed);
                })
                .collect_into_vec(&mut results);
        });

        // The work is done in the thread pool, but back here in the main
        // thread, handle progress reporting and checking for early
        // cancellation
        while !cancelled.load(Ordering::Relaxed) {
            if let Err(e) = py.check_signals() {
                cancelled.store(true, Ordering::Relaxed);
                return Err(e);
            }

            let num_completed = num_completed.load(Ordering::Relaxed);
            let progress = num_completed as f32 / num_points as f32 * 100.;
            info!("Completed RTM for {num_completed}/{num_points} profiles ({progress:0.2}%)");

            // All finished without cancelling early
            if num_completed == num_points {
                break;
            }

            py.allow_threads(|| {
                std::thread::sleep(Duration::from_secs(5));
            });
        }

        Ok(())
    })?;

    // Copy the intermediate results to the output arrays
    debug!("copying RTM output");
    let mut output = AtmoParameters::new(num_points, num_freq);
    results
        .into_iter()
        .enumerate()
        .try_for_each(|(index, rtm_output)| -> Result<_, RtmError> {
            let rtm::RtmOutputs {
                tran,
                tb_up,
                tb_down,
            } = rtm_output?;

            let rhs = ArrayView1::from(tran.as_slice());
            output.tran.index_axis_mut(Axis(0), index).assign(&rhs);

            let rhs = ArrayView1::from(tb_up.as_slice());
            output.tb_up.index_axis_mut(Axis(0), index).assign(&rhs);

            let rhs = ArrayView1::from(tb_down.as_slice());
            output.tb_down.index_axis_mut(Axis(0), index).assign(&rhs);

            Ok(())
        })?;

    Ok(output)
}

/// A Python module implemented in Rust.
#[pymodule]
fn access_atmosphere(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_function(wrap_pyfunction!(compute_rtm, m)?)?;
    m.add_class::<AtmoParameters>()?;
    Ok(())
}
