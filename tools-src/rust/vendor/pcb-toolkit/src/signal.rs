//! Signal-integrity rules of thumb: what to *do* with an impedance.
//!
//! Source: Howard Johnson and Martin Graham, *High-Speed Digital Design: A Handbook of Black
//! Magic*, Prentice Hall 1993, and the Mathcad worksheets shipped with it.
//!
//! Everything here is first-order and deliberately so. The rest of this crate computes
//! cross-sections; these are the handful of relations that turn a cross-section into a layout
//! decision — how much bandwidth an edge actually contains, how long a run can be before it
//! has to be treated as a transmission line, and what resistor to put at the driver.
//!
//! Two of these are calculators Saturn PCB Toolkit has and this crate did not: its
//! "bandwidth" tab is [`knee_frequency`] and its "max conductor length" tab is
//! [`critical_length`]. Note that Saturn's bandwidth tab computes Er_eff with `0.457·Er + 0.67`
//! where its own crosstalk form uses `0.475` — the two disagree with each other — which is
//! part of why neither is treated as a reference here. See `CLAUDE.md`.

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;
use crate::constants::SPEED_OF_LIGHT_IN_NS;

/// A run shorter than `rise_time / (RISETIME_RULE × Tpd)` behaves as a lumped element.
///
/// Johnson & Graham use six: an unterminated line whose one-way delay is under a sixth of
/// the driver's rise time rings by less than a few percent, because the reflections land
/// inside the edge. The figure is a rule of thumb derived from a step-response sweep, not a
/// closed form, which is why it is a named parameter of [`critical_length`] rather than a
/// buried literal — a design with a tighter overshoot budget wants a larger divisor.
pub const RISETIME_RULE: f64 = 6.0;

// ---------------------------------------------------------------------------------------
// Knee frequency
// ---------------------------------------------------------------------------------------

/// Result of a signal-edge bandwidth calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeResult {
    /// The 10–90% rise time the calculation was made for (ns).
    pub rise_time_ns: f64,
    /// Highest frequency that carries meaningful energy in the edge (Hz).
    pub knee_frequency_hz: f64,
    /// The same figure in MHz, which is how it is normally quoted.
    pub knee_frequency_mhz: f64,
}

/// Highest frequency that matters in a digital edge: `F_knee = 0.5 / Tr`.
///
/// Above the knee a step contains little energy; below it the spectrum follows the step. Any
/// circuit that behaves properly up to the knee passes the edge more or less intact.
///
/// Note what this does **not** depend on: the clock rate. A 10 MHz clock with a 500 ps edge
/// is a 1 GHz design problem, and that is the entire reason the figure is worth computing.
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] if `rise_time_ns` ≤ 0.
pub fn knee_frequency(rise_time_ns: f64) -> Result<EdgeResult, CalcError> {
    all_finite(&[("rise_time_ns", rise_time_ns)])?;
    if rise_time_ns <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "rise_time_ns",
            value: rise_time_ns,
            expected: "> 0",
        });
    }
    let hz = 0.5 / (rise_time_ns * 1e-9);
    Ok(EdgeResult {
        rise_time_ns,
        knee_frequency_hz: hz,
        knee_frequency_mhz: hz / 1e6,
    })
}

/// The 10–90% rise time (ns) whose knee frequency is `f_knee_hz`.
///
/// The inverse of [`knee_frequency`]: turns a component bandwidth into the fastest edge that
/// component can carry.
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] if `f_knee_hz` ≤ 0.
pub fn rise_time_for_knee(f_knee_hz: f64) -> Result<f64, CalcError> {
    all_finite(&[("f_knee_hz", f_knee_hz)])?;
    if f_knee_hz <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "f_knee_hz",
            value: f_knee_hz,
            expected: "> 0",
        });
    }
    Ok(0.5 / f_knee_hz * 1e9)
}

// ---------------------------------------------------------------------------------------
// Critical length and termination
// ---------------------------------------------------------------------------------------

