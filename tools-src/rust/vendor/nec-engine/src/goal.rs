//! What an optimization is for, as a single number to minimize.

use crate::Metrics;
use serde::{Deserialize, Serialize};

/// The quantity a [`Goal`] constrains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalKind {
    /// SWR at most the target (default 1).
    Swr,
    /// Forward gain at least the target, dBi; with no target, as high as possible.
    Gain,
    /// Front-to-back at least the target, dB; with no target, as high as possible.
    FrontToBack,
    /// Front-to-rear (forward against the strongest of the rear half) at least the target, dB;
    /// with no target, as high as possible.
    FrontToRear,
    /// Input resistance equal to the target (default 50 Ω).
    Resistance,
    /// Input reactance equal to the target (default 0 Ω).
    Reactance,
    /// Efficiency at least the target (default 100 %).
    Efficiency,
}

impl GoalKind {
    /// Whether this goal needs the radiation pattern.
    #[must_use]
    pub const fn needs_pattern(self) -> bool {
        matches!(self, Self::Gain | Self::FrontToBack | Self::FrontToRear)
    }
}

/// One weighted objective.
///
/// Each goal's shortfall is measured in units chosen to be roughly comparable — one SWR unit,
/// one dB, ten ohms, ten percent — so equal weights are a reasonable start. Over several
/// frequencies a goal counts its worst one: "SWR below 1.5 across the band" is a statement
/// about the band edge, not the average.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    /// What is constrained.
    pub kind: GoalKind,
    /// The target, in the quantity's own units; `None` takes the kind's default.
    pub target: Option<f64>,
    /// Relative importance.
    pub weight: f64,
}

/// A large finite cost for an evaluation that failed, so a simplex or population can still
/// rank it.
pub(crate) const FAILED: f64 = 1.0e6;

/// Front-to-back beyond this earns nothing more: deeper nulls are fragile (they move with
/// frequency and construction tolerance) and chasing them trades away everything else.
const FRONT_TO_BACK_CAP_DB: f64 = 40.0;

impl Goal {
    /// The shortfall at one frequency; zero when met.
    #[must_use]
    pub fn penalty(&self, m: &Metrics) -> f64 {
        let shortfall_below = |value: Option<f64>, cap: f64| match (value, self.target) {
            (None, _) => FAILED,
            (Some(v), Some(t)) => (t - v.min(cap)).max(0.0),
            (Some(v), None) => -v.min(cap),
        };
        match self.kind {
            GoalKind::Swr => (m.swr.min(100.0) - self.target.unwrap_or(1.0)).max(0.0),
            GoalKind::Gain => shortfall_below(m.gain, f64::INFINITY),
            GoalKind::FrontToBack => shortfall_below(m.front_to_back, FRONT_TO_BACK_CAP_DB),
            GoalKind::FrontToRear => shortfall_below(m.front_to_rear, FRONT_TO_BACK_CAP_DB),
            GoalKind::Resistance => (m.z[0] - self.target.unwrap_or(50.0)).abs() / 10.0,
            GoalKind::Reactance => (m.z[1] - self.target.unwrap_or(0.0)).abs() / 10.0,
            GoalKind::Efficiency => (self.target.unwrap_or(100.0) - m.efficiency).max(0.0) / 10.0,
        }
    }
}

/// The weighted sum over goals of each goal's worst penalty across `metrics` (one entry per
/// frequency).
#[must_use]
pub fn cost(goals: &[Goal], metrics: &[Metrics]) -> f64 {
    goals
        .iter()
        .map(|g| {
            let worst = metrics
                .iter()
                .map(|m| g.penalty(m))
                .fold(f64::NEG_INFINITY, f64::max);
            g.weight * if worst.is_finite() { worst } else { FAILED }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(swr: f64, gain: f64) -> Metrics {
        Metrics {
            mhz: 14.0,
            z: [50.0, 0.0],
            swr,
            gain: Some(gain),
            front_to_back: Some(20.0),
            front_to_rear: Some(15.0),
            elevation: Some(0.0),
            efficiency: 100.0,
        }
    }

    #[test]
    fn worst_frequency_counts() {
        let g = [Goal {
            kind: GoalKind::Swr,
            target: Some(1.5),
            weight: 2.0,
        }];
        assert!((cost(&g, &[at(1.2, 0.0), at(2.0, 0.0)]) - 1.0).abs() < 1e-12);
        assert!(cost(&g, &[at(1.2, 0.0), at(1.4, 0.0)]).abs() < 1e-12);
    }

    #[test]
    fn untargeted_gain_is_maximized() {
        let g = [Goal {
            kind: GoalKind::Gain,
            target: None,
            weight: 1.0,
        }];
        assert!(cost(&g, &[at(1.0, 7.0)]) < cost(&g, &[at(1.0, 6.0)]));
        let met = [Goal {
            kind: GoalKind::Gain,
            target: Some(6.5),
            weight: 1.0,
        }];
        assert!(cost(&met, &[at(1.0, 7.0)]).abs() < 1e-12);
    }
}
