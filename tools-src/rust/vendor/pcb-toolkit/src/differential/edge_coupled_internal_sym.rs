//! Edge-coupled internal symmetric (centered stripline) differential pair impedance calculator.
//!
//! Computes odd-mode, even-mode, and differential impedance for a symmetric stripline
//! differential pair using the Cohn stripline Z0 formula with exponential coupling correction.

use super::types::{DifferentialResult, kb_terminated};
use crate::CalcError;
use crate::all_finite;

/// Inputs for edge-coupled internal symmetric (centered stripline) differential pair.
pub struct EdgeCoupledInternalSymInput {
    /// Conductor width (mils).
    pub width: f64,
    /// Gap between traces (mils).
    pub spacing: f64,
    /// Dielectric height — distance from trace to each ground plane (mils).
    /// For centered stripline, total dielectric thickness = 2 × height.
    pub height: f64,
    /// Conductor thickness (mils).
    pub thickness: f64,
    /// Substrate relative permittivity.
    pub er: f64,
}

/// Compute differential impedance for an edge-coupled internal symmetric (centered stripline) pair.
pub fn calculate(input: &EdgeCoupledInternalSymInput) -> Result<DifferentialResult, CalcError> {
    let EdgeCoupledInternalSymInput {
        width,
        spacing,
        height,
        thickness,
        er,
    } = *input;

    all_finite(&[
        ("width", width),
        ("spacing", spacing),
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
    if spacing <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "spacing",
            value: spacing,
        });
    }
    if height <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "height",
            value: height,
        });
    }
    if thickness <= 0.0 {
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

    // Plane-to-plane dielectric thickness. Saturn V8.47 takes this as a direct input
    // ("Enter the distance between the two planes") and feeds the *same* value to both
    // the Z0 logarithm and the coupling exponent; we derive it from the per-plane height
    // so the two stay consistent here too.
    let b = 2.0 * height + thickness;

    let z0 = (60.0 / er.sqrt()) * (1.9 * b / (0.8 * width + thickness)).ln();

    // Edge-coupled *stripline* coupling. This previously used the edge-coupled
    // *microstrip* pair (0.48 / −0.96) divided by the per-plane height, which ran
    // 20–32% low at the S/B ratios differential pairs are actually designed at.
    // Saturn V8.47 ships 0.374 here, a transposition of the published 0.347; we use
    // the literature value, which is ~1% high against the binary at S/B ≈ 0.2 and
    // converges above it. See docs/notes/18-v847-vs-rust-comparison.md §4.
    let zodd = z0 * (1.0 - 0.347 * (-2.9 * spacing / b).exp());
    let zeven = z0 * z0 / zodd;
    let zdiff = 2.0 * zodd;
    let kb = (zeven - zodd) / (zeven + zodd);
    let kb_db = 20.0 * kb.log10();
    let kb_term = kb_terminated(kb);
    let kb_term_db = 20.0 * kb_term.log10();

    Ok(DifferentialResult {
        zdiff,
        zo: z0,
        zodd,
        zeven,
        kb,
        kb_db,
        kb_term,
        kb_term_db,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn input(
        width: f64,
        spacing: f64,
        height: f64,
        thickness: f64,
        er: f64,
    ) -> EdgeCoupledInternalSymInput {
        EdgeCoupledInternalSymInput {
            width,
            spacing,
            height,
            thickness,
            er,
        }
    }

    /// Absolute assertion pinning the edge-coupled stripline model.
    ///
    /// W=10, S=63, H=63 (per plane, so B = 127.2), T=1.2, Er=4. Hand-derived:
    ///   B      = 2(63) + 1.2                      = 127.2
    ///   Z0     = (60/√4)·ln(1.9·127.2/9.2)        = 98.05233
    ///   S/B                                       = 0.4952830
    ///   Zodd   = Z0·(1 − 0.347·e^(−2.9·0.4952830)) = 89.96135
    ///   Zeven  = Z0²/Zodd                         = 106.87100
    ///
    /// This replaces a `140.0 <= zdiff <= 170.0` range check, which passed while the
    /// module applied the edge-coupled *microstrip* law over the per-plane height
    /// (Zdiff 160.05 via a Z0 25% high and a coupling term 18% low — two errors
    /// cancelling into a plausible-looking total). See VALIDATION.md on range checks.
    #[test]
    fn stripline_reference_point() {
        let r = calculate(&input(10.0, 63.0, 63.0, 1.2, 4.0)).unwrap();
        assert_relative_eq!(r.zo, 98.05233, epsilon = 1e-4);
        assert_relative_eq!(r.zodd, 89.96135, epsilon = 1e-4);
        assert_relative_eq!(r.zeven, 106.87100, epsilon = 1e-4);
        assert_relative_eq!(r.zdiff, 2.0 * r.zodd, max_relative = 1e-12);
        // Zeven must exceed Zodd for any physically coupled pair.
        assert!(r.zeven > r.zodd, "Zeven {} !> Zodd {}", r.zeven, r.zodd);
    }

    /// Coupling must be driven by S/B, not S/H. Doubling the per-plane height at fixed
    /// S doubles B and so *halves* the exponent argument, raising coupling. Guards the
    /// denominator against regressing to `height`.
    #[test]
    fn coupling_scales_with_plane_to_plane_spacing() {
        let thin = calculate(&input(10.0, 10.0, 10.0, 1.4, 4.6)).unwrap();
        let thick = calculate(&input(10.0, 10.0, 20.0, 1.4, 4.6)).unwrap();
        assert!(
            thick.kb.abs() > thin.kb.abs(),
            "larger B must raise coupling: {} !> {}",
            thick.kb,
            thin.kb
        );
    }

    /// elektroda.com vector: T=1.2, H=63, W=10, S=63, Er=4 →
    /// Zodd=78.4, Zeven=78.3, Zdiff=157, Zcommon=39.2.
    ///
    /// Unusable as an oracle: it reports Zeven *below* Zodd, which no coupled-line
    /// model produces (Zeven > Zodd always), and Zodd ≈ Zeven implies essentially zero
    /// coupling at S/B ≈ 0.5 where every published stripline model gives ~8%. Its
    /// Zdiff ≈ 2·Zodd therefore only restates the uncoupled case, so it cannot
    /// discriminate between coupling laws. Its Z0 ≈ 78.4 also only reconciles if its
    /// "H=63" means plane-to-plane rather than per-plane. Left here, ignored, rather
    /// than deleted — resolving it needs a trusted second source.
    #[test]
    #[ignore = "reference vector is internally inconsistent (Zeven < Zodd); see comment"]
    fn elektroda_vector_unreconciled() {
        let r = calculate(&input(10.0, 63.0, 63.0, 1.2, 4.0)).unwrap();
        assert_relative_eq!(r.zodd, 78.4, epsilon = 0.1);
        assert_relative_eq!(r.zdiff, 157.0, epsilon = 0.5);
    }

    /// Narrower spacing increases coupling magnitude (higher Kb).
    #[test]
    fn wider_spacing_reduces_coupling() {
        let close = calculate(&input(10.0, 5.0, 63.0, 1.2, 4.0)).unwrap();
        let far = calculate(&input(10.0, 20.0, 63.0, 1.2, 4.0)).unwrap();

        assert!(
            far.kb.abs() < close.kb.abs(),
            "wider spacing Kb {:.4} should be smaller than {:.4}",
            far.kb,
            close.kb
        );
    }

    /// Higher Er gives lower Z0.
    #[test]
    fn higher_er_gives_lower_z0() {
        let low_er = calculate(&input(10.0, 10.0, 63.0, 1.2, 2.2)).unwrap();
        let high_er = calculate(&input(10.0, 10.0, 63.0, 1.2, 4.6)).unwrap();

        assert!(
            high_er.zo < low_er.zo,
            "higher Er Z0 {:.3} should be less than {:.3}",
            high_er.zo,
            low_er.zo
        );
    }

    /// For a fully embedded stripline, Er_eff equals Er exactly.
    #[test]
    fn er_eff_equals_er() {
        let er = 4.6_f64;
        let result = calculate(&input(10.0, 10.0, 63.0, 1.2, er)).unwrap();

        // Z0 = (60/sqrt(Er)) * ln(...); verify by back-computing Er from Z0 and the log term.
        let ln_term = (1.9_f64 * (2.0 * 63.0 + 1.2) / (0.8 * 10.0 + 1.2)).ln();
        let er_back = (60.0 * ln_term / result.zo).powi(2);
        assert_relative_eq!(er_back, er, max_relative = 1e-10);
    }

    #[test]
    fn rejects_negative_width() {
        let result = calculate(&input(-1.0, 10.0, 63.0, 1.2, 4.0));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_er_below_one() {
        let result = calculate(&input(10.0, 10.0, 63.0, 1.2, 0.5));
        assert!(result.is_err());
    }
}