/// Result of a critical-length / termination decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticalLengthResult {
    /// Longest run that still behaves as a lumped element (inches).
    pub critical_length_in: f64,
    /// The same length in mm.
    pub critical_length_mm: f64,
    /// One-way flight time of the critical length (ns) — always `rise_time / rule`.
    pub critical_flight_time_ns: f64,
    /// The divisor used, normally [`RISETIME_RULE`].
    pub rule: f64,
}

/// Longest run (inches) that still behaves as a lumped element.
///
/// `Lcrit = Tr / (rule × Tpd)`. Johnson & Graham's rule of thumb: a point-to-point line has
/// to be treated as a transmission line — and probably terminated — once its one-way delay
/// exceeds about a sixth of the driver's rise time. Below that the reflections land inside
/// the edge and the line is just a small lumped capacitance.
///
/// `tpd_ps_per_in` comes straight out of any [`crate::impedance`] or [`crate::wires`]
/// result, or from a datasheet.
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] if any argument is ≤ 0.
pub fn critical_length(
    rise_time_ns: f64,
    tpd_ps_per_in: f64,
    rule: f64,
) -> Result<CriticalLengthResult, CalcError> {
    all_finite(&[
        ("rise_time_ns", rise_time_ns),
        ("tpd_ps_per_in", tpd_ps_per_in),
        ("rule", rule),
    ])?;
    if rise_time_ns <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "rise_time_ns",
            value: rise_time_ns,
            expected: "> 0",
        });
    }
    if tpd_ps_per_in <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "tpd_ps_per_in",
            value: tpd_ps_per_in,
            expected: "> 0",
        });
    }
    if rule <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "rule",
            value: rule,
            expected: "> 0",
        });
    }
    // Tr in ns, Tpd in ps/in → length in inches needs Tpd in ns/in.
    let inches = rise_time_ns / (rule * tpd_ps_per_in / 1000.0);
    Ok(CriticalLengthResult {
        critical_length_in: inches,
        critical_length_mm: inches * 25.4,
        critical_flight_time_ns: rise_time_ns / rule,
        rule,
    })
}

// ---------------------------------------------------------------------------------------
// Mismatch
// ---------------------------------------------------------------------------------------

/// Result of an impedance-mismatch calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MismatchResult {
    /// Voltage reflection coefficient Γ, signed.
    pub gamma: f64,
    /// Magnitude of Γ, which is what the dB figures are taken from.
    pub gamma_magnitude: f64,
    /// Voltage standing wave ratio. Infinite at a total reflection.
    pub vswr: f64,
    /// Return loss (dB, positive). Infinite at a perfect match.
    pub return_loss_db: f64,
    /// Mismatch (transmission) loss in dB, positive.
    pub mismatch_loss_db: f64,
    /// Series resistor that source-terminates the line (Ω), 0 when none helps.
    pub series_terminator_ohms: f64,
}

