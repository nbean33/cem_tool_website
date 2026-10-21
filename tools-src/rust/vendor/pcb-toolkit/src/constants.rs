//! Physical constants shared across calculators.
//!
//! Everything here has at least one caller — a constant with no consumer is deleted rather
//! than kept "in case". Values that follow from other values are **derived** rather than
//! written out, so they cannot drift apart or disagree with their own documentation.
//!
//! Module-private constants are fine when they are genuinely local (IPC-2221A's `k`
//! coefficients, the plane-capacitance figure in `pdn`). They are not fine when they
//! duplicate something here — `MU_0` previously existed in three places with two different
//! spellings.

/// Speed of light in vacuum (m/s). Exact by definition of the metre.
pub const SPEED_OF_LIGHT_M_PER_S: f64 = 299_792_458.0;

/// Metres per inch. Exact by definition of the inch.
pub const METERS_PER_INCH: f64 = 0.0254;

/// Speed of light in inches per nanosecond ≈ 11.8029.
///
/// Derived, not written out. An earlier version of this file stored `11.803` under a doc
/// comment that said "Exact: … = 11.80285", which was a 0.0013% lie; deriving it removes the
/// possibility. Note Saturn uses `11.8` here, which is why our propagation delays run about
/// 0.025% below its.
pub const SPEED_OF_LIGHT_IN_NS: f64 = SPEED_OF_LIGHT_M_PER_S / METERS_PER_INCH * 1e-9;

/// Permeability of free space µ₀ (H/m).
pub const MU_0: f64 = 4.0 * std::f64::consts::PI * 1e-7;

/// Free-space wave impedance η₀ = µ₀·c (Ω).
///
/// Derived, so it agrees with `MU_0` by construction: 376.7303…. Used by the
/// Hammerstad-Jensen air-microstrip impedance Z01(u). Saturn uses 120π ≈ 376.9911 instead,
/// which is where the ~0.069% offset in our Zo comes from.
pub const ETA_0: f64 = MU_0 * SPEED_OF_LIGHT_M_PER_S;

/// One mil in metres. Derived from [`METERS_PER_INCH`].
pub const MIL_TO_M: f64 = METERS_PER_INCH / 1000.0;

/// Copper resistivity at 20 °C (Ω·m).
///
/// Stored in SI because that is what the skin-depth calculation needs. The Ω·cm form this
/// replaced had no callers. [`crate::resistance::COPPER_RESISTIVITY_OHM_MIL`] is derived
/// from this one rather than typed again.
pub const COPPER_RESISTIVITY_OHM_M: f64 = 1.724e-8;

/// The temperature at which [`COPPER_RESISTIVITY_OHM_M`] is quoted (°C).
pub const ROOM_TEMP_C: f64 = 20.0;

/// Thermal coefficient of resistance for copper, per °C.
///
/// An earlier version of this file carried this constant with **no callers at all** and it
/// was deleted in the 2026-10-08 cleanup on exactly that basis. It is back because
/// [`crate::resistance::temperature_factor`] is the caller it was missing, and because
/// `current.rs` was carrying the same 0.00393 inline — the duplication this module exists to
/// prevent. Nothing here is kept speculatively; a constant that loses its last caller should
/// be deleted again.
pub const COPPER_TEMP_COEFF: f64 = 0.00393;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// The derived speed of light must match the hand-computed value, and must *not* be the
    /// rounded 11.803 that used to live here.
    #[test]
    fn speed_of_light_in_ns_is_exact() {
        assert_relative_eq!(
            SPEED_OF_LIGHT_IN_NS,
            11.802_852_677_165_355,
            max_relative = 1e-15
        );
        assert!(
            (SPEED_OF_LIGHT_IN_NS - 11.803).abs() > 1e-6,
            "should be the exact value, not the rounded one"
        );
    }

    /// η₀ must agree with the textbook figure and with √(µ₀/ε₀).
    #[test]
    fn eta_0_is_consistent() {
        assert_relative_eq!(ETA_0, 376.730_313_668, max_relative = 1e-9);
        // √(µ₀/ε₀) is the same quantity by a different route.
        let epsilon_0 = 1.0 / (MU_0 * SPEED_OF_LIGHT_M_PER_S * SPEED_OF_LIGHT_M_PER_S);
        assert_relative_eq!((MU_0 / epsilon_0).sqrt(), ETA_0, max_relative = 1e-12);
    }

    #[test]
    fn length_conversions_are_consistent() {
        assert_relative_eq!(MIL_TO_M, 2.54e-5, max_relative = 1e-15);
        assert_relative_eq!(MIL_TO_M * 1000.0, METERS_PER_INCH, max_relative = 1e-15);
    }

    /// µ₀ as 4π×1e-7 must match the measured-value spelling it replaced.
    #[test]
    fn mu_0_matches_the_tabulated_value() {
        assert_relative_eq!(MU_0, 1.256_637_061_435_9e-6, max_relative = 1e-12);
    }
}
