//! Solving one model at many frequencies, or many variants of a model at one frequency.
//!
//! Points are independent, so they are solved in parallel and each is reported as it finishes
//! — in completion order, not sweep order; the callback gets the point's index. Setting the
//! cancellation flag makes the remaining points return [`EngineError::Cancelled`] without
//! solving.

use crate::{EngineError, EvalOptions, Frame, Metrics, Model, Result};
use nec_cards::SymbolTable;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// `n` evenly spaced values from `start` to `stop` inclusive (one value, `start`, if `n < 2`).
#[must_use]
pub fn linspace(start: f64, stop: f64, n: usize) -> Vec<f64> {
    if n < 2 {
        return vec![start];
    }
    #[allow(clippy::cast_precision_loss)]
    let step = (stop - start) / (n - 1) as f64;
    #[allow(clippy::cast_precision_loss)]
    (0..n).map(|k| start + step * k as f64).collect()
}

/// Solve `model` at each frequency in `mhz`.
pub fn frequencies<F>(
    model: &Model,
    mhz: &[f64],
    options: &EvalOptions,
    cancel: &AtomicBool,
    on_point: F,
) -> Vec<Result<Metrics>>
where
    F: Fn(usize, &Result<Metrics>) + Sync,
{
    mhz.par_iter()
        .enumerate()
        .map(|(i, &f)| {
            let r = if cancel.load(Ordering::Relaxed) {
                Err(EngineError::Cancelled)
            } else {
                model.evaluate(f, options)
            };
            on_point(i, &r);
            r
        })
        .collect()
}

/// Solve `model` at each frequency in `mhz`, also handing `on_point` what
/// [`EvalOptions::capture`] asks to keep of each (for play-back). Frames are not kept.
pub fn frequencies_with_frames<F>(
    model: &Model,
    mhz: &[f64],
    options: &EvalOptions,
    cancel: &AtomicBool,
    on_point: F,
) -> Vec<Result<Metrics>>
where
    F: Fn(usize, &Result<Metrics>, Option<&Frame>) + Sync,
{
    mhz.par_iter()
        .enumerate()
        .map(|(i, &f)| {
            let r = if cancel.load(Ordering::Relaxed) {
                Err(EngineError::Cancelled)
            } else {
                model.evaluate_with_frame(f, options)
            };
            let (metrics, frame) = match r {
                Ok((m, frame)) => (Ok(m), frame),
                Err(e) => (Err(e), None),
            };
            on_point(i, &metrics, frame.as_ref());
            metrics
        })
        .collect()
}

/// Solve the deck `text` at `mhz` once for each value of the `SY` variable `name`.
pub fn parameter<F>(
    text: &str,
    name: &str,
    values: &[f64],
    mhz: f64,
    options: &EvalOptions,
    cancel: &AtomicBool,
    on_point: F,
) -> Vec<Result<Metrics>>
where
    F: Fn(usize, &Result<Metrics>) + Sync,
{
    values
        .par_iter()
        .enumerate()
        .map(|(i, &v)| {
            let r = if cancel.load(Ordering::Relaxed) {
                Err(EngineError::Cancelled)
            } else {
                let mut pins = SymbolTable::new();
                pins.set(name, v);
                Model::with_options(text, &pins, &crate::NgfStore::default(), &crate::ModelOptions { mhz: Some(mhz), ..options.model })
                    .and_then(|m| m.evaluate(mhz, options))
            };
            on_point(i, &r);
            r
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::DIPOLE;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn frequency_sweep_reports_every_point() {
        let m = Model::new(DIPOLE).expect("prepares");
        let seen = AtomicUsize::new(0);
        let f = linspace(280.0, 320.0, 5);
        let out = frequencies(
            &m,
            &f,
            &EvalOptions::default(),
            &AtomicBool::new(false),
            |_, r| {
                assert!(r.is_ok());
                seen.fetch_add(1, Ordering::Relaxed);
            },
        );
        assert_eq!(seen.into_inner(), 5);
        // Reactance rises through resonance.
        let x: Vec<f64> = out
            .iter()
            .map(|r| r.as_ref().expect("solves").z[1])
            .collect();
        assert!(x.windows(2).all(|w| w[1] > w[0]), "{x:?}");
    }

    #[test]
    fn parameter_sweep_changes_the_structure() {
        let lens = [0.22, 0.24, 0.26];
        let out = parameter(
            DIPOLE,
            "LEN",
            &lens,
            300.0,
            &EvalOptions::default(),
            &AtomicBool::new(false),
            |_, _| {},
        );
        let x: Vec<f64> = out
            .iter()
            .map(|r| r.as_ref().expect("solves").z[1])
            .collect();
        // Longer is more inductive.
        assert!(x[0] < 0.0 && x[2] > 0.0, "{x:?}");
    }

    #[test]
    fn cancelled_points_are_not_solved() {
        let m = Model::new(DIPOLE).expect("prepares");
        let out = frequencies(
            &m,
            &[300.0, 310.0],
            &EvalOptions::default(),
            &AtomicBool::new(true),
            |_, _| {},
        );
        assert!(out.iter().all(|r| *r == Err(EngineError::Cancelled)));
    }
}
