//! Copper resistance: temperature correction, and the spreading resistance of a plane.
//!
//! Source: the RESIST worksheet shipped with Johnson & Graham, *High-Speed Digital Design*.
//!
//! Trace and wire DC resistance already live in [`crate::current`] and
//! [`crate::wire_gauge`]. What is here is the part neither of them had:
//!
//! * [`temperature_factor`] — the single home for copper's thermal coefficient. It had no
//!   caller before, which is why `constants::COPPER_TEMP_COEFF` was deleted as dead in the
//!   2026-10-08 cleanup; this module is the caller it was missing.
//! * [`plane_resistance`] — current in a plane is **not** uniform. It spreads out from one
//!   contact and re-converges at the other, so resistance goes as the *logarithm* of
//!   separation and the contact diameter sets the scale. Doubling the distance between two
//!   vias barely changes anything; halving the via diameter matters more. Every other
//!   resistance formula in this crate is `ρL/A`, which would be badly wrong here.
//!
//! These are DC numbers. At any frequency a digital edge cares about, skin effect confines
//! the current to the surface and the real resistance is higher — treat them as a floor.

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;
use crate::constants::{COPPER_RESISTIVITY_OHM_M, COPPER_TEMP_COEFF, MIL_TO_M, ROOM_TEMP_C};

/// Copper resistivity at 20 °C in Ω·mil, for `R = ρ·L/A` with `L` in mils and `A` in mil².
///
/// Derived from the SI figure so the two cannot drift: `1.724e-8 Ω·m / 2.54e-5 m/mil`
/// = `6.787e-4 Ω·mil`, which is the number the Saturn binary and the RESIST worksheet both
/// carry (the latter as `6.787e-7 Ω·in`).
pub const COPPER_RESISTIVITY_OHM_MIL: f64 = COPPER_RESISTIVITY_OHM_M / MIL_TO_M;

/// Resistance multiplier at `temp_c` relative to 20 °C.
///
/// `1 + α·(T − 20)` with α = 0.00393/°C. Copper rises about 0.39% per degree, so over a
/// 0–70 °C operating range a resistance varies by 28% — worth remembering before sizing a
/// voltage-drop budget against a room-temperature number.
///
/// Returns a multiplier, not a resistance, so it composes with any of the `ρL/A` forms.
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] below −234.5 °C, where the linear model predicts a
/// non-positive resistance. That is far outside copper's range of validity anyway, but the
/// alternative is silently returning a negative resistance.
pub fn temperature_factor(temp_c: f64) -> Result<f64, CalcError> {
    all_finite(&[("temp_c", temp_c)])?;
    let factor = 1.0 + (temp_c - ROOM_TEMP_C) * COPPER_TEMP_COEFF;
    if factor <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "temp_c",
            value: temp_c,
            expected: "> -234.5 (where the linear model still gives a positive resistance)",
        });
    }
    Ok(factor)
}

/// Result of a plane spreading-resistance calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaneResistanceResult {
    /// DC resistance between the two contacts (Ω).
    pub resistance_ohms: f64,
    /// The same figure in milliohms, which is the useful scale here.
    pub resistance_milliohms: f64,
    /// Sheet resistance of the copper (Ω per square) at the given temperature.
    pub sheet_resistance_ohms_per_sq: f64,
    /// Temperature multiplier applied.
    pub temperature_factor: f64,
}

