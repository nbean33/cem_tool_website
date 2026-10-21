//! Standalone crosstalk calculator (microstrip).
//!
//! Estimates backward crosstalk (NEXT) using the standard coupled-line model:
//!
//! Kb = 1 / (4 × (1 + (S/H)²))
//! Lsat = rise_time × v_prop / 2
//! NEXT = Kb × min(coupled_length / Lsat, 1.0)
//!
//! # Divergence from Saturn PCB Toolkit — deliberate
//!
//! Saturn reports −2.23327 dB / 3.86640 V for the reference geometry
//! (`docs/notes/17-test-vectors.md` §17); we report −35.4755 dB / 0.0842 V. **Saturn is high
//! by 48× (+33.6 dB) and we are correct.** Saturn's handler was decompiled from V8.47
//! (`TForm12.Button1Click` at `0x501780`, thunking to `0x501888`; microstrip branch
//! `0x501f05`–`0x501fb9`) and reads:
//!
//! ```text
//! er_eff = 0.475*Er + 0.67
//! t      = 1.017 * sqrt(er_eff)       // 1.017 is ns per FOOT
//! ratio  = 2*t*L_inches / Trise_ns    // clamped at 1.0
//! coeff  = [1/(1 + (S/H)^2)] * ratio  // no 1/4 factor
//! ```
//!
//! That reproduces Saturn's published vector to all six of its digits, so the diagnosis is
//! not a guess. Two independent defects, both in Saturn:
//!
//! 1. The 1/4 is missing from the backward-coupling coefficient — the solver contains no
//!    `0.25` and no `4`. → 4× (+12.0 dB)
//! 2. `1.017` is ns per *foot*, but the coupled length is converted mils→inches and then
//!    multiplied by it. → 12× (+21.6 dB)
//!
//! Saturn itself files this calculator under a menu heading captioned `Unsupported`.
//! Do **not** "fix" this module toward Saturn's numbers. See
//! `docs/notes/18-v847-vs-rust-comparison.md` §1 and VALIDATION.md finding L5.
//!
//! # Scope
//!
//! Microstrip only — Saturn additionally offers a stripline mode taking two plane heights
//! (branch `0x502794`), which is not implemented here.
//!
//! # Known limitations
//!
//! Microstrip only, and backward (NEXT) only. Saturn has a stripline mode (branch
//! `0x502794`, two plane heights) and this crate does not; the output is labelled microstrip
//! so the result cannot be mistaken for a buried pair. Forward crosstalk (FEXT) is not
//! modelled. Both are listed under "deliberately not implemented" in `STATUS.md` rather than
//! as pending work.

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;
use crate::constants::SPEED_OF_LIGHT_IN_NS;
use crate::impedance::common;

/// Inputs for crosstalk estimation.
pub struct CrosstalkInput {
    /// Signal rise time (ns).
    pub rise_time_ns: f64,
    /// Signal voltage (V).
    pub voltage: f64,
    /// Coupled (parallel) trace length (mils).
    pub coupled_length_mils: f64,
    /// Edge-to-edge spacing between traces (mils).
    pub spacing_mils: f64,
    /// Dielectric height — trace to ground plane (mils).
    pub height_mils: f64,
    /// Substrate relative permittivity.
    pub er: f64,
    /// Trace width (mils).
    ///
    /// Feeds `u = W/H` into [`common::er_eff_static`] and nothing else, so it reaches the
    /// result only through `lsat_mils`. Saturn's own crosstalk form has **no** width field
    /// — it approximates Er_eff as `0.475·Er + 0.67` instead — so this input has no
    /// counterpart in the reference vectors. Use [`DEFAULT_TRACE_WIDTH_MILS`] when a
    /// reference vector does not supply one, so every front-end agrees.
    pub trace_width_mils: f64,
}

/// Trace width assumed when a source (such as the Saturn reference vector in
/// `docs/notes/17-test-vectors.md` §17) specifies none.
///
/// Saturn's crosstalk form has no width input, so vectors taken from it are underdetermined
/// for this module. Pinning one value here keeps the CLI, the desktop app and the test suite
/// from each picking their own and reporting different dB for the same documented geometry.
pub const DEFAULT_TRACE_WIDTH_MILS: f64 = 10.0;

/// Result of a crosstalk estimation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrosstalkResult {
    /// Saturated backward crosstalk coefficient Kb (dimensionless, 0–0.25).
    ///
    /// This is the purely *geometric* quarter-sum form `1/(4(1+(S/H)²))`, capped at 0.25,
    /// and it is the **open/unterminated** case. It is *not* the same quantity as
    /// [`crate::differential::types::DifferentialResult::kb`], which is the modal form
    /// `(Zeven−Zodd)/(Zeven+Zodd)` and ranges to 1 — the two differ by roughly 1.8× at
    /// comparable geometry despite the shared name.
    pub kb: f64,
    /// Crosstalk in dB, `20 × log10(next_coefficient)`.
    ///
    /// Note the argument is the **length-derated** coefficient, whereas
    /// `DifferentialResult::kb_db` is `20 × log10(kb)` with no derating. The two are not
    /// comparable: for the §17 reference geometry the derating factor alone is −22.5 dB.
    pub crosstalk_db: f64,
    /// Coupled voltage (V) = NEXT × voltage.
    pub coupled_voltage: f64,
    /// NEXT coefficient (dimensionless, 0–Kb) — Kb derated by `min(L/Lsat, 1)`.
    pub next_coefficient: f64,
    /// Saturation length (mils) — the coupled length at which NEXT reaches Kb.
    pub lsat_mils: f64,
}

