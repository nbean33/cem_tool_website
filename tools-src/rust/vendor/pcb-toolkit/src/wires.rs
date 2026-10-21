//! Non-planar transmission lines: round wire over a plane, coax, twisted pair.
//!
//! Source: the ROUND, COAX and TWIST worksheets shipped with Johnson & Graham, *High-Speed
//! Digital Design*. Each is a single logarithm of a shape ratio, which is what makes them
//! worth having — a flying lead or a bit of coax on the bench is exactly where you want a
//! two-significant-figure answer without opening a field solver.
//!
//! ```text
//!   round wire over a plane:   Zo = 60·ln(4h/d)              (air, so Er_eff = 1)
//!   coax:                      Zo = (60/√Er)·ln(D/d)
//!   twisted pair:              Zo = (120/√Er)·ln(2s/d)
//! ```
//!
//! All three return the same [`ImpedanceResult`] as the planar topologies, through the same
//! [`common::finish`] post-condition, so Tpd, Lo and Co are derived identically. The
//! worksheets also publish explicit per-inch L and C coefficients, and those agree with the
//! derived ones — `√(L/C)` recovers each `Zo` coefficient to four digits and `√(LC)` recovers
//! the free-space delay. The `L_from_Zo` route is used here because it shares a single
//! definition of `c` with the rest of the crate; see
//! [`derived_lc_matches_the_worksheet_coefficients`](#).
//!
//! # Range of validity
//!
//! These are the wide-separation forms. The round-wire and twisted-pair expressions assume
//! the conductors are far enough apart that each one's current is still uniformly
//! distributed; below roughly `h/d < 1` (wire almost touching the plane) or `s/d < 1.5`
//! (wires almost touching) proximity effect redistributes it and the real impedance is
//! lower. Coax has no such limit — it is exact for a perfect coaxial geometry.

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;
use crate::impedance::common;
use crate::impedance::types::ImpedanceResult;

/// Inputs for a round wire suspended above a ground plane — wire-wrap, a flying lead.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoundWireInput {
    /// Wire diameter (mils).
    pub diameter: f64,
    /// Height of the wire centre above the plane (mils).
    pub height: f64,
    /// Relative permittivity of the surrounding medium. 1.0 for bare wire in air.
    pub er: f64,
}

/// Inputs for coaxial cable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CoaxInput {
    /// Centre-conductor diameter (mils).
    pub inner_diameter: f64,
    /// Inside diameter of the shield (mils).
    pub shield_diameter: f64,
    /// Relative permittivity of the dielectric. 2.1 for solid PTFE, 2.3 for polyethylene.
    pub er: f64,
}

/// Inputs for a twisted pair.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TwistedPairInput {
    /// Wire diameter (mils).
    pub diameter: f64,
    /// Centre-to-centre spacing between the two wires (mils).
    pub spacing: f64,
    /// **Effective** relative permittivity of the medium between the wires. Lower than the
    /// insulation's own figure, because much of the field is in air — 1.8 to 2.5 is typical
    /// for PVC-insulated pairs where the insulation itself is nearer 3.5.
    pub er: f64,
}

fn check_er(er: f64) -> Result<(), CalcError> {
    if er < 1.0 {
        return Err(CalcError::OutOfRange {
            name: "er",
            value: er,
            expected: ">= 1.0",
        });
    }
    Ok(())
}

fn check_positive(name: &'static str, value: f64) -> Result<(), CalcError> {
    if value <= 0.0 {
        return Err(CalcError::NegativeDimension { name, value });
    }
    Ok(())
}

