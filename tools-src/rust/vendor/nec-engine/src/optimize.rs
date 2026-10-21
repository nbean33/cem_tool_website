//! Tuning a deck's `SY` variables against goals.

use crate::goal::FAILED;
use crate::optim::{self, Bounds, Method, Settings};
use crate::{Direction, EngineError, EvalOptions, Goal, Metrics, Model, Result, cost};
use nec_cards::SymbolTable;
use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};

/// A variable to tune, and its range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    /// The `SY` variable.
    pub name: String,
    /// Smallest allowed value.
    pub min: f64,
    /// Largest allowed value.
    pub max: f64,
}

/// An optimization problem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizeSpec {
    /// The variables to tune; the deck's current values are the starting point.
    pub parameters: Vec<Parameter>,
    /// Frequencies at which the goals are judged, MHz.
    pub frequencies_mhz: Vec<f64>,
    /// What to achieve.
    pub goals: Vec<Goal>,
    /// Reference impedance for SWR, ohms.
    pub z0: f64,
    /// Forward, for gain and front-to-back goals.
    pub direction: Direction,
    /// Which minimizer.
    pub method: Method,
    /// Evaluation budget; each evaluation solves every frequency.
    pub max_evaluations: usize,
    /// Seed for differential evolution.
    pub seed: u64,
    /// Auto-segmentation and stepped-radius correction.
    #[serde(default)]
    pub model: crate::ModelOptions,
}

/// The best design so far, as a search reports it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OptimizeProgress {
    /// Evaluations so far.
    pub evaluations: usize,
    /// Iterations (simplex steps or generations) so far.
    pub iteration: usize,
    /// Cost of the best design.
    pub cost: f64,
    /// Its variable values, in the order of [`OptimizeSpec::parameters`].
    pub values: Vec<f64>,
    /// Its metrics at each frequency, when they could be computed.
    pub metrics: Vec<Metrics>,
}

/// How an optimization ended.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OptimizeOutcome {
    /// The design as it started.
    pub initial: OptimizeProgress,
    /// The best design found.
    pub best: OptimizeProgress,
    /// Whether the search converged.
    pub converged: bool,
    /// Whether it was cancelled.
    pub cancelled: bool,
}

/// Metrics at each frequency for `values` of the spec's parameters.
fn evaluate(
    text: &str,
    spec: &OptimizeSpec,
    options: &EvalOptions,
    values: &[f64],
) -> Result<Vec<Metrics>> {
    let mut pins = SymbolTable::new();
    for (p, v) in spec.parameters.iter().zip(values) {
        pins.set(&p.name, *v);
    }
    let model = Model::with_options(text, &pins, &crate::NgfStore::default(), &spec.model)?;
    spec.frequencies_mhz
        .iter()
        .map(|&f| model.evaluate(f, options))
        .collect()
}

