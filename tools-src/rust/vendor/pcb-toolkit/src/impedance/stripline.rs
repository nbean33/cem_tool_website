//! Stripline impedance calculator (centered), Cohn's two-branch model.
//!
//! Reference: Seymour B. Cohn, "Problems in Strip Transmission Lines", *IRE Transactions on
//! Microwave Theory and Techniques* MTT-3 No. 2, March 1955; summarized in Harlan Howe,
//! *Stripline Circuit Design*, Artech House, 1974. The skinny-trace shape factor carries a
//! correction credited to Robert Canright.
//!
//! Cohn's model has **two branches**, selected on `w > 0.35·b`:
//!
//! ```text
//!   b  = 2·height + thickness                       (plane-to-plane spacing)
//!   K1 = (w/2)·(1 + t/(πw)·(1 + ln(4πw/t)) + 0.255·(t/w)²)
//!   K2 = (2/u)·ln(1/u + 1) − (1/u − 1)·ln(1/u² − 1),  u = 1 − t/b
//!
//!   skinny (w ≤ 0.35b):  Zo = (60/√Er)·ln(4b / (π·K1))
//!   wide   (w > 0.35b):  Zo = (94.15/√Er) / ( (w/b)/(1 − t/b) + K2/π )
//! ```
//!
//! `Er_eff = Er` — the trace is fully embedded in dielectric, so there is no air interface
//! and no dispersion. That is exact, not an approximation.
//!
//! # What this replaced, and why
//!
//! Until 2026-10-08 this module computed `Zo = (60/√Er)·ln(1.9·(2H + T)/(0.8W + T))`, the
//! single-branch IPC-2141 approximation, under a header that cited Cohn. The two are close
//! where the IPC form is valid and diverge sharply outside it:
//!
//! | w/b  | IPC-2141 vs Cohn |
//! |------|------------------|
//! | 0.12 | +0.45%           |
//! | 0.30 | −0.03%           |
//! | 0.40 | −1.7%            |
//! | 0.60 | −6.0%            |
//! | 1.00 | −19.4%           |
//!
//! 50 Ω stripline on a thin core routinely lands at w/b ≈ 0.3–0.5, which is where that
//! starts to matter, and the old form had no absolute test to notice — every assertion in
//! this module was a range or a monotonicity check. See `VALIDATION.md`.
//!
//! # Range of validity
//!
//! Better than 1.3% for `t/b < 0.25` and `t/w < 0.11`. Outside that the wide-trace branch
//! stays well-behaved (it has no logarithm of the geometry to drive negative), so the shared
//! post-condition in [`common::finish`] rarely fires now — but [`check_accuracy`] will tell
//! you when a cross-section has wandered out of the published band.

use serde::{Deserialize, Serialize};

use super::common;
use super::types::ImpedanceResult;
use crate::CalcError;
use crate::all_finite;

/// Inputs for centered stripline impedance calculation. All dimensions in mils.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StriplineInput {
    /// Conductor width (mils).
    pub width: f64,
    /// Dielectric height — distance from trace to each ground plane (mils).
    /// For centered stripline, the plane-to-plane spacing is `2 × height + thickness`.
    pub height: f64,
    /// Conductor thickness (mils).
    pub thickness: f64,
    /// Substrate relative permittivity.
    pub er: f64,
}

/// The `w > 0.35·b` threshold at which Cohn's wide-trace branch takes over.
const WIDE_TRACE_RATIO: f64 = 0.35;

/// Cohn's skinny-trace shape factor K1, in the Canright-corrected form.
///
/// The `t → 0` limit is `w/2`, taken explicitly: the `t·ln(1/t)` term tends to zero but
/// evaluates as `0 × ∞` in floating point, and `thickness = 0` is an accepted input here.
fn k1(w: f64, t: f64) -> f64 {
    if t <= 0.0 {
        return w / 2.0;
    }
    (w / 2.0)
        * (1.0
            + t / (std::f64::consts::PI * w) * (1.0 + (4.0 * std::f64::consts::PI * w / t).ln())
            + 0.255 * (t / w) * (t / w))
}

/// Cohn's wide-trace shape factor K2.
///
/// The `t → 0` limit is `2·ln 2`: the second term is `ε·ln(2ε) → 0`, but evaluates as
/// `0 × −∞` in floating point.
fn k2(b: f64, t: f64) -> f64 {
    if t <= 0.0 {
        return 2.0 * std::f64::consts::LN_2;
    }
    let u = 1.0 - t / b;
    2.0 / u * (1.0 / u + 1.0).ln() - (1.0 / u - 1.0) * (1.0 / (u * u) - 1.0).ln()
}

/// Cohn's centered-stripline impedance for a plane-to-plane spacing `b` (mils).
///
/// Shared with [`super::offset_stripline`], which forms the parallel combination of two of
/// these — one about each plane. Taking `b` rather than a per-plane height is what makes that
/// reuse possible, and is why the conversion `b = 2h + t` lives in [`calculate`] instead.
pub(crate) fn zo_for_spacing(b: f64, w: f64, t: f64, er: f64) -> f64 {
    if w > WIDE_TRACE_RATIO * b {
        94.15 / er.sqrt() / ((w / b) / (1.0 - t / b) + k2(b, t) / std::f64::consts::PI)
    } else {
        60.0 / er.sqrt() * (4.0 * b / (std::f64::consts::PI * k1(w, t))).ln()
    }
}