/// Characteristic impedance and derived quantities for a round wire over a plane.
///
/// `Zo = (60/√Er)·ln(4h/d)`. The `4h` is the image-charge separation: the plane is
/// equivalent to a second wire at `-h`, so the wire sees twice its own height, doubled again
/// by the ratio convention.
///
/// # Errors
/// [`CalcError::NegativeDimension`] for non-positive dimensions,
/// [`CalcError::OutOfRange`] if `er < 1`, and [`CalcError::UnphysicalResult`] (via the shared
/// post-condition) if the wire is so close to the plane that `4h ≤ d` and the logarithm goes
/// non-positive.
pub fn round_wire(input: &RoundWireInput) -> Result<ImpedanceResult, CalcError> {
    let RoundWireInput {
        diameter,
        height,
        er,
    } = *input;
    all_finite(&[("diameter", diameter), ("height", height), ("er", er)])?;
    check_positive("diameter", diameter)?;
    check_positive("height", height)?;
    check_er(er)?;

    let zo = 60.0 / er.sqrt() * (4.0 * height / diameter).ln();
    common::finish(zo, er, er)
}

/// Characteristic impedance and derived quantities for coaxial cable.
///
/// `Zo = (60/√Er)·ln(D/d)`.
///
/// # Errors
/// [`CalcError::NegativeDimension`] for non-positive diameters, [`CalcError::OutOfRange`] if
/// the shield does not exceed the centre conductor or `er < 1`.
pub fn coax(input: &CoaxInput) -> Result<ImpedanceResult, CalcError> {
    let CoaxInput {
        inner_diameter,
        shield_diameter,
        er,
    } = *input;
    all_finite(&[
        ("inner_diameter", inner_diameter),
        ("shield_diameter", shield_diameter),
        ("er", er),
    ])?;
    check_positive("inner_diameter", inner_diameter)?;
    check_positive("shield_diameter", shield_diameter)?;
    check_er(er)?;
    if shield_diameter <= inner_diameter {
        return Err(CalcError::OutOfRange {
            name: "shield_diameter",
            value: shield_diameter,
            expected: "> inner_diameter",
        });
    }

    let zo = 60.0 / er.sqrt() * (shield_diameter / inner_diameter).ln();
    common::finish(zo, er, er)
}

