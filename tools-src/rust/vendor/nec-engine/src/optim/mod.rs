//! Box-constrained minimizers for expensive objectives, independent of antennas.
//!
//! Both work in the unit cube: each variable is mapped from its `[lo, hi]` range to `[0, 1]`,
//! so a step means the same fraction of every range however different their units (a boom
//! length in metres beside a wire radius in millimetres). Objectives must be `Sync`, because
//! independent evaluations — a simplex's initial vertices, a population's trial vectors — run
//! in parallel.
//!
//! * [`Method::NelderMead`] is the local workhorse: few evaluations, good for polishing a
//!   design that is already close.
//! * [`Method::DifferentialEvolution`] searches the whole box and so escapes the local minima
//!   antenna problems are full of, at many more evaluations; it finishes with a Nelder–Mead
//!   polish of its best point.

mod de;
mod nelder_mead;
mod rng;

use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;

pub use rng::Rng;

/// Which minimizer to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    /// Nelder–Mead simplex search from the starting point.
    NelderMead,
    /// Differential evolution over the whole box, then a Nelder–Mead polish.
    DifferentialEvolution,
}

/// How long and how hard to search.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Which minimizer.
    pub method: Method,
    /// Stop after this many objective evaluations.
    pub max_evaluations: usize,
    /// Seed for differential evolution, so a run can be repeated exactly.
    pub seed: u64,
}

/// The search space: a box.
#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    /// Lower bounds.
    pub lo: Vec<f64>,
    /// Upper bounds, each at least the matching lower bound.
    pub hi: Vec<f64>,
}

impl Bounds {
    /// The point at unit-cube coordinates `u`.
    #[must_use]
    pub fn to_x(&self, u: &[f64]) -> Vec<f64> {
        u.iter()
            .zip(self.lo.iter().zip(&self.hi))
            .map(|(&u, (&lo, &hi))| lo + u.clamp(0.0, 1.0) * (hi - lo))
            .collect()
    }

    /// The unit-cube coordinates of `x`, clamped into the box.
    #[must_use]
    pub fn to_u(&self, x: &[f64]) -> Vec<f64> {
        x.iter()
            .zip(self.lo.iter().zip(&self.hi))
            .map(|(&x, (&lo, &hi))| {
                if hi > lo {
                    ((x - lo) / (hi - lo)).clamp(0.0, 1.0)
                } else {
                    0.5
                }
            })
            .collect()
    }
}

/// Where a search stands, reported once per iteration (simplex step or generation).
#[derive(Debug, Clone, PartialEq)]
pub struct Progress<'a> {
    /// Objective evaluations so far.
    pub evaluations: usize,
    /// Iterations so far.
    pub iteration: usize,
    /// The best point so far.
    pub best: &'a [f64],
    /// Its cost.
    pub best_cost: f64,
}

/// How a search ended.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// The best point found.
    pub x: Vec<f64>,
    /// Its cost.
    pub cost: f64,
    /// Objective evaluations used.
    pub evaluations: usize,
    /// Whether the search converged, rather than running out of evaluations or being stopped.
    pub converged: bool,
    /// Whether the progress callback stopped it.
    pub stopped: bool,
}

