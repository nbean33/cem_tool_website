//! Tests of a model's own reliability, as 4nec2 runs them from its Generate (F7) and Optimizer
//! (F12) windows.
//!
//! * The **average-gain test** (AGT): with every loss removed, all the input power is radiated,
//!   so NEC's average power gain over the sphere must come out at 1 (2 over a perfect ground,
//!   where only the upper hemisphere is averaged). How far it misses measures how well the
//!   segmentation represents the structure; the shortfall in dB corrects the pattern's gains.
//! * The **convergence test**: refine every wire's segmentation step by step; a trustworthy model
//!   settles, an unreliable one keeps moving.

use crate::segmentation::{resegment, wires};
use crate::{EngineError, EvalOptions, Metrics, Model, NgfStore, Result};
use nec_cards::SymbolTable;
use nec_cards::edit::{cards, delete_line, replace_card};
use nec_core::Frequency;
use nec_solve::{GroundSpec, PatternRequest};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// The outcome of an average-gain test.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AverageGainTest {
    /// Average gain over its ideal: 1 for a perfect model.
    pub ratio: f64,
    /// The correction to apply to the model's gains, dB: `−10·log10(ratio)`.
    pub correction_db: f64,
    /// 4nec2's grading: "accurate" (0.95–1.05), "usable" (0.90–1.10), "improvable"
    /// (0.80–1.20) or "questionable".
    pub grade: &'static str,
    /// Whether the test ran over a perfect ground (the model has one).
    pub over_ground: bool,
    /// The frequency tested, MHz.
    pub mhz: f64,
}

/// 4nec2's grades for an average-gain ratio.
#[must_use]
pub fn agt_grade(ratio: f64) -> &'static str {
    let d = (ratio - 1.0).abs();
    if d < 0.05 {
        "accurate"
    } else if d < 0.10 {
        "usable"
    } else if d < 0.20 {
        "improvable"
    } else {
        "questionable"
    }
}