/// Reflection, VSWR, return loss and the series terminator for a step from
/// `z_source_ohms` into `z_line_ohms`.
///
/// `Γ = (Z_line − Z_source) / (Z_line + Z_source)`. Positive means the line looks higher
/// than what drives it and the voltage steps *up* at the junction; multiply by the driver
/// swing for the size of the step.
///
/// The series terminator is `max(Z_line − Z_source, 0)`. It works by making the driver's
/// output impedance add up to the line impedance, so the reflection returning from the far
/// end is absorbed rather than re-reflected. The resistor goes **at the driver pin** — any
/// line between the driver and the resistor is unmatched. Zero means the driver already
/// matches or exceeds the line, where a series resistor makes things worse.
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] if either impedance is ≤ 0.
pub fn mismatch(z_line_ohms: f64, z_source_ohms: f64) -> Result<MismatchResult, CalcError> {
    all_finite(&[
        ("z_line_ohms", z_line_ohms),
        ("z_source_ohms", z_source_ohms),
    ])?;
    if z_line_ohms <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "z_line_ohms",
            value: z_line_ohms,
            expected: "> 0",
        });
    }
    if z_source_ohms <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "z_source_ohms",
            value: z_source_ohms,
            expected: "> 0",
        });
    }

    let gamma = (z_line_ohms - z_source_ohms) / (z_line_ohms + z_source_ohms);
    let mag = gamma.abs();

    // Both dB figures are infinite at a perfect match, which is correct and serialises as
    // JSON `null`; f64::INFINITY is the honest answer and the caller can render it.
    let (vswr, return_loss_db) = if mag == 0.0 {
        (1.0, f64::INFINITY)
    } else {
        ((1.0 + mag) / (1.0 - mag), -20.0 * mag.log10())
    };
    let mismatch_loss_db = -10.0 * (1.0 - mag * mag).log10();

    Ok(MismatchResult {
        gamma,
        gamma_magnitude: mag,
        vswr,
        return_loss_db,
        mismatch_loss_db,
        series_terminator_ohms: (z_line_ohms - z_source_ohms).max(0.0),
    })
}

// ---------------------------------------------------------------------------------------
// Er from a measured delay
// ---------------------------------------------------------------------------------------

/// Result of backing an effective permittivity out of a delay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErFromDelayResult {
    /// Effective relative permittivity implied by the delay.
    pub er_eff: f64,
    /// Propagation velocity (in/ns).
    pub velocity_in_per_ns: f64,
    /// Velocity as a fraction of c.
    pub velocity_fraction_of_c: f64,
}