/// Tune the deck `text` to `spec`, reporting each improvement through `on_progress`. Setting
/// `cancel` stops the search and returns the best design so far.
pub fn optimize<P>(
    text: &str,
    spec: &OptimizeSpec,
    cancel: &AtomicBool,
    mut on_progress: P,
) -> Result<OptimizeOutcome>
where
    P: FnMut(&OptimizeProgress),
{
    if spec.parameters.is_empty() {
        return Err(EngineError::Program(
            "choose at least one variable to tune".into(),
        ));
    }
    if spec.frequencies_mhz.is_empty() || spec.goals.is_empty() {
        return Err(EngineError::Program(
            "an optimization needs a frequency and a goal".into(),
        ));
    }
    let base = Model::new(text)?;
    base.program()?;
    let x0 = spec
        .parameters
        .iter()
        .map(|p| {
            base.variables
                .iter()
                .find(|v| v.name.eq_ignore_ascii_case(&p.name))
                .map(|v| v.value)
                .ok_or_else(|| EngineError::UnknownVariable(p.name.clone()))
        })
        .collect::<Result<Vec<f64>>>()?;
    let options = EvalOptions {
        z0: spec.z0,
        pattern: spec
            .goals
            .iter()
            .any(|g| g.kind.needs_pattern())
            .then_some(spec.direction),
        model: spec.model,
        capture: crate::Capture::None,
    };
    let snapshot = |evaluations, iteration, values: &[f64]| -> Result<OptimizeProgress> {
        let metrics = evaluate(text, spec, &options, values)?;
        Ok(OptimizeProgress {
            evaluations,
            iteration,
            cost: cost(&spec.goals, &metrics),
            values: values.to_vec(),
            metrics,
        })
    };
    let initial = snapshot(0, 0, &x0)?;

    let objective = |x: &[f64]| {
        if cancel.load(Ordering::Relaxed) {
            return FAILED;
        }
        evaluate(text, spec, &options, x).map_or(FAILED, |m| cost(&spec.goals, &m))
    };
    let bounds = Bounds {
        lo: spec.parameters.iter().map(|p| p.min.min(p.max)).collect(),
        hi: spec.parameters.iter().map(|p| p.max.max(p.min)).collect(),
    };
    let settings = Settings {
        method: spec.method,
        max_evaluations: spec.max_evaluations,
        seed: spec.seed,
    };
    let mut best = initial.clone();
    let outcome = optim::minimize(objective, &x0, &bounds, &settings, |p| {
        if cancel.load(Ordering::Relaxed) {
            return ControlFlow::Break(());
        }
        if p.best_cost < best.cost
            && let Ok(s) = snapshot(p.evaluations, p.iteration, p.best)
        {
            best = s;
        }
        best.evaluations = p.evaluations;
        best.iteration = p.iteration;
        on_progress(&best);
        ControlFlow::Continue(())
    });
    // The search may end on an improvement no iteration reported.
    if outcome.cost < best.cost
        && let Ok(s) = snapshot(outcome.evaluations, best.iteration, &outcome.x)
    {
        best = s;
    }
    best.evaluations = outcome.evaluations;
    Ok(OptimizeOutcome {
        initial,
        best,
        converged: outcome.converged,
        cancelled: outcome.stopped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GoalKind;
    use crate::testdata::DIPOLE;

    fn spec(method: Method) -> OptimizeSpec {
        OptimizeSpec {
            parameters: vec![Parameter {
                name: "len".into(),
                min: 0.2,
                max: 0.28,
            }],
            frequencies_mhz: vec![300.0],
            goals: vec![Goal {
                kind: GoalKind::Reactance,
                target: None,
                weight: 1.0,
            }],
            z0: 50.0,
            direction: Direction::default(),
            method,
            max_evaluations: 600,
            seed: 1,
            model: crate::ModelOptions::default(),
        }
    }

    /// Tuning a too-long dipole back to resonance, which this one reaches at a half-length of
    /// 0.2418 m (0.4836 λ overall).
    #[test]
    fn tunes_a_dipole_to_resonance() {
        let long = DIPOLE.replace("len=.2418", "len=.26");
        for method in [Method::NelderMead, Method::DifferentialEvolution] {
            let mut reports = 0;
            let out = optimize(&long, &spec(method), &AtomicBool::new(false), |_| {
                reports += 1
            })
            .expect("runs");
            assert!(reports > 0);
            assert!(out.initial.metrics[0].z[1] > 10.0, "{:?}", out.initial);
            let x = out.best.metrics[0].z[1];
            assert!(
                x.abs() < 0.05,
                "{method:?}: X = {x} at len {:?}",
                out.best.values
            );
            let len = out.best.values[0];
            assert!((len - 0.2418).abs() < 2e-4, "{len}");
        }
    }

    #[test]
    fn unknown_variables_are_refused() {
        let mut s = spec(Method::NelderMead);
        s.parameters[0].name = "nope".into();
        assert_eq!(
            optimize(DIPOLE, &s, &AtomicBool::new(false), |_| {}),
            Err(EngineError::UnknownVariable("nope".into()))
        );
    }
}
