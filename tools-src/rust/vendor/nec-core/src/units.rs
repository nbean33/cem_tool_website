//! Frequency and wavelength.
//!
//! NEC works internally in wavelengths, not metres. Keeping that conversion in one typed place
//! stops it from being open-coded (and silently varied) across the solver.

use crate::constants::CVEL_MHZ_M;
use crate::{CoreError, Result};

/// A frequency, stored in MHz because that is the unit every NEC card uses.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Frequency {
    mhz: f64,
}

impl Frequency {
    /// Construct from MHz, rejecting non-positive and non-finite values.
    pub fn from_mhz(mhz: f64) -> Result<Self> {
        if !mhz.is_finite() || mhz <= 0.0 {
            return Err(CoreError::InvalidFrequency(mhz));
        }
        Ok(Self { mhz })
    }

    /// Frequency in MHz.
    #[must_use]
    pub const fn mhz(self) -> f64 {
        self.mhz
    }

    /// Free-space wavelength in metres, using NEC-2's constant (see
    /// [`CVEL_MHZ_M`]).
    #[must_use]
    pub fn wavelength_m(self) -> f64 {
        CVEL_MHZ_M / self.mhz
    }

    /// Free-space wavenumber `k = 2*pi / lambda`, in radians per metre.
    #[must_use]
    pub fn wavenumber_per_m(self) -> f64 {
        core::f64::consts::TAU / self.wavelength_m()
    }
}

/// A length expressed in wavelengths at some frequency.
///
/// NEC scales geometry into wavelengths before filling the interaction matrix. This newtype
/// exists so a metres value cannot be passed where wavelengths are expected.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Wavelengths(pub f64);

impl Wavelengths {
    /// Convert a length in metres at the given frequency into wavelengths.
    #[must_use]
    pub fn from_metres(metres: f64, f: Frequency) -> Self {
        Self(metres / f.wavelength_m())
    }

    /// Convert back to metres at the given frequency.
    #[must_use]
    pub fn to_metres(self, f: Frequency) -> f64 {
        self.0 * f.wavelength_m()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nonpositive_and_nonfinite() {
        assert!(Frequency::from_mhz(0.0).is_err());
        assert!(Frequency::from_mhz(-1.0).is_err());
        assert!(Frequency::from_mhz(f64::NAN).is_err());
        assert!(Frequency::from_mhz(f64::INFINITY).is_err());
        assert!(Frequency::from_mhz(14.05).is_ok());
    }

    #[test]
    fn wavelength_matches_reference_engine() {
        let f = Frequency::from_mhz(300.0).expect("valid");
        assert!((f.wavelength_m() - 0.999_333).abs() < 5e-7);
    }

    #[test]
    fn metres_wavelengths_round_trip() {
        let f = Frequency::from_mhz(14.05).expect("valid");
        let metres = 10.106;
        let back = Wavelengths::from_metres(metres, f).to_metres(f);
        assert!((back - metres).abs() < 1e-12);
    }

    #[test]
    fn example1_dipole_is_about_half_a_wavelength() {
        // models/Example1.nec: a dipole from -0.2418 to +0.2418 m at 300 MHz.
        let f = Frequency::from_mhz(300.0).expect("valid");
        let total = Wavelengths::from_metres(0.2418 * 2.0, f);
        assert!(
            (total.0 - 0.4838).abs() < 1e-3,
            "expected ~0.484 wavelengths, got {}",
            total.0
        );
    }
}