/// Effective relative permittivity implied by a propagation delay.
///
/// `Er_eff = (Tpd · c)²`, the exact inverse of [`crate::impedance::common::propagation_delay`].
/// Useful for backing an Er_eff out of a TDR measurement or a datasheet delay — the number
/// you then feed to [`critical_length`] or compare against a stackup's nominal.
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] if `tpd_ps_per_in` ≤ 0, or
/// [`CalcError::UnphysicalResult`] if the delay is faster than light in vacuum, which would
/// imply `Er_eff < 1`.
pub fn er_from_delay(tpd_ps_per_in: f64) -> Result<ErFromDelayResult, CalcError> {
    all_finite(&[("tpd_ps_per_in", tpd_ps_per_in)])?;
    if tpd_ps_per_in <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "tpd_ps_per_in",
            value: tpd_ps_per_in,
            expected: "> 0",
        });
    }
    let tpd_ns_per_in = tpd_ps_per_in / 1000.0;
    let er_eff = (tpd_ns_per_in * SPEED_OF_LIGHT_IN_NS).powi(2);
    // A delay quoted to the digit a TDR prints can land a part in 10^9 below free space, so
    // the guard carries the same small relative tolerance `impedance::common::finish` uses
    // for an exactly-homogeneous line. Anything genuinely superluminal is far outside it.
    if er_eff < 1.0 - 1e-9 {
        return Err(CalcError::UnphysicalResult {
            name: "er_eff",
            value: er_eff,
            reason: "delay is faster than light in vacuum",
        });
    }
    let velocity = 1.0 / tpd_ns_per_in;
    Ok(ErFromDelayResult {
        er_eff,
        velocity_in_per_ns: velocity,
        velocity_fraction_of_c: velocity / SPEED_OF_LIGHT_IN_NS,
    })
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::impedance::common::propagation_delay;

    /// `0.5 / Tr`, checked at the figure the book quotes: a 1 ns edge is a 500 MHz problem.
    #[test]
    fn knee_frequency_of_a_1ns_edge_is_500mhz() {
        let r = knee_frequency(1.0).unwrap();
        assert_relative_eq!(r.knee_frequency_mhz, 500.0, max_relative = 1e-12);
        assert_relative_eq!(r.knee_frequency_hz, 5e8, max_relative = 1e-12);
    }

    /// A 350 ps edge — a plain LVCMOS output — is a 1429 MHz problem. This is the number
    /// that makes the point: nothing about it mentions a clock rate.
    #[test]
    fn knee_frequency_of_a_350ps_edge() {
        let r = knee_frequency(0.35).unwrap();
        assert_relative_eq!(
            r.knee_frequency_mhz,
            1_428.571_428_571_428_7,
            max_relative = 1e-12
        );
    }

    #[test]
    fn knee_and_rise_time_are_inverses() {
        for tr in [0.05, 0.35, 1.0, 5.0] {
            let f = knee_frequency(tr).unwrap().knee_frequency_hz;
            assert_relative_eq!(rise_time_for_knee(f).unwrap(), tr, max_relative = 1e-12);
        }
    }

    /// 1 ns edge on FR-4 inner layer (180 ps/in) → 0.926 in, the worked figure from the
    /// source worksheet.
    #[test]
    fn critical_length_of_a_1ns_edge_on_fr4() {
        let r = critical_length(1.0, 180.0, RISETIME_RULE).unwrap();
        assert_relative_eq!(
            r.critical_length_in,
            0.925_925_925_925_925_9,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            r.critical_length_mm,
            23.518_518_518_518_51,
            max_relative = 1e-12
        );
    }

    /// The defining property, independent of any constant: a run at exactly the critical
    /// length has a one-way flight time of `Tr / rule`. If the unit conversion between ps/in
    /// and ns is wrong, this fails and the length assertion above would not necessarily.
    #[test]
    fn critical_length_flight_time_is_rise_time_over_the_rule() {
        for (tr, tpd) in [(1.0, 180.0), (0.35, 150.0), (5.0, 85.0)] {
            let r = critical_length(tr, tpd, RISETIME_RULE).unwrap();
            let flight_ns = r.critical_length_in * tpd / 1000.0;
            assert_relative_eq!(flight_ns, tr / RISETIME_RULE, max_relative = 1e-12);
            assert_relative_eq!(flight_ns, r.critical_flight_time_ns, max_relative = 1e-12);
        }
    }

    /// A tighter rule must allow a shorter run, and the dependence is exactly reciprocal.
    #[test]
    fn a_stricter_rule_shortens_the_allowed_run() {
        let six = critical_length(1.0, 180.0, 6.0).unwrap().critical_length_in;
        let twelve = critical_length(1.0, 180.0, 12.0)
            .unwrap()
            .critical_length_in;
        assert_relative_eq!(six / twelve, 2.0, max_relative = 1e-12);
    }

    /// Γ at the figure the book quotes: 65 Ω line driven from 50 Ω reflects +0.13.
    #[test]
    fn reflection_from_50_into_65_ohms() {
        let r = mismatch(65.0, 50.0).unwrap();
        assert_relative_eq!(r.gamma, 0.130_434_782_608_695_65, max_relative = 1e-12);
        assert_relative_eq!(r.series_terminator_ohms, 15.0, max_relative = 1e-12);
    }

    /// A perfect match is the one case every derived figure has to special-case.
    #[test]
    fn a_perfect_match_reflects_nothing() {
        let r = mismatch(50.0, 50.0).unwrap();
        assert_eq!(r.gamma, 0.0);
        assert_relative_eq!(r.vswr, 1.0, max_relative = 1e-12);
        assert!(r.return_loss_db.is_infinite());
        assert_relative_eq!(r.mismatch_loss_db, 0.0, epsilon = 1e-15);
        assert_eq!(r.series_terminator_ohms, 0.0);
    }

    /// Γ, VSWR and return loss are three spellings of one quantity, so they must agree by
    /// construction rather than by coincidence.
    #[test]
    fn vswr_and_return_loss_agree_with_gamma() {
        for (zl, zs) in [(75.0, 50.0), (50.0, 75.0), (100.0, 50.0), (33.0, 50.0)] {
            let r = mismatch(zl, zs).unwrap();
            // Γ recovered from VSWR.
            assert_relative_eq!(
                (r.vswr - 1.0) / (r.vswr + 1.0),
                r.gamma_magnitude,
                max_relative = 1e-12
            );
            // Γ recovered from return loss.
            assert_relative_eq!(
                10f64.powf(-r.return_loss_db / 20.0),
                r.gamma_magnitude,
                max_relative = 1e-12
            );
            // Mismatch loss is the power that is not reflected.
            assert_relative_eq!(
                10f64.powf(-r.mismatch_loss_db / 10.0),
                1.0 - r.gamma_magnitude * r.gamma_magnitude,
                max_relative = 1e-12
            );
        }
    }

    /// Swapping source and line flips the sign of Γ and leaves every magnitude alone.
    #[test]
    fn reflection_is_antisymmetric() {
        let a = mismatch(75.0, 50.0).unwrap();
        let b = mismatch(50.0, 75.0).unwrap();
        assert_relative_eq!(a.gamma, -b.gamma, max_relative = 1e-12);
        assert_relative_eq!(a.vswr, b.vswr, max_relative = 1e-12);
        assert_relative_eq!(a.return_loss_db, b.return_loss_db, max_relative = 1e-12);
    }

    /// A series terminator only exists when the driver is weaker than the line.
    #[test]
    fn series_terminator_is_zero_for_a_strong_driver() {
        assert_eq!(mismatch(50.0, 80.0).unwrap().series_terminator_ohms, 0.0);
        assert_relative_eq!(
            mismatch(65.0, 30.0).unwrap().series_terminator_ohms,
            35.0,
            max_relative = 1e-12
        );
    }

    /// `er_from_delay` must invert `propagation_delay` exactly — same constant, both ways.
    #[test]
    fn er_from_delay_inverts_propagation_delay() {
        for er in [1.0, 2.2, 3.5, 4.6, 10.2] {
            let tpd = propagation_delay(er);
            let back = er_from_delay(tpd).unwrap();
            assert_relative_eq!(back.er_eff, er, max_relative = 1e-12);
            assert_relative_eq!(
                back.velocity_in_per_ns,
                SPEED_OF_LIGHT_IN_NS / er.sqrt(),
                max_relative = 1e-12
            );
            assert_relative_eq!(
                back.velocity_fraction_of_c,
                1.0 / er.sqrt(),
                max_relative = 1e-12
            );
        }
    }

    /// Free space is the anchor: 84.72528 ps/in is Er_eff = 1 travelling at c.
    ///
    /// The figure follows from this crate's exact `c` (11.802852677 in/ns). The source
    /// worksheets round it to 84.72, which is a 0.006% difference and the reason the
    /// tolerance here is 1e-9 rather than exact.
    #[test]
    fn free_space_delay_gives_er_of_one() {
        let r = er_from_delay(84.725_280_180_330_6).unwrap();
        assert_relative_eq!(r.er_eff, 1.0, max_relative = 1e-9);
        assert_relative_eq!(r.velocity_fraction_of_c, 1.0, max_relative = 1e-9);
    }

    /// A delay faster than light is an error, not an `Er_eff` below 1.
    #[test]
    fn superluminal_delay_is_rejected() {
        assert!(er_from_delay(50.0).is_err());
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        assert!(knee_frequency(0.0).is_err());
        assert!(knee_frequency(-1.0).is_err());
        assert!(rise_time_for_knee(0.0).is_err());
        assert!(critical_length(0.0, 180.0, 6.0).is_err());
        assert!(critical_length(1.0, 0.0, 6.0).is_err());
        assert!(critical_length(1.0, 180.0, 0.0).is_err());
        assert!(mismatch(0.0, 50.0).is_err());
        assert!(mismatch(50.0, -1.0).is_err());
        assert!(er_from_delay(0.0).is_err());
    }
}