/// Warnings about a cross-section that has left Cohn's published accuracy band.
///
/// Empty means the geometry is inside `t/b < 0.25` and `t/w < 0.11`, where the model is
/// better than 1.3%. This reports rather than refuses: a number slightly outside the band is
/// still the best closed form available, and silently returning a worse answer is the thing
/// worth avoiding.
pub fn check_accuracy(input: &StriplineInput) -> Vec<String> {
    let b = 2.0 * input.height + input.thickness;
    let mut problems = Vec::new();
    let t_b = input.thickness / b;
    if t_b > 0.25 {
        problems.push(format!("t/b = {t_b:.3} is above 0.25"));
    }
    let t_w = input.thickness / input.width;
    if t_w > 0.11 {
        problems.push(format!("t/w = {t_w:.3} is above 0.11"));
    }
    problems
}

/// Compute centered stripline characteristic impedance and derived quantities.
pub fn calculate(input: &StriplineInput) -> Result<ImpedanceResult, CalcError> {
    let StriplineInput {
        width,
        height,
        thickness,
        er,
    } = *input;

    all_finite(&[
        ("width", width),
        ("height", height),
        ("thickness", thickness),
        ("er", er),
    ])?;

    if width <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "width",
            value: width,
        });
    }
    if height <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "height",
            value: height,
        });
    }
    if thickness < 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "thickness",
            value: thickness,
        });
    }
    if er < 1.0 {
        return Err(CalcError::OutOfRange {
            name: "er",
            value: er,
            expected: ">= 1.0",
        });
    }

    let b = 2.0 * height + thickness;
    if thickness >= b {
        return Err(CalcError::OutOfRange {
            name: "thickness",
            value: thickness,
            expected: "< plane-to-plane spacing (2*height + thickness)",
        });
    }

    let zo = zo_for_spacing(b, width, thickness, er);

    // Stripline is homogeneous: the wave sees only Er, so Er_eff == Er exactly.
    let mut result = common::finish(zo, er, er)?;
    result.accuracy_warnings = check_accuracy(input);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    /// The worked example stored in the SLINE Mathcad worksheet shipped with Johnson &
    /// Graham, *High-Speed Digital Design*: b = 20 mil, w = 6 mil, t = 1.37 mil, Er = 4.5.
    ///
    /// This is the module's **first absolute assertion**. Before the Cohn rewrite every test
    /// here was a range or a monotonicity check, which is why a single-branch approximation
    /// sat under a Cohn attribution for as long as it did.
    ///
    /// `height` is per-plane, so `b = 20` means `height = (20 − 1.37)/2 = 9.315`. The stored
    /// results are Zo = 51.437079595177309 Ω, and at Er = 1 an inductance of
    /// 9.2441824189360338 nH/in.
    #[test]
    fn hsdd_sline_worksheet_vector() {
        let r = calculate(&StriplineInput {
            width: 6.0,
            height: 9.315,
            thickness: 1.37,
            er: 4.5,
        })
        .unwrap();
        assert_relative_eq!(r.zo, 51.437_079_595_177_31, max_relative = 1e-9);
        assert_relative_eq!(r.er_eff, 4.5, epsilon = 1e-12);

        // Lo = Zo·Tpd is derived, so checking it against the worksheet's own L at Er = 1
        // also confirms our propagation delay. The worksheet rounds c to 84.72 ps/in where we
        // derive 84.7231, so the agreement is to ~4e-5 rather than exact.
        let air = calculate(&StriplineInput {
            width: 6.0,
            height: 9.315,
            thickness: 1.37,
            er: 1.0,
        })
        .unwrap();
        assert_relative_eq!(air.lo_nh_per_in, 9.244_182_418_936_033, max_relative = 1e-4);
    }

    /// Both branches must be exercised, and the switch must not leave a visible step.
    ///
    /// Cohn's two forms are independent approximations that happen to agree near the
    /// changeover; they are not constructed to be continuous there. Agreeing to better than
    /// 2% at the seam is the real check that the threshold is the published 0.35 and not
    /// something else.
    #[test]
    fn the_two_branches_meet_at_the_threshold() {
        let b = 20.0;
        let (w, t, er): (f64, f64, f64) = (0.35 * b, 1.4, 4.3);
        let skinny = 60.0 / er.sqrt() * (4.0 * b / (std::f64::consts::PI * k1(w, t))).ln();
        let wide = 94.15 / er.sqrt() / ((w / b) / (1.0 - t / b) + k2(b, t) / std::f64::consts::PI);
        assert_relative_eq!(skinny, wide, max_relative = 2e-2);
    }

    /// A wide trace must stay physical. The IPC-2141 form this replaced drove its logarithm's
    /// argument below 1 and produced a *negative* impedance here (VALIDATION.md H2); the
    /// wide-trace branch has no such failure mode, so the geometry now simply computes.
    #[test]
    fn wide_trace_is_physical_rather_than_rejected() {
        let r = calculate(&StriplineInput {
            width: 100.0,
            height: 10.0,
            thickness: 1.4,
            er: 4.6,
        })
        .unwrap();
        assert!(r.zo > 0.0 && r.zo < 20.0, "zo = {}", r.zo);
    }

    #[test]
    fn zero_thickness_limits_are_finite() {
        let r = calculate(&StriplineInput {
            width: 6.0,
            height: 10.0,
            thickness: 0.0,
            er: 4.6,
        })
        .unwrap();
        assert!(r.zo.is_finite() && r.zo > 0.0);
        // And the wide branch at t = 0, where K2's limit is 2·ln2.
        let wide = calculate(&StriplineInput {
            width: 30.0,
            height: 10.0,
            thickness: 0.0,
            er: 4.6,
        })
        .unwrap();
        assert!(wide.zo.is_finite() && wide.zo > 0.0);
        assert_relative_eq!(k2(20.0, 0.0), 1.386_294_361_119_890_6, epsilon = 1e-12);
    }

    #[test]
    fn er_eff_equals_er() {
        let r = calculate(&StriplineInput {
            width: 8.0,
            height: 10.0,
            thickness: 1.4,
            er: 4.6,
        })
        .unwrap();
        assert_relative_eq!(r.er_eff, 4.6, epsilon = 1e-12);
    }

    #[test]
    fn narrow_trace_higher_impedance() {
        let narrow = calculate(&StriplineInput {
            width: 4.0,
            height: 10.0,
            thickness: 1.4,
            er: 4.6,
        })
        .unwrap();
        let wide = calculate(&StriplineInput {
            width: 12.0,
            height: 10.0,
            thickness: 1.4,
            er: 4.6,
        })
        .unwrap();
        assert!(
            narrow.zo > wide.zo,
            "narrow {} should exceed wide {}",
            narrow.zo,
            wide.zo
        );
    }

    #[test]
    fn higher_er_lower_impedance() {
        let low = calculate(&StriplineInput {
            width: 8.0,
            height: 10.0,
            thickness: 1.4,
            er: 2.2,
        })
        .unwrap();
        let high = calculate(&StriplineInput {
            width: 8.0,
            height: 10.0,
            thickness: 1.4,
            er: 10.2,
        })
        .unwrap();
        assert!(low.zo > high.zo);
        // Zo scales as 1/√Er in both branches, exactly.
        assert_relative_eq!(
            low.zo / high.zo,
            (10.2f64 / 2.2).sqrt(),
            max_relative = 1e-12
        );
    }

    #[test]
    fn impedance_decreases_with_width_across_the_branch_switch() {
        let mut last = f64::INFINITY;
        for w in [2.0, 4.0, 6.0, 7.0, 7.1, 8.0, 12.0, 20.0, 40.0] {
            let r = calculate(&StriplineInput {
                width: w,
                height: 10.0,
                thickness: 1.4,
                er: 4.6,
            })
            .unwrap();
            assert!(r.zo < last, "zo rose at w = {w}: {} >= {last}", r.zo);
            last = r.zo;
        }
    }

    #[test]
    fn accuracy_band_is_reported_not_enforced() {
        let inside = StriplineInput {
            width: 20.0,
            height: 10.0,
            thickness: 1.4,
            er: 4.6,
        };
        assert!(check_accuracy(&inside).is_empty());

        // Thick copper on a thin core leaves both bands, and still computes.
        let outside = StriplineInput {
            width: 4.0,
            height: 3.0,
            thickness: 2.8,
            er: 4.6,
        };
        let problems = check_accuracy(&outside);
        assert_eq!(problems.len(), 2, "{problems:?}");

        // And they must travel *with the result*. `check_accuracy` previously existed with
        // no caller anywhere — tested, public, and unreachable from either front end — so a
        // user outside the published band was told nothing.
        let r = calculate(&outside).unwrap();
        assert_eq!(r.accuracy_warnings, problems);
        assert!(r.zo > 0.0);
        assert!(
            calculate(&inside).unwrap().accuracy_warnings.is_empty(),
            "a cross-section inside the band must carry no warnings"
        );
    }

    #[test]
    fn rejects_negative_width() {
        assert!(
            calculate(&StriplineInput {
                width: -5.0,
                height: 10.0,
                thickness: 1.4,
                er: 4.6,
            })
            .is_err()
        );
    }

    #[test]
    fn rejects_low_er() {
        assert!(
            calculate(&StriplineInput {
                width: 8.0,
                height: 10.0,
                thickness: 1.4,
                er: 0.5,
            })
            .is_err()
        );
    }

    #[test]
    fn rejects_thickness_filling_the_cavity() {
        // height > 0 and thickness >= b cannot both hold, so this is reachable only through
        // a zero-height input, which the height check catches first. Guard it anyway: the
        // wide branch divides by (1 − t/b).
        assert!(
            calculate(&StriplineInput {
                width: 8.0,
                height: 0.0,
                thickness: 1.4,
                er: 4.6,
            })
            .is_err()
        );
    }
}