/// Minimize `f` over `bounds`, starting from `x0`.
///
/// `progress` sees every iteration and may stop the search by returning
/// [`ControlFlow::Break`]; the best point so far is still returned.
pub fn minimize<F, P>(
    f: F,
    x0: &[f64],
    bounds: &Bounds,
    settings: &Settings,
    mut progress: P,
) -> Outcome
where
    F: Fn(&[f64]) -> f64 + Sync,
    P: FnMut(&Progress<'_>) -> ControlFlow<()>,
{
    let g = |u: &[f64]| f(&bounds.to_x(u));
    let u0 = bounds.to_u(x0);
    let mut report = |evaluations: usize, iteration: usize, u: &[f64], cost: f64| {
        progress(&Progress {
            evaluations,
            iteration,
            best: &bounds.to_x(u),
            best_cost: cost,
        })
    };
    let budget = settings.max_evaluations.max(2);
    let (u, cost, evaluations, converged, stopped) = match settings.method {
        Method::NelderMead => nelder_mead::run(&g, &u0, 0.1, budget, 0, &mut report),
        Method::DifferentialEvolution => {
            let mut rng = Rng::new(settings.seed);
            let global = de::run(&g, &u0, budget, &mut rng, &mut report);
            let left = budget.saturating_sub(global.2);
            if global.4 || left < 2 * u0.len() + 2 {
                global
            } else {
                // Polish locally with what is left, in a simplex sized to the population's
                // final spread.
                let polish = nelder_mead::run(&g, &global.0, 0.02, left, global.2, &mut report);
                if polish.1 < global.1 {
                    polish
                } else {
                    (global.0, global.1, polish.2, polish.3, polish.4)
                }
            }
        }
    };
    Outcome {
        x: bounds.to_x(&u),
        cost,
        evaluations,
        converged,
        stopped,
    }
}

/// The internal result: unit-cube point, cost, evaluations, converged, stopped.
type Run = (Vec<f64>, f64, usize, bool, bool);

/// The progress callback in unit-cube terms: evaluations, iteration, point, cost.
type Report<'a> = dyn FnMut(usize, usize, &[f64], f64) -> ControlFlow<()> + 'a;

#[allow(clippy::cast_precision_loss)]
fn as_f64(n: usize) -> f64 {
    n as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rosenbrock(x: &[f64]) -> f64 {
        (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2)
    }

    /// Many local minima on a grid; the global one is 0 at the origin.
    fn rastrigin(x: &[f64]) -> f64 {
        10.0 * as_f64(x.len())
            + x.iter()
                .map(|v| v * v - 10.0 * (std::f64::consts::TAU * v).cos())
                .sum::<f64>()
    }

    fn settings(method: Method, max_evaluations: usize) -> Settings {
        Settings {
            method,
            max_evaluations,
            seed: 7,
        }
    }

    #[test]
    fn nelder_mead_finds_the_rosenbrock_valley_floor() {
        let b = Bounds {
            lo: vec![-2.0, -1.0],
            hi: vec![2.0, 3.0],
        };
        let out = minimize(
            rosenbrock,
            &[-1.2, 1.0],
            &b,
            &settings(Method::NelderMead, 2000),
            |_| ControlFlow::Continue(()),
        );
        assert!(out.converged, "{out:?}");
        assert!(
            (out.x[0] - 1.0).abs() < 1e-3 && (out.x[1] - 1.0).abs() < 2e-3,
            "{out:?}"
        );
    }

    #[test]
    fn nelder_mead_respects_the_box() {
        // The unconstrained minimum (3, 3) is outside; the constrained one is the corner.
        let f = |x: &[f64]| (x[0] - 3.0).powi(2) + (x[1] - 3.0).powi(2);
        let b = Bounds {
            lo: vec![0.0, 0.0],
            hi: vec![1.0, 2.0],
        };
        let out = minimize(
            f,
            &[0.5, 0.5],
            &b,
            &settings(Method::NelderMead, 1000),
            |_| ControlFlow::Continue(()),
        );
        assert!(
            (out.x[0] - 1.0).abs() < 1e-6 && (out.x[1] - 2.0).abs() < 1e-6,
            "{out:?}"
        );
    }

    #[test]
    fn differential_evolution_escapes_local_minima() {
        let b = Bounds {
            lo: vec![-5.12; 3],
            hi: vec![5.12; 3],
        };
        // Started in a local minimum where Nelder–Mead stays.
        let start = [2.0, -3.0, 1.0];
        let nm = minimize(
            rastrigin,
            &start,
            &b,
            &settings(Method::NelderMead, 3000),
            |_| ControlFlow::Continue(()),
        );
        assert!(nm.cost > 1.0, "{nm:?}");
        let de = minimize(
            rastrigin,
            &start,
            &b,
            &settings(Method::DifferentialEvolution, 6000),
            |_| ControlFlow::Continue(()),
        );
        assert!(de.cost < 1e-6, "{de:?}");
    }

    #[test]
    fn a_callback_can_stop_the_search() {
        let b = Bounds {
            lo: vec![-2.0, -1.0],
            hi: vec![2.0, 3.0],
        };
        let mut calls = 0;
        let out = minimize(
            rosenbrock,
            &[-1.2, 1.0],
            &b,
            &settings(Method::DifferentialEvolution, 5000),
            |p| {
                calls += 1;
                assert!(p.best_cost.is_finite());
                if calls == 3 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        );
        assert!(out.stopped && !out.converged);
        assert_eq!(calls, 3);
    }
}
