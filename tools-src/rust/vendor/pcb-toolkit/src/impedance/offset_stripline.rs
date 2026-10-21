//! Offset (asymmetric) single-ended stripline — a buried trace that is not centred.
//!
//! Reference: the SLINE2 worksheet shipped with Johnson & Graham, *High-Speed Digital
//! Design*, which composes Cohn's centred solution rather than deriving a new one:
//!
//! ```text
//!   Za = Zo_centered(b = 2·h_below + t)     // the trace mirrored about the lower plane
//!   Zb = Zo_centered(b = 2·h_above + t)     // and about the upper plane
//!   Zo = 2·Za·Zb / (Za + Zb)                // the two half-lines in parallel
//! ```
//!
//! The physical argument: each plane sees a trace at its own distance, and the two
//! capacitances add. Setting `h_below == h_above` collapses it to the centred case exactly,
//! which is the first test below.
//!
//! An asymmetric inner layer is extremely common — any time a core and a prepreg differ in
//! thickness — and this was the most-requested gap in `STATUS.md`'s "deliberately not
//! implemented" list. It is cheap only because [`super::stripline`] now implements Cohn's
//! *two* branches: a typical offset cross-section puts one half in the wide-trace regime and
//! the other in the skinny one, so a single-branch centred model could not compose correctly.
//!
//! # Range of validity
//!
//! The worksheet says plainly that this is an approximation with no published accuracy
//! bound, and that honesty is worth carrying forward — [`check_accuracy`] always returns at
//! least that caveat. The composition is exact in the symmetric limit and degrades as the
//! offset grows.

use serde::{Deserialize, Serialize};

use super::common;
use super::stripline::zo_for_spacing;
use super::types::ImpedanceResult;
use crate::CalcError;
use crate::all_finite;

/// Inputs for offset stripline impedance calculation. All dimensions in mils.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OffsetStriplineInput {
    /// Conductor width (mils).
    pub width: f64,
    /// Dielectric height from the trace down to the lower ground plane (mils).
    pub height_below: f64,
    /// Dielectric height from the trace up to the upper ground plane (mils).
    pub height_above: f64,
    /// Conductor thickness (mils).
    pub thickness: f64,
    /// Substrate relative permittivity.
    pub er: f64,
}

impl OffsetStriplineInput {
    /// Total plane-to-plane spacing, `b = h_below + h_above + t` (mils).
    pub fn separation(&self) -> f64 {
        self.height_below + self.height_above + self.thickness
    }
}

/// Warnings about the cross-section.
///
/// Always carries the standing caveat that this form has no published accuracy bound, plus
/// Cohn's `t/w < 0.11` band when the trace is thick for its width.
pub fn check_accuracy(input: &OffsetStriplineInput) -> Vec<String> {
    let mut problems =
        vec!["offset stripline is an approximation with no published accuracy bound".to_string()];
    let t_w = input.thickness / input.width;
    if t_w > 0.11 {
        problems.push(format!("t/w = {t_w:.3} is above 0.11"));
    }
    problems
}