/// Characteristic impedance and derived quantities for a twisted pair.
///
/// `Zo = (120/√Er)·ln(2s/d)`. Same shape as coax with `2s/d` in place of `D/d`, and twice
/// the coefficient because both conductors carry field rather than one inside a shield.
///
/// Note that `er` here is an *effective* permittivity, not the insulation's — see
/// [`TwistedPairInput::er`].
///
/// **This is also the parallel-wires formula.** `120·ln(2s/d)` is `276.31·log10(2s/d)`, and
/// the textbook parallel-wires expression is written with that coefficient rounded to 276 —
/// so the two agree to 0.11%, the rounding being the whole of the difference. That is the
/// form usually quoted for an untwisted pair held at a fixed spacing: a
/// ribbon pair, or two wires in free air. Pass the actual centre-to-centre spacing and the
/// medium's effective permittivity (1.0 for bare wires in air) and the answer is the same;
/// twisting changes the coupling to the *surroundings*, not the impedance between the two
/// conductors. So there is no separate parallel-wires calculator, by design.
///
/// # Errors
/// [`CalcError::NegativeDimension`] for non-positive dimensions, [`CalcError::OutOfRange`] if
/// the wires would overlap (`spacing < diameter`) or `er < 1`.
pub fn twisted_pair(input: &TwistedPairInput) -> Result<ImpedanceResult, CalcError> {
    let TwistedPairInput {
        diameter,
        spacing,
        er,
    } = *input;
    all_finite(&[("diameter", diameter), ("spacing", spacing), ("er", er)])?;
    check_positive("diameter", diameter)?;
    check_positive("spacing", spacing)?;
    check_er(er)?;
    if spacing < diameter {
        return Err(CalcError::OutOfRange {
            name: "spacing",
            value: spacing,
            expected: ">= diameter (wires cannot overlap)",
        });
    }

    let zo = 120.0 / er.sqrt() * (2.0 * spacing / diameter).ln();
    common::finish(zo, er, er)
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    /// 50 Ω coax sanity: solid PTFE (Er = 2.1) needs `D/d = exp(50√2.1/60) = 3.3`.
    ///
    /// This is the check that needs no worksheet: the shield-to-centre ratio for 50 Ω in
    /// PTFE is about 3.35, and it falls straight out of the formula.
    #[test]
    fn fifty_ohm_ptfe_coax_has_the_expected_diameter_ratio() {
        let ratio = (50.0 * 2.1f64.sqrt() / 60.0).exp();
        assert_relative_eq!(ratio, 3.345_495_214_628_275_7, max_relative = 1e-12);
        let r = coax(&CoaxInput {
            inner_diameter: 10.0,
            shield_diameter: 10.0 * ratio,
            er: 2.1,
        })
        .unwrap();
        assert_relative_eq!(r.zo, 50.0, max_relative = 1e-12);
    }

    /// RG-58-like geometry: 0.032" centre, 0.116" shield, polyethylene Er = 2.3 → ~51 Ω.
    #[test]
    fn rg58_geometry_lands_near_50_ohms() {
        let r = coax(&CoaxInput {
            inner_diameter: 32.0,
            shield_diameter: 116.0,
            er: 2.3,
        })
        .unwrap();
        assert!(r.zo > 49.0 && r.zo < 52.0, "zo = {}", r.zo);
        assert_relative_eq!(r.er_eff, 2.3, epsilon = 1e-12);
    }

    /// The worked example stored in the TWIST worksheet: d = 20 mil, s = 38 mil, Er = 2.5
    /// → 101.3 Ω.
    #[test]
    fn hsdd_twist_worksheet_vector() {
        let r = twisted_pair(&TwistedPairInput {
            diameter: 20.0,
            spacing: 38.0,
            er: 2.5,
        })
        .unwrap();
        assert_relative_eq!(r.zo, 101.3, max_relative = 5e-4);
    }

    /// A 10 mil wire 100 mil above a plane: `60·ln(40) = 221.3 Ω`, at the speed of light.
    #[test]
    fn round_wire_in_air_is_fast_and_high_impedance() {
        let r = round_wire(&RoundWireInput {
            diameter: 10.0,
            height: 100.0,
            er: 1.0,
        })
        .unwrap();
        assert_relative_eq!(r.zo, 60.0 * 40f64.ln(), max_relative = 1e-12);
        assert_relative_eq!(r.zo, 221.332_767_246_836_17, max_relative = 1e-12);
        assert_relative_eq!(r.er_eff, 1.0, epsilon = 1e-12);
        // Free-space delay: 84.72528 ps/in, derived from the crate's exact c. The
        // worksheets round this to 84.72.
        assert_relative_eq!(r.tpd_ps_per_in, 84.725_280_180_330_6, max_relative = 1e-12);
    }

    /// The worksheets publish explicit per-inch L and C coefficients for these three
    /// geometries; this crate derives L and C from Zo and Tpd instead, through the shared
    /// post-condition. The two routes must agree, or one of them is wrong.
    ///
    /// Worksheet coefficients: round wire `L = 5.08 nH·ln(4h/d)`, `C = 1.413 pF/ln(4h/d)`;
    /// coax `L = 5.08 nH·ln(D/d)`, `C = 1.41·Er pF/ln(D/d)`; twisted pair
    /// `L = 10.16 nH·ln(2s/d)`, `C = 0.7065·Er pF/ln(2s/d)`. They are rounded to four
    /// figures in the source — `1.41` for coax capacitance is only three — hence the 2e-3
    /// tolerance.
    #[test]
    fn derived_lc_matches_the_worksheet_coefficients() {
        // Round wire in air.
        let shape = (4.0 * 100.0 / 10.0f64).ln();
        let r = round_wire(&RoundWireInput {
            diameter: 10.0,
            height: 100.0,
            er: 1.0,
        })
        .unwrap();
        assert_relative_eq!(r.lo_nh_per_in, 5.08 * shape, max_relative = 2e-3);
        assert_relative_eq!(r.co_pf_per_in, 1.413 / shape, max_relative = 2e-3);

        // Coax.
        let shape = (116.0 / 32.0f64).ln();
        let c = coax(&CoaxInput {
            inner_diameter: 32.0,
            shield_diameter: 116.0,
            er: 2.3,
        })
        .unwrap();
        assert_relative_eq!(c.lo_nh_per_in, 5.08 * shape, max_relative = 2e-3);
        assert_relative_eq!(c.co_pf_per_in, 1.41 * 2.3 / shape, max_relative = 2e-3);

        // Twisted pair — note the doubled inductance coefficient.
        let shape = (2.0 * 38.0 / 20.0f64).ln();
        let t = twisted_pair(&TwistedPairInput {
            diameter: 20.0,
            spacing: 38.0,
            er: 2.5,
        })
        .unwrap();
        assert_relative_eq!(t.lo_nh_per_in, 10.16 * shape, max_relative = 2e-3);
        assert_relative_eq!(t.co_pf_per_in, 0.7065 * 2.5 / shape, max_relative = 2e-3);
    }

    /// Zo scales as 1/√Er in all three, exactly.
    #[test]
    fn impedance_scales_as_inverse_root_er() {
        let a = coax(&CoaxInput {
            inner_diameter: 10.0,
            shield_diameter: 35.0,
            er: 1.0,
        })
        .unwrap();
        let b = coax(&CoaxInput {
            inner_diameter: 10.0,
            shield_diameter: 35.0,
            er: 4.0,
        })
        .unwrap();
        assert_relative_eq!(a.zo / b.zo, 2.0, max_relative = 1e-12);
    }

    /// Wider spacing raises impedance in every geometry; that is the whole shape of the
    /// logarithm and the easiest thing to get backwards.
    #[test]
    fn more_separation_means_higher_impedance() {
        let low = round_wire(&RoundWireInput {
            diameter: 10.0,
            height: 20.0,
            er: 1.0,
        })
        .unwrap();
        let high = round_wire(&RoundWireInput {
            diameter: 10.0,
            height: 200.0,
            er: 1.0,
        })
        .unwrap();
        assert!(high.zo > low.zo);

        let near = twisted_pair(&TwistedPairInput {
            diameter: 20.0,
            spacing: 22.0,
            er: 2.0,
        })
        .unwrap();
        let far = twisted_pair(&TwistedPairInput {
            diameter: 20.0,
            spacing: 60.0,
            er: 2.0,
        })
        .unwrap();
        assert!(far.zo > near.zo);
    }

    /// A wire resting on the plane (`4h ≤ d`) has no physical answer in this model, and must
    /// be refused rather than returning a zero or negative impedance.
    #[test]
    fn wire_touching_the_plane_is_rejected() {
        let r = round_wire(&RoundWireInput {
            diameter: 100.0,
            height: 25.0,
            er: 1.0,
        });
        assert!(r.is_err(), "4h == d should be rejected, got {r:?}");
    }

    #[test]
    fn invalid_geometry_is_rejected() {
        assert!(
            coax(&CoaxInput {
                inner_diameter: 50.0,
                shield_diameter: 50.0,
                er: 2.1
            })
            .is_err()
        );
        assert!(
            coax(&CoaxInput {
                inner_diameter: -1.0,
                shield_diameter: 50.0,
                er: 2.1
            })
            .is_err()
        );
        assert!(
            twisted_pair(&TwistedPairInput {
                diameter: 20.0,
                spacing: 10.0,
                er: 2.0
            })
            .is_err()
        );
        assert!(
            twisted_pair(&TwistedPairInput {
                diameter: 20.0,
                spacing: 40.0,
                er: 0.5
            })
            .is_err()
        );
        assert!(
            round_wire(&RoundWireInput {
                diameter: 10.0,
                height: 0.0,
                er: 1.0
            })
            .is_err()
        );
    }
}
