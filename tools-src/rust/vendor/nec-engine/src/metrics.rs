//! What is read off one solution.

use crate::{EngineError, Model, Result};
use nec_core::Frequency;
use nec_solve::{GroundSpec, PatternPoint, PatternRequest};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// Where "forward" is, for gain and front-to-back.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Direction {
    /// Azimuth φ of the main beam, degrees (0 = +x, counter-clockwise).
    pub azimuth: f64,
    /// Elevation above the horizon, degrees; `None` searches the vertical plane at
    /// [`Direction::azimuth`] for the strongest elevation, which is what "gain" means over
    /// ground, where the takeoff angle moves with height and frequency.
    pub elevation: Option<f64>,
}

impl Default for Direction {
    fn default() -> Self {
        Self {
            azimuth: 0.0,
            elevation: None,
        }
    }
}

/// What to compute besides impedance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EvalOptions {
    /// Reference impedance for SWR, ohms.
    pub z0: f64,
    /// Compute gain and front-to-back toward this direction.
    pub pattern: Option<Direction>,
    /// How decks are built into models where the evaluation builds them (parameter sweeps).
    #[serde(default)]
    pub model: crate::ModelOptions,
    /// What to keep of each solution for play-back ([`Model::evaluate_with_frame`]).
    #[serde(default)]
    pub capture: Capture,
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            z0: 50.0,
            pattern: None,
            model: crate::ModelOptions::default(),
            capture: Capture::None,
        }
    }
}

/// What an evaluation keeps besides its metrics, for playing a sweep back.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capture {
    /// Nothing.
    #[default]
    None,
    /// The azimuth and elevation cuts through the forward direction.
    Patterns,
    /// The cuts and the current magnitude on every segment.
    PatternsAndCurrents,
}

/// One frequency of a sweep, kept for play-back.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    /// The azimuth cut at the forward elevation, and the elevation cut at the forward azimuth.
    pub azimuth: crate::pattern::Cut,
    /// See `azimuth`.
    pub elevation: crate::pattern::Cut,
    /// Current magnitude at each segment centre, amperes, when asked for.
    pub currents: Option<Vec<f32>>,
}

/// The figures of merit at one frequency, for the deck's first source.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    /// Frequency, MHz.
    pub mhz: f64,
    /// Input impedance, ohms, as `[R, X]`.
    pub z: [f64; 2],
    /// SWR against [`EvalOptions::z0`]; infinite for a purely reactive input.
    pub swr: f64,
    /// Power gain toward the forward direction, dBi.
    pub gain: Option<f64>,
    /// Gain forward minus gain in the back direction, dB: the opposite azimuth at the same
    /// elevation over ground, the opposite direction in free space (as 4nec2 takes it).
    pub front_to_back: Option<f64>,
    /// Gain forward minus the strongest gain in the rear half of the pattern, dB
    /// ([`crate::pattern::front_to_rear`], on a 5° grid).
    #[serde(default)]
    pub front_to_rear: Option<f64>,
    /// Elevation at which [`Metrics::gain`] was found, degrees.
    pub elevation: Option<f64>,
    /// Radiated over input power, percent (structure and network losses only, as NEC
    /// reports it; ground loss is not counted).
    pub efficiency: f64,
}

/// SWR of `z` against a real reference `z0`.
#[must_use]
pub fn swr(z: Complex64, z0: f64) -> f64 {
    let g = ((z - z0) / (z + z0)).norm();
    if g < 1.0 {
        (1.0 + g) / (1.0 - g)
    } else {
        f64::INFINITY
    }
}

fn request(theta0: f64, phi0: f64, n_theta: i32, dtheta: f64) -> PatternRequest {
    PatternRequest::from_fields(0, n_theta, 1, 0, [theta0, phi0, dtheta, 0.0, 0.0, 0.0])
}

impl Model {
    /// Solve at `mhz` and read off the metrics.
    pub fn evaluate(&self, mhz: f64, options: &EvalOptions) -> Result<Metrics> {
        self.evaluate_with_frame(mhz, &EvalOptions { capture: Capture::None, ..*options }).map(|(m, _)| m)
    }