/// Estimate backward crosstalk (NEXT) between parallel microstrip traces.
pub fn calculate(input: &CrosstalkInput) -> Result<CrosstalkResult, CalcError> {
    let CrosstalkInput {
        rise_time_ns,
        voltage,
        coupled_length_mils,
        spacing_mils,
        height_mils,
        er,
        trace_width_mils,
    } = *input;

    all_finite(&[
        ("rise_time_ns", rise_time_ns),
        ("voltage", voltage),
        ("coupled_length_mils", coupled_length_mils),
        ("spacing_mils", spacing_mils),
        ("height_mils", height_mils),
        ("er", er),
        ("trace_width_mils", trace_width_mils),
    ])?;

    if rise_time_ns <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "rise_time_ns",
            value: rise_time_ns,
            expected: "> 0",
        });
    }
    if voltage <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "voltage",
            value: voltage,
            expected: "> 0",
        });
    }
    if coupled_length_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "coupled_length_mils",
            value: coupled_length_mils,
        });
    }
    if spacing_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "spacing_mils",
            value: spacing_mils,
        });
    }
    if height_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "height_mils",
            value: height_mils,
        });
    }
    if er < 1.0 {
        return Err(CalcError::OutOfRange {
            name: "er",
            value: er,
            expected: ">= 1.0",
        });
    }
    if trace_width_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "trace_width_mils",
            value: trace_width_mils,
        });
    }

    // Backward crosstalk coefficient
    let s_over_h = spacing_mils / height_mils;
    let kb = 1.0 / (4.0 * (1.0 + s_over_h * s_over_h));

    // Propagation velocity from Er_eff
    let u = trace_width_mils / height_mils;
    let er_eff = common::er_eff_static(u, er);
    // v_prop in in/ns
    let v_prop = SPEED_OF_LIGHT_IN_NS / er_eff.sqrt();
    // Convert to mils/ns: 1 in = 1000 mils
    let v_prop_mils_ns = v_prop * 1000.0;

    // Saturation length (mils)
    let lsat_mils = rise_time_ns * v_prop_mils_ns / 2.0;

    // NEXT coefficient (saturates at Kb)
    let length_ratio = coupled_length_mils / lsat_mils;
    let next_coefficient = kb * length_ratio.min(1.0);

    // Coupled voltage
    let coupled_voltage = next_coefficient * voltage;

    // Crosstalk in dB. The input validators above force `next_coefficient > 0`, so this is
    // unreachable today — but `log10(0)` is `-inf`, which serialises to JSON `null` rather
    // than erroring, so guard it rather than relying on validation staying as it is. The
    // `is_finite` half is what catches NaN, since every comparison against NaN is false.
    if !next_coefficient.is_finite() || next_coefficient <= 0.0 {
        return Err(CalcError::UnphysicalResult {
            name: "next_coefficient",
            value: next_coefficient,
            reason: "not positive — cannot express as dB",
        });
    }
    let crosstalk_db = 20.0 * next_coefficient.log10();

    Ok(CrosstalkResult {
        kb,
        crosstalk_db,
        coupled_voltage,
        next_coefficient,
        lsat_mils,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// The §17 reference geometry, which no test asserted before: Trise 1 ns, V 5,
    /// L 6350 µm (250 mil), S 254 µm (10 mil), H 762 µm (30 mil), Er 4.6, microstrip.
    /// Width is not supplied by the vector — see [`DEFAULT_TRACE_WIDTH_MILS`].
    fn reference_vector() -> CrosstalkInput {
        CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 250.0,
            spacing_mils: 10.0,
            height_mils: 30.0,
            er: 4.6,
            trace_width_mils: DEFAULT_TRACE_WIDTH_MILS,
        }
    }

    /// Absolute assertion on the reference geometry.
    ///
    /// Every other test in this module is a range or monotonicity check, and all of them
    /// pass for a result 33 dB wrong — which is exactly how this module sat unvalidated.
    /// `Kb` is pinned exactly because it is pure geometry: `1/(4(1+(10/30)²)) = 9/40`.
    #[test]
    fn reference_geometry_absolute() {
        let r = calculate(&reference_vector()).unwrap();
        assert_relative_eq!(r.kb, 0.225, max_relative = 1e-15);
        assert_relative_eq!(r.lsat_mils, 3341.1698170025916, max_relative = 1e-12);
        assert_relative_eq!(
            r.next_coefficient,
            0.016835420849833554,
            max_relative = 1e-12
        );
        assert_relative_eq!(r.coupled_voltage, 0.08417710424916777, max_relative = 1e-12);
        assert_relative_eq!(r.crosstalk_db, -35.47552045397531, max_relative = 1e-12);
    }

    /// Pins the *known* divergence from Saturn so it stays explained rather than open.
    ///
    /// Saturn V8.47 publishes −2.23327 dB / 3.86640 V for this geometry, i.e. a coupling
    /// coefficient of 0.773279546. Its decompiled chain (see the module docs) accounts for
    /// the whole gap as an exact product of three factors:
    ///
    /// | factor | value | cause |
    /// |---|---|---|
    /// | missing 1/4 | 4.000000 | Saturn's coefficient is `1/(1+(S/H)²)`, not `0.25/(…)` |
    /// | ns/ft vs ns/in | 12.003501 | `1.017 × 11.8028527` — Saturn applies a per-foot constant to inches |
    /// | Er_eff model | 0.956631 | `√(2.855/3.119729)` — Saturn's `0.475·Er+0.67` vs our Hammerstad-Jensen |
    ///
    /// Product: 45.931703, which is the observed ratio. The first two factors are Saturn's
    /// bugs and multiply to 48.014× (+33.63 dB); the third is a modelling difference.
    /// If this test fails, our formula changed — check it against the table above before
    /// assuming Saturn was right.
    #[test]
    fn saturn_divergence_is_fully_accounted_for() {
        let r = calculate(&reference_vector()).unwrap();
        const SATURN_COEFF: f64 = 0.773_279_546;

        let observed = SATURN_COEFF / r.next_coefficient;
        assert_relative_eq!(observed, 45.931_703, max_relative = 1e-6);

        // Saturn's geometric term is exactly 4x ours because it omits the 1/4.
        let saturn_geometric = 1.0 / (1.0 + (10.0_f64 / 30.0).powi(2));
        assert_relative_eq!(saturn_geometric / r.kb, 4.0, max_relative = 1e-12);

        // And the two Saturn defects alone, with Er_eff held equal, are 48x. The ns/ft
        // factor is `1.017 × c`, using our exact speed of light rather than Saturn's 11.8.
        assert_relative_eq!(
            4.0 * 1.017 * crate::constants::SPEED_OF_LIGHT_IN_NS,
            48.014_005,
            max_relative = 1e-6
        );
    }

    #[test]
    fn basic_crosstalk() {
        let result = calculate(&CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 1000.0,
            spacing_mils: 10.0,
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        })
        .unwrap();

        // Kb should be in range (0, 0.25]
        assert!(result.kb > 0.0 && result.kb <= 0.25, "Kb = {}", result.kb);
        // NEXT should be in range (0, Kb]
        assert!(
            result.next_coefficient > 0.0 && result.next_coefficient <= result.kb,
            "NEXT = {}",
            result.next_coefficient
        );
        assert!(result.coupled_voltage > 0.0 && result.coupled_voltage < 5.0);
        assert!(result.crosstalk_db < 0.0); // dB is negative for NEXT < 1
        assert!(result.lsat_mils > 0.0);
    }

    #[test]
    fn wider_spacing_less_crosstalk() {
        let close = calculate(&CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 1000.0,
            spacing_mils: 5.0,
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        })
        .unwrap();

        let far = calculate(&CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 1000.0,
            spacing_mils: 20.0,
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        })
        .unwrap();

        assert!(
            close.kb > far.kb,
            "close Kb {} should be > far Kb {}",
            close.kb,
            far.kb
        );
    }

    #[test]
    fn kb_max_at_zero_spacing_limit() {
        // As S→0, Kb→0.25
        let result = calculate(&CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 1000.0,
            spacing_mils: 0.01, // very small spacing
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        })
        .unwrap();

        assert!(result.kb > 0.24, "Kb at near-zero spacing = {}", result.kb);
    }

    #[test]
    fn short_coupled_length_reduces_next() {
        let long = calculate(&CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 10000.0,
            spacing_mils: 10.0,
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        })
        .unwrap();

        let short = calculate(&CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 100.0,
            spacing_mils: 10.0,
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        })
        .unwrap();

        assert!(
            short.next_coefficient <= long.next_coefficient,
            "short NEXT {} should be <= long NEXT {}",
            short.next_coefficient,
            long.next_coefficient
        );
    }

    #[test]
    fn rejects_negative_spacing() {
        let result = calculate(&CrosstalkInput {
            rise_time_ns: 1.0,
            voltage: 5.0,
            coupled_length_mils: 1000.0,
            spacing_mils: -1.0,
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        });
        assert!(result.is_err());
    }

    #[test]
    fn rejects_zero_rise_time() {
        let result = calculate(&CrosstalkInput {
            rise_time_ns: 0.0,
            voltage: 5.0,
            coupled_length_mils: 1000.0,
            spacing_mils: 10.0,
            height_mils: 5.0,
            er: 4.6,
            trace_width_mils: 10.0,
        });
        assert!(result.is_err());
    }
}