/// The deck with its losses removed for the test: every `LD` card goes (4nec2: "Wire/ground
/// losses are set to zero for AGT-test"), a lossy ground becomes perfect, and a second ground
/// medium (`GD`) goes with it.
fn lossless(text: &str) -> Result<String> {
    let deck_err = |e: nec_cards::CardError| EngineError::Deck(e.to_string());
    let all = cards(text, &["LD", "GN", "GD"]).map_err(deck_err)?;
    let mut out = text.to_string();
    // Bottom up, so earlier line numbers stay valid.
    for c in all.iter().rev() {
        match c.mnemonic.as_str() {
            "LD" | "GD" => out = delete_line(&out, c.line).ok_or_else(|| EngineError::Deck("line".into()))?,
            "GN" => {
                #[allow(clippy::cast_possible_truncation)]
                let kind = c.values.first().copied().flatten().unwrap_or(0.0).round() as i64;
                if kind != -1 && kind != 1 {
                    out = replace_card(&out, c.line, "GN", &["1".to_string()]).ok_or_else(|| EngineError::Deck("GN".into()))?;
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

/// Run the average-gain test on `text` at `mhz` (the deck's first frequency when `None`).
///
/// # Errors
/// The deck cannot be built or solved, or has no source.
pub fn average_gain_test(text: &str, mhz: Option<f64>, store: &NgfStore) -> Result<AverageGainTest> {
    let clean = lossless(text)?;
    let m = Model::with_context(&clean, &SymbolTable::new(), store)?;
    let prog = m.program()?;
    if prog.sources.is_empty() {
        return Err(EngineError::NoSource);
    }
    let f = match mhz {
        Some(v) => Frequency::from_mhz(v).map_err(|e| EngineError::Frequency(e.to_string()))?,
        None => *prog.frequencies.first().ok_or_else(|| EngineError::Frequency("the deck has no frequency".into()))?,
    };
    let sol = m.solve(f, prog.options(f))?;
    let over_ground = prog.ground != GroundSpec::FreeSpace;
    // 2° in θ and 4° in φ, averaged — the resolution the power-balance property test uses,
    // which integrates a lossless dipole to within 5e-3.
    let n_theta = if over_ground { 46 } else { 91 };
    let req = PatternRequest::from_fields(0, n_theta, 91, 1, [0.0, 0.0, 2.0, 4.0, 0.0, 0.0]);
    let avg = prog
        .pattern(&m.geom, &sol, &req)
        .average
        .ok_or_else(|| EngineError::Solve("no average gain".into()))?
        .0;
    let ratio = if over_ground { avg / 2.0 } else { avg };
    Ok(AverageGainTest {
        ratio,
        correction_db: -10.0 * ratio.log10(),
        grade: agt_grade(ratio),
        over_ground,
        mhz: f.mhz(),
    })
}

/// One step of a convergence test.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConvergenceStep {
    /// How many times the original segmentation.
    pub factor: f64,
    /// Segments of the antenna (dummy wires not counted).
    pub segments: usize,
    /// What was measured.
    pub metrics: Metrics,
}

/// Refine every wire's segmentation by `growth` (0.2–0.5 is 4nec2's range) per step, up to
/// `steps` steps or `max_segments`, and evaluate each at `mhz`. Steps arrive through `on_step`
/// as they finish.
///
/// # Errors
/// The deck cannot be built, or the first step fails.
#[allow(clippy::too_many_arguments)]
pub fn convergence_test<F>(
    text: &str,
    mhz: f64,
    steps: usize,
    growth: f64,
    max_segments: usize,
    options: &EvalOptions,
    cancel: &AtomicBool,
    on_step: F,
) -> Result<Vec<ConvergenceStep>>
where
    F: Fn(&ConvergenceStep),
{
    let base = wires(text)?;
    let mut out = Vec::new();
    for k in 0..steps.max(1) {
        if cancel.load(Ordering::Relaxed) {
            return Err(EngineError::Cancelled);
        }
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let factor = (1.0 + growth).powi(k as i32);
        // Dummy wires (a current source's, a matching network's) keep their one segment.
        let counts: HashMap<usize, usize> = base
            .iter()
            .filter(|(c, _)| c.values.first().copied().flatten().is_none_or(|t| t < f64::from(crate::model::AUXILIARY_TAG)))
            .filter_map(|(c, _)| {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let n = c.values.get(1).copied().flatten()?.abs() as usize;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
                let refined = (n as f64 * factor).ceil() as usize;
                Some((c.line, refined.max(n)))
            })
            .collect();
        let (refined, _) = resegment(text, &counts)?;
        let model = Model::new(&refined)?;
        let segments = model.geom.segments.iter().filter(|s| s.tag < crate::model::AUXILIARY_TAG).count();
        if k > 0 && segments > max_segments {
            break;
        }
        let metrics = match model.evaluate(mhz, options) {
            Ok(m) => m,
            Err(e) if k == 0 => return Err(e),
            Err(_) => break,
        };
        let step = ConvergenceStep { factor, segments, metrics };
        on_step(&step);
        out.push(step);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Direction;

    const DIPOLE: &str = "CE\nGW 1 21 0 -.24 0 0 .24 0 .001\nGE 0\nEX 0 1 11 0 1 0\nFR 0 1 0 0 299.8 0\nEN\n";

    #[test]
    fn a_good_dipole_passes_the_average_gain_test() {
        let t = average_gain_test(DIPOLE, None, &NgfStore::default()).expect("runs");
        assert!((t.ratio - 1.0).abs() < 5e-3, "{t:?}");
        assert_eq!(t.grade, "accurate");
        assert!(!t.over_ground);
    }

    #[test]
    fn losses_and_lossy_ground_are_taken_out() {
        let lossy = "CE\nGW 1 11 0 0 0 0 0 .24 .001\nGE 1\nGN 2 0 0 0 13 .005\nLD 5 1 0 0 3.5e7\nEX 0 1 1 0 1\nFR 0 1 0 0 299.8 0\nEN\n";
        let clean = lossless(lossy).expect("cleans");
        assert!(!clean.contains("LD"), "{clean}");
        assert!(clean.contains("GN 1"), "{clean}");
        let t = average_gain_test(lossy, None, &NgfStore::default()).expect("runs");
        assert!(t.over_ground);
        assert!((t.ratio - 1.0).abs() < 5e-3, "{t:?}");
    }

    #[test]
    fn a_coarse_model_is_graded_down() {
        // Three segments on a half-wave dipole: far too coarse.
        let coarse = DIPOLE.replace("GW 1 21", "GW 1 3").replace("EX 0 1 11", "EX 0 1 2");
        let t = average_gain_test(&coarse, None, &NgfStore::default()).expect("runs");
        assert!((t.ratio - 1.0).abs() > 5e-3, "{t:?}");
    }

    #[test]
    fn a_dipole_converges() {
        let options = EvalOptions { z0: 50.0, pattern: Some(Direction { azimuth: 0.0, elevation: Some(0.0) }), ..EvalOptions::default() };
        let steps = convergence_test(DIPOLE, 299.8, 4, 0.5, 8000, &options, &AtomicBool::new(false), |_| {})
            .expect("runs");
        assert_eq!(steps.len(), 4);
        assert!(steps.windows(2).all(|w| w[1].segments > w[0].segments));
        // The impedance settles: the last change is smaller than the first.
        let dz = |a: &ConvergenceStep, b: &ConvergenceStep| (a.metrics.z[0] - b.metrics.z[0]).hypot(a.metrics.z[1] - b.metrics.z[1]);
        assert!(dz(&steps[2], &steps[3]) < dz(&steps[0], &steps[1]));
    }

    #[test]
    fn a_matched_feed_keeps_its_dummy_wire_whole() {
        use crate::matching::{Quality, apply, design_l};
        let m = Model::new(DIPOLE).expect("builds").evaluate(299.8, &EvalOptions::default()).expect("solves");
        let z = num_complex::Complex64::new(m.z[0], m.z[1]);
        let net = design_l(z, 50.0, 299.8, Quality { q_l: 0.0, q_c: 0.0 }).remove(0);
        let matched = apply(DIPOLE, 1, 11, &net, 299.8).expect("applies");
        let options = EvalOptions { z0: 50.0, pattern: None, ..EvalOptions::default() };
        let steps = convergence_test(&matched, 299.8, 2, 0.5, 8000, &options, &AtomicBool::new(false), |_| {}).expect("runs");
        assert_eq!(steps[0].segments, 21);
        assert_eq!(steps[1].segments, 32);
        // The source still sees the match, refinement only nudging the antenna.
        for s in &steps {
            assert!((s.metrics.z[0] - 50.0).abs() < 3.0 && s.metrics.z[1].abs() < 3.0, "{:?}", s.metrics.z);
        }
    }
}