/// DC resistance between two contact points on a copper plane.
///
/// ```text
///   R = ρ/(2πt) · [ ln(2s/d₁) + ln(2s/d₂) ] · temperature_factor
/// ```
///
/// `contact1_mils` and `contact2_mils` are the contact diameters — a via, a pad —
/// `thickness_mils` the plane copper and `separation_mils` the distance between contacts.
///
/// Assumes contacts well inside the plane. Near an edge the resistance can be twice this,
/// and near a corner higher still; the model has no way to know, so that caveat stays in the
/// docs rather than in the number.
///
/// # Errors
/// Returns [`CalcError::NegativeDimension`] for non-positive dimensions, and
/// [`CalcError::OutOfRange`] if the contacts are closer than a contact radius, where the
/// spreading picture breaks down and a logarithm would go negative.
pub fn plane_resistance(
    contact1_mils: f64,
    contact2_mils: f64,
    thickness_mils: f64,
    separation_mils: f64,
    temp_c: f64,
) -> Result<PlaneResistanceResult, CalcError> {
    all_finite(&[
        ("contact1_mils", contact1_mils),
        ("contact2_mils", contact2_mils),
        ("thickness_mils", thickness_mils),
        ("separation_mils", separation_mils),
        ("temp_c", temp_c),
    ])?;
    for (name, value) in [
        ("contact1_mils", contact1_mils),
        ("contact2_mils", contact2_mils),
        ("thickness_mils", thickness_mils),
        ("separation_mils", separation_mils),
    ] {
        if value <= 0.0 {
            return Err(CalcError::NegativeDimension { name, value });
        }
    }

    let largest_radius = contact1_mils.max(contact2_mils) / 2.0;
    if separation_mils <= largest_radius {
        return Err(CalcError::OutOfRange {
            name: "separation_mils",
            value: separation_mils,
            expected: "> half the larger contact diameter",
        });
    }

    let tf = temperature_factor(temp_c)?;
    let sheet = COPPER_RESISTIVITY_OHM_MIL / thickness_mils * tf;
    let shape =
        (2.0 * separation_mils / contact1_mils).ln() + (2.0 * separation_mils / contact2_mils).ln();
    let r = sheet / (2.0 * std::f64::consts::PI) * shape;

    Ok(PlaneResistanceResult {
        resistance_ohms: r,
        resistance_milliohms: r * 1000.0,
        sheet_resistance_ohms_per_sq: sheet,
        temperature_factor: tf,
    })
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    /// The Ω·mil form must be the SI constant converted, not a second hand-typed number.
    #[test]
    fn resistivity_in_ohm_mil_matches_the_si_constant() {
        assert_relative_eq!(COPPER_RESISTIVITY_OHM_MIL, 6.787e-4, max_relative = 1e-3);
        assert_relative_eq!(
            COPPER_RESISTIVITY_OHM_MIL * MIL_TO_M,
            COPPER_RESISTIVITY_OHM_M,
            max_relative = 1e-15
        );
    }

    /// The figures the RESIST worksheet quotes: unity at 20 °C, and about 1.195 at 70 °C.
    #[test]
    fn temperature_factor_matches_the_worksheet() {
        assert_relative_eq!(temperature_factor(20.0).unwrap(), 1.0, epsilon = 1e-15);
        assert_relative_eq!(
            temperature_factor(70.0).unwrap(),
            1.1965,
            max_relative = 1e-3
        );
    }

    /// The 28% swing over a 0–70 °C operating range, which is the whole reason to have this.
    #[test]
    fn a_0_to_70_degree_range_is_a_28_percent_swing() {
        let cold = temperature_factor(0.0).unwrap();
        let hot = temperature_factor(70.0).unwrap();
        let room = temperature_factor(20.0).unwrap();
        assert_relative_eq!((hot - cold) / room, 0.275, epsilon = 0.01);
    }

    /// Linear in temperature, by construction.
    #[test]
    fn temperature_factor_is_linear() {
        let a = temperature_factor(20.0).unwrap();
        let b = temperature_factor(70.0).unwrap();
        let c = temperature_factor(120.0).unwrap();
        assert_relative_eq!(b - a, c - b, max_relative = 1e-12);
    }

    #[test]
    fn absurdly_cold_is_rejected_rather_than_going_negative() {
        assert!(temperature_factor(-300.0).is_err());
        assert!(temperature_factor(-200.0).is_ok());
    }

    /// The worked example stored in the RESIST worksheet: two 25 mil vias 4 inches apart in
    /// 1 oz copper → 0.91 mΩ.
    ///
    /// Just under a milliohm across four inches of plane is the number worth internalising,
    /// and it is why plane resistance is rarely the problem that plane *inductance* is.
    #[test]
    fn hsdd_resist_worksheet_vector() {
        let r = plane_resistance(25.0, 25.0, 1.37, 4000.0, ROOM_TEMP_C).unwrap();
        assert_relative_eq!(r.resistance_milliohms, 0.91, max_relative = 5e-3);
    }

    /// The headline property: resistance goes as the **log** of separation, and the lesson
    /// is in how that compares to contact size.
    ///
    /// Doubling the separation and halving both contact diameters add *exactly the same*
    /// amount — `ρ/(2πt)·2·ln 2` — because each contributes one `ln 2` per logarithm. That
    /// equality is the point: moving two vias from 2 inches apart to 4 inches is a large
    /// layout change, and shrinking them from 25 mil to 12.5 mil is a small one, for the
    /// same penalty. Asserting the shared quantum rather than an inequality is also what
    /// catches a slipped factor of two in either logarithm.
    #[test]
    fn separation_and_contact_size_enter_logarithmically_and_equally() {
        let base = plane_resistance(25.0, 25.0, 1.37, 2000.0, ROOM_TEMP_C)
            .unwrap()
            .resistance_ohms;
        let twice_as_far = plane_resistance(25.0, 25.0, 1.37, 4000.0, ROOM_TEMP_C)
            .unwrap()
            .resistance_ohms;
        let half_the_via = plane_resistance(12.5, 12.5, 1.37, 2000.0, ROOM_TEMP_C)
            .unwrap()
            .resistance_ohms;

        // Doubling the distance adds well under 20% — nothing like a factor of two.
        assert!(
            twice_as_far / base < 1.2,
            "doubling separation gave {}x",
            twice_as_far / base
        );

        let quantum = COPPER_RESISTIVITY_OHM_MIL / 1.37 / (2.0 * std::f64::consts::PI)
            * 2.0
            * std::f64::consts::LN_2;
        assert_relative_eq!(twice_as_far - base, quantum, max_relative = 1e-12);
        assert_relative_eq!(half_the_via - base, quantum, max_relative = 1e-12);
        assert_relative_eq!(twice_as_far, half_the_via, max_relative = 1e-12);

        // Halving only one contact is half the penalty, which is the other half of the
        // same statement.
        let half_one_via = plane_resistance(12.5, 25.0, 1.37, 2000.0, ROOM_TEMP_C)
            .unwrap()
            .resistance_ohms;
        assert_relative_eq!(half_one_via - base, quantum / 2.0, max_relative = 1e-12);
    }

    /// Thinner copper is proportionally worse, and sheet resistance must be `ρ/t`.
    #[test]
    fn resistance_is_inversely_proportional_to_thickness() {
        let thin = plane_resistance(25.0, 25.0, 0.685, 4000.0, ROOM_TEMP_C).unwrap();
        let thick = plane_resistance(25.0, 25.0, 1.37, 4000.0, ROOM_TEMP_C).unwrap();
        assert_relative_eq!(
            thin.resistance_ohms / thick.resistance_ohms,
            2.0,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            thick.sheet_resistance_ohms_per_sq,
            COPPER_RESISTIVITY_OHM_MIL / 1.37,
            max_relative = 1e-12
        );
    }

    /// Half an ounce of copper is about 0.98 mΩ/square, the figure usually quoted as
    /// "half a milliohm per square for an ounce".
    #[test]
    fn sheet_resistance_of_one_ounce_copper_is_about_half_a_milliohm() {
        let r = plane_resistance(25.0, 25.0, 1.37, 1000.0, ROOM_TEMP_C).unwrap();
        assert_relative_eq!(
            r.sheet_resistance_ohms_per_sq * 1000.0,
            0.495,
            max_relative = 1e-2
        );
    }

    /// Temperature must scale the whole result, not just part of it.
    #[test]
    fn temperature_scales_the_result() {
        let cool = plane_resistance(25.0, 25.0, 1.37, 4000.0, 20.0).unwrap();
        let warm = plane_resistance(25.0, 25.0, 1.37, 4000.0, 70.0).unwrap();
        assert_relative_eq!(
            warm.resistance_ohms / cool.resistance_ohms,
            temperature_factor(70.0).unwrap(),
            max_relative = 1e-12
        );
        assert_relative_eq!(warm.temperature_factor, 1.1965, max_relative = 1e-3);
    }

    #[test]
    fn contacts_closer_than_a_radius_are_rejected() {
        assert!(plane_resistance(100.0, 25.0, 1.37, 40.0, ROOM_TEMP_C).is_err());
        assert!(plane_resistance(100.0, 25.0, 1.37, 60.0, ROOM_TEMP_C).is_ok());
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        assert!(plane_resistance(0.0, 25.0, 1.37, 1000.0, ROOM_TEMP_C).is_err());
        assert!(plane_resistance(25.0, -1.0, 1.37, 1000.0, ROOM_TEMP_C).is_err());
        assert!(plane_resistance(25.0, 25.0, 0.0, 1000.0, ROOM_TEMP_C).is_err());
        assert!(plane_resistance(25.0, 25.0, 1.37, 0.0, ROOM_TEMP_C).is_err());
    }
}