/// Compute offset stripline characteristic impedance and derived quantities.
pub fn calculate(input: &OffsetStriplineInput) -> Result<ImpedanceResult, CalcError> {
    let OffsetStriplineInput {
        width,
        height_below,
        height_above,
        thickness,
        er,
    } = *input;

    all_finite(&[
        ("width", width),
        ("height_below", height_below),
        ("height_above", height_above),
        ("thickness", thickness),
        ("er", er),
    ])?;

    if width <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "width",
            value: width,
        });
    }
    if height_below <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "height_below",
            value: height_below,
        });
    }
    if height_above <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "height_above",
            value: height_above,
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

    let za = zo_for_spacing(2.0 * height_below + thickness, width, thickness, er);
    let zb = zo_for_spacing(2.0 * height_above + thickness, width, thickness, er);
    let zo = 2.0 * za * zb / (za + zb);

    // Homogeneous dielectric, same as the centred case: the wave sees only Er.
    let mut result = common::finish(zo, er, er)?;
    result.accuracy_warnings = check_accuracy(input);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::super::stripline::{self, StriplineInput};
    use super::*;

    /// The worked example stored in the SLINE2 worksheet: h1 = 7 mil, h2 = 32 mil,
    /// w = 8 mil, t = 1.5 mil, Er = 4.5 → **51.726315927621933 Ω**.
    ///
    /// This vector is the reason the centred model had to be fixed first. Za is computed at
    /// b = 15.5 mil where w/b = 0.52, which is Cohn's **wide**-trace regime; Zb at
    /// b = 64.5 mil where w/b = 0.12, which is the **skinny** regime. A single-branch
    /// centred model gets 50.39 here — 2.6% low — and the error is invisible on a symmetric
    /// cross-section, so this is the test that pins both branches at once.
    #[test]
    fn hsdd_sline2_worksheet_vector() {
        let r = calculate(&OffsetStriplineInput {
            width: 8.0,
            height_below: 7.0,
            height_above: 32.0,
            thickness: 1.5,
            er: 4.5,
        })
        .unwrap();
        assert_relative_eq!(r.zo, 51.726_315_927_621_93, max_relative = 1e-9);
        assert_relative_eq!(r.er_eff, 4.5, epsilon = 1e-12);
    }

    /// Equal heights must reproduce [`stripline::calculate`] bit for bit.
    ///
    /// With `Za == Zb` the parallel form `2ZaZb/(Za+Zb)` collapses to `Za`, so this is an
    /// identity rather than an approximation — and it is the check that the `b = 2h + t`
    /// conversion is the same on both sides. Getting that factor of two wrong in one place
    /// is the single most likely way to break this module.
    #[test]
    fn symmetric_case_equals_centered_stripline() {
        for (w, h, t, er) in [
            (6.0, 10.0, 1.4, 4.6),
            (20.0, 10.0, 1.4, 4.6), // wide branch
            (4.0, 25.0, 0.7, 3.0),  // skinny branch
        ] {
            let offset = calculate(&OffsetStriplineInput {
                width: w,
                height_below: h,
                height_above: h,
                thickness: t,
                er,
            })
            .unwrap();
            let centered = stripline::calculate(&StriplineInput {
                width: w,
                height: h,
                thickness: t,
                er,
            })
            .unwrap();
            assert_relative_eq!(offset.zo, centered.zo, max_relative = 1e-12);
            assert_relative_eq!(
                offset.tpd_ps_per_in,
                centered.tpd_ps_per_in,
                epsilon = 1e-12
            );
        }
    }

    /// Swapping the two heights must not change the answer — there is no "up".
    #[test]
    fn is_symmetric_under_swapping_the_planes() {
        let a = calculate(&OffsetStriplineInput {
            width: 8.0,
            height_below: 5.0,
            height_above: 30.0,
            thickness: 1.4,
            er: 4.3,
        })
        .unwrap();
        let b = calculate(&OffsetStriplineInput {
            width: 8.0,
            height_below: 30.0,
            height_above: 5.0,
            thickness: 1.4,
            er: 4.3,
        })
        .unwrap();
        assert_relative_eq!(a.zo, b.zo, max_relative = 1e-12);
    }

    /// Moving a trace off centre at fixed plane-to-plane spacing must *lower* Zo: the nearer
    /// plane dominates the capacitance, and capacitance only goes up.
    #[test]
    fn offsetting_at_fixed_separation_lowers_impedance() {
        let b_total = 40.0;
        let t = 1.4;
        let mut last = f64::INFINITY;
        // h_below shrinks from centred (19.3) toward the lower plane.
        for h_below in [19.3, 15.0, 10.0, 6.0, 3.0, 1.5] {
            let r = calculate(&OffsetStriplineInput {
                width: 8.0,
                height_below: h_below,
                height_above: b_total - t - h_below,
                thickness: t,
                er: 4.3,
            })
            .unwrap();
            assert!(r.zo < last, "zo rose at h_below = {h_below}: {}", r.zo);
            last = r.zo;
        }
    }

    /// The published separation must be the sum of the parts, which is what a stackup gives
    /// you and the one place an off-by-a-thickness would hide.
    #[test]
    fn separation_is_the_sum_of_the_parts() {
        let i = OffsetStriplineInput {
            width: 8.0,
            height_below: 7.0,
            height_above: 32.0,
            thickness: 1.5,
            er: 4.5,
        };
        assert_relative_eq!(i.separation(), 40.5, epsilon = 1e-12);
    }

    #[test]
    fn accuracy_always_carries_the_caveat() {
        let problems = check_accuracy(&OffsetStriplineInput {
            width: 20.0,
            height_below: 10.0,
            height_above: 10.0,
            thickness: 1.4,
            er: 4.6,
        });
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("no published accuracy bound"));
    }

    /// Every offset result must carry the caveat, since this form has no published bound at
    /// any geometry — there is no "inside the band" case to fall back on.
    #[test]
    fn every_result_carries_the_accuracy_caveat() {
        let r = calculate(&OffsetStriplineInput {
            width: 8.0,
            height_below: 7.0,
            height_above: 32.0,
            thickness: 1.5,
            er: 4.5,
        })
        .unwrap();
        assert!(
            r.accuracy_warnings
                .iter()
                .any(|w| w.contains("no published accuracy bound")),
            "{:?}",
            r.accuracy_warnings
        );
    }

    #[test]
    fn rejects_bad_geometry() {
        let base = OffsetStriplineInput {
            width: 8.0,
            height_below: 7.0,
            height_above: 32.0,
            thickness: 1.5,
            er: 4.5,
        };
        assert!(
            calculate(&OffsetStriplineInput {
                width: -1.0,
                ..base
            })
            .is_err()
        );
        assert!(
            calculate(&OffsetStriplineInput {
                height_below: 0.0,
                ..base
            })
            .is_err()
        );
        assert!(
            calculate(&OffsetStriplineInput {
                height_above: -3.0,
                ..base
            })
            .is_err()
        );
        assert!(calculate(&OffsetStriplineInput { er: 0.9, ..base }).is_err());
    }
}