    /// Solve at `mhz` and read off the metrics, and what [`EvalOptions::capture`] asks to keep.
    pub fn evaluate_with_frame(&self, mhz: f64, options: &EvalOptions) -> Result<(Metrics, Option<Frame>)> {
        if let Some(m) = self.for_frequency(mhz)? {
            return m.evaluate_with_frame(mhz, options);
        }
        let prog = self.program()?;
        let f = Frequency::from_mhz(mhz).map_err(|e| EngineError::Frequency(e.to_string()))?;
        if prog.sources.is_empty() {
            return Err(EngineError::NoSource);
        }
        let sol = self.solve(f, prog.options(f))?;
        let inputs = self.source_inputs(&sol);
        let input = inputs.first().ok_or(EngineError::NoSource)?;
        let mut m = Metrics {
            mhz,
            z: [input.impedance.re, input.impedance.im],
            swr: swr(input.impedance, options.z0),
            gain: None,
            front_to_back: None,
            front_to_rear: None,
            elevation: None,
            efficiency: sol.power.efficiency_percent(),
        };
        let mut frame = None;
        let capture = options.capture != Capture::None;
        if let Some(dir) = options.pattern.or_else(|| capture.then(Direction::default)) {
            let ground = prog.ground != GroundSpec::FreeSpace;
            let strongest = |points: &[PatternPoint]| {
                points
                    .iter()
                    .copied()
                    .max_by(|a, b| a.total.total_cmp(&b.total))
            };
            let forward = match dir.elevation {
                Some(el) => {
                    prog.pattern(&self.geom, &sol, &request(90.0 - el, dir.azimuth, 1, 0.0))
                }
                None => {
                    // 1° steps from the zenith down to the horizon (over ground) or the nadir.
                    let n = if prog.ground == GroundSpec::FreeSpace {
                        181
                    } else {
                        91
                    };
                    prog.pattern(&self.geom, &sol, &request(0.0, dir.azimuth, n, 1.0))
                }
            };
            if let Some(peak) = strongest(&forward.points) {
                if options.pattern.is_some() {
                    let (bt, bp) = crate::pattern::back_direction(peak.theta, dir.azimuth, ground);
                    let back = prog.pattern(&self.geom, &sol, &request(bt, bp, 1, 0.0));
                    // The rear half on a 5° grid: azimuths within 90° of the back, every θ.
                    let n_theta = if ground { 19 } else { 37 };
                    let rear = prog.pattern(
                        &self.geom,
                        &sol,
                        &crate::pattern::request(n_theta, 37, 0, 0.0, bp - 90.0, 5.0, 5.0),
                    );
                    m.gain = Some(peak.total);
                    m.elevation = Some(90.0 - peak.theta);
                    m.front_to_back = back.points.first().map(|b| peak.total - b.total);
                    m.front_to_rear =
                        crate::pattern::front_to_rear(peak.total, peak.theta, dir.azimuth, ground, &rear.points);
                }
                if capture {
                    let (azimuth, elevation) = crate::pattern::cuts(prog, &self.geom, &sol, peak.theta, dir.azimuth);
                    #[allow(clippy::cast_possible_truncation)]
                    let currents = (options.capture == Capture::PatternsAndCurrents)
                        .then(|| sol.currents.iter().map(|c| c.norm() as f32).collect());
                    frame = Some(Frame { azimuth, elevation, currents });
                }
            }
        }
        Ok((m, frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::DIPOLE;

    #[test]
    fn free_space_front_to_back_looks_the_opposite_way_and_frames_carry_the_cuts() {
        // A driven element and a reflector in free space, judged 30° above the horizon.
        let deck = "CE\nGW 1 21 0 -.24 0 0 .24 0 .001\nGW 2 21 -.15 -.26 0 -.15 .26 0 .001\nGE 0\nEX 0 1 11 0 1 0\nFR 0 1 0 0 299.8 0\nEN\n";
        let m = Model::new(deck).expect("prepares");
        let opts = EvalOptions {
            pattern: Some(Direction { azimuth: 0.0, elevation: Some(30.0) }),
            capture: Capture::PatternsAndCurrents,
            ..EvalOptions::default()
        };
        let (r, frame) = m.evaluate_with_frame(299.8, &opts).expect("solves");
        let prog = m.program().expect("program");
        let f = prog.frequencies[0];
        let sol = m.solve(f, prog.options(f)).expect("solves");
        let gain = |theta: f64, phi: f64| prog.pattern(&m.geom, &sol, &request(theta, phi, 1, 0.0)).points[0].total;
        // Back of (θ 60°, φ 0°) is (θ 120°, φ 180°): 30° below the horizon behind.
        let fb = gain(60.0, 0.0) - gain(120.0, 180.0);
        assert!((r.front_to_back.expect("f/b") - fb).abs() < 1e-9);
        let fr = r.front_to_rear.expect("f/r");
        assert!(fr > 0.0 && fr <= fb + 1e-9, "F/R {fr} against F/B {fb}");
        let frame = frame.expect("captured");
        assert_eq!(frame.azimuth.angle.len(), 361);
        assert_eq!(frame.elevation.angle.len(), 361);
        assert!((frame.azimuth.at - 30.0).abs() < 1e-9);
        assert_eq!(frame.currents.expect("currents").len(), 42);
        // Without capture, nothing is kept.
        assert!(m.evaluate_with_frame(299.8, &EvalOptions::default()).expect("solves").1.is_none());
    }

    #[test]
    fn a_dipole_s_metrics() {
        let m = Model::new(DIPOLE).expect("prepares");
        let opts = EvalOptions {
            z0: 50.0,
            pattern: Some(Direction {
                azimuth: 0.0,
                elevation: None,
            }),
            ..EvalOptions::default()
        };
        let r = m.evaluate(300.0, &opts).expect("solves");
        assert!((r.z[0] - 72.08).abs() < 0.01, "{:?}", r.z);
        assert!((r.swr - 1.44).abs() < 0.01);
        // The wire lies along y, so the whole x-z plane is broadside: 2.13 dBi at every
        // elevation of the scan, and symmetric front to back.
        assert!((r.gain.expect("gain") - 2.13).abs() < 0.02, "{r:?}");
        assert!(r.front_to_back.expect("f/b").abs() < 1e-6);
        assert!((r.efficiency - 100.0).abs() < 1e-9);
    }
}
