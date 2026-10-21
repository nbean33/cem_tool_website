//! Physical constants, including the one that must *not* be the physically correct value.

/// Speed of light in m/s, as physics defines it.
///
/// Provided for reference and for any post-processing that should be physically exact. It is
/// **not** what converts frequency to wavelength in this engine — see [`CVEL_MHZ_M`].
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// NEC-2's frequency-to-wavelength constant, in MHz·metres: `wavelength_m = CVEL_MHZ_M / f_MHz`.
///
/// NEC-2 uses `299.8`, not `299.792458`. This was confirmed empirically rather than assumed:
/// running the reference engine (`nec2dxs3k0.exe`, build 2.7) on a 300 MHz model reports
///
/// ```text
/// FREQUENCY= 3.0000E+02 MHZ
/// WAVELENGTH= 9.9933E-01 METERS
/// ```
///
/// and `299.8 / 300 = 0.999333…` matches, while `299.792458 / 300 = 0.999308…` does not.
///
/// Keeping NEC-2's value is a deliberate choice. Substituting the true speed of light shifts
/// every electrical length by about 0.0085%, which is far too small to notice by eye but large
/// enough to show up as a systematic bias when differential-testing impedance against the
/// reference engine. Physical exactness is not the goal here; agreement with the oracle is.
///
/// The value is NEC's literal exactly as Fortran stores it: `DATA CVEL/299.8/` has no `D`
/// exponent, so it is single precision, `299.79998779…`, widened. The 4e-8 difference is invisible
/// in impedance, but not in the phase of a pattern computed at a range: `RP` adds
/// `−360°·frac(R/λ)`, and at 1000 m and 300 MHz the exact `299.8` puts that phase 0.015° off the
/// engine's — a field difference of 2.9e-4. With the single-precision value it is 4e-5.
#[allow(clippy::cast_lossless)]
pub const CVEL_MHZ_M: f64 = 299.8_f32 as f64;

/// Permeability of free space, H/m.
pub const MU0: f64 = 4.0e-7 * core::f64::consts::PI;

/// Permittivity of free space, F/m, derived from [`MU0`] and [`SPEED_OF_LIGHT`].
pub const EPSILON0: f64 = 1.0 / (MU0 * SPEED_OF_LIGHT * SPEED_OF_LIGHT);

/// Intrinsic impedance of free space, ohms (`sqrt(MU0 / EPSILON0)`), ~376.73.
///
/// NEC-2 carries this as a literal rather than deriving it; if differential tests show a
/// systematic gain offset, check this against the value in the reference engine.
pub const ETA0: f64 = 376.730_313_461_770_7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nec_wavelength_matches_reference_engine_output() {
        // The reference engine printed WAVELENGTH= 9.9933E-01 METERS at 300 MHz.
        let lambda = CVEL_MHZ_M / 300.0;
        assert!(
            (lambda - 0.999_333).abs() < 5e-7,
            "expected ~0.999333 m, got {lambda}"
        );

        // And the physically correct constant would *not* round to that printed value.
        let physical = (SPEED_OF_LIGHT / 1e6) / 300.0;
        assert!(
            (physical - 0.999_333).abs() > 2e-5,
            "physical c should be distinguishable from NEC's constant"
        );
    }

    #[test]
    fn free_space_impedance_is_consistent_with_mu0_and_eps0() {
        let derived = (MU0 / EPSILON0).sqrt();
        assert!(
            (derived - ETA0).abs() < 1e-9,
            "ETA0 {ETA0} inconsistent with derived {derived}"
        );
    }
}
