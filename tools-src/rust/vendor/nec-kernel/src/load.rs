//! Segment loading (`LOAD`, `ZINT`).
//!
//! A load is an impedance distributed along a segment. NEC stores it as impedance per unit
//! length, with length in wavelengths (`ZARRAY`), and subtracts `(A + C)·Z'` from the matrix row
//! of the loaded segment for every basis function touching it (`CMSET`) — the tangential field
//! a current of `A + C` at the segment centre produces across that impedance.
//!
//! | `LD` type | meaning | fields |
//! |---|---|---|
//! | 0 | series RLC, lumped | R (Ω), L (H), C (F) |
//! | 1 | parallel RLC, lumped | R, L, C |
//! | 2 | series RLC, per metre | R (Ω/m), L (H/m), C (F/m) |
//! | 3 | parallel RLC, per metre | R, L, C |
//! | 4 | fixed impedance, lumped | R, X (Ω) |
//! | 5 | wire conductivity | σ (S/m) |
//!
//! A zero `C` (or `L`, `R` in the parallel forms) means the element is absent, not a short.

use crate::cmath::{cdiv, rdiv};
use num_complex::Complex64;

type C = Complex64;

/// `j·2π·c`, with NEC's `c = 299.8e6`: `DATA TPCJX/0.,1.883698955D+9/`. Multiplied by an
/// inductance and divided by the wavelength in metres, it gives `jωL`.
const TPCJ: C = C::new(0.0, 1.883_698_955e9);

/// Resistance, inductance and capacitance as written on an `LD` card — per element for the
/// lumped types, per metre for the distributed ones. Zero means the element is absent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rlc {
    /// Resistance, Ω (or Ω/m).
    pub r: f64,
    /// Inductance, H (or H/m).
    pub l: f64,
    /// Capacitance, F (or F/m).
    pub c: f64,
}

/// The kind of load, with its values as written on the `LD` card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadKind {
    /// `LD 0`: series R, L, C in ohms, henrys, farads.
    SeriesRlc(Rlc),
    /// `LD 1`: parallel R, L, C.
    ParallelRlc(Rlc),
    /// `LD 2`: series R, L, C per metre.
    SeriesRlcPerMetre(Rlc),
    /// `LD 3`: parallel R, L, C per metre.
    ParallelRlcPerMetre(Rlc),
    /// `LD 4`: a fixed impedance `R + jX`, ohms.
    Impedance {
        /// Resistance, Ω.
        r: f64,
        /// Reactance, Ω.
        x: f64,
    },
    /// `LD 5`: wire of conductivity `sigma`, S/m.
    Conductivity {
        /// Conductivity, S/m.
        sigma: f64,
    },
}

impl LoadKind {
    /// `LOAD`: the load on one segment as impedance per unit length in wavelengths (`ZT`).
    ///
    /// `si` and `bi` are the segment length and radius in wavelengths, `wlam` the wavelength in
    /// metres.
    #[must_use]
    pub fn per_unit_length(&self, si: f64, bi: f64, wlam: f64) -> C {
        let present = |x: f64| x.abs() > 1e-20;
        match *self {
            Self::SeriesRlc(Rlc { r, l, c }) => {
                let mut zt = r / si + TPCJ * l / (si * wlam);
                if present(c) {
                    zt += rdiv(wlam, TPCJ * si * c);
                }
                zt
            }
            Self::ParallelRlc(Rlc { r, l, c }) => {
                let mut zt = TPCJ * si * c / wlam;
                if present(l) {
                    zt += rdiv(si * wlam, TPCJ * l);
                }
                if present(r) {
                    zt += si / r;
                }
                rdiv(1.0, zt)
            }
            Self::SeriesRlcPerMetre(Rlc { r, l, c }) => {
                let mut zt = r * wlam + TPCJ * l;
                if present(c) {
                    zt += rdiv(1.0, TPCJ * si * si * c);
                }
                zt
            }
            Self::ParallelRlcPerMetre(Rlc { r, l, c }) => {
                let mut zt = TPCJ * si * si * c;
                if present(l) {
                    zt += rdiv(1.0, TPCJ * l);
                }
                if present(r) {
                    zt += 1.0 / (r * wlam);
                }
                rdiv(1.0, zt)
            }
            Self::Impedance { r, x } => C::new(r, x) / si,
            Self::Conductivity { sigma } => zint(sigma * wlam, bi),
        }
    }
}

/// `ZINT`: internal impedance per unit length of a round wire, from Kelvin (`ber`/`bei`)
/// functions — polynomial fits below `x = 8`, asymptotic forms above, the high-frequency limit
/// beyond 110. `sigl` is conductivity × wavelength, `rolam` the radius in wavelengths.
///
/// The constants are NEC's truncated ones (`PI = 3.1415926`, `CN = 0.70710678`), kept as written.
#[must_use]
#[allow(
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::approx_constant,
    clippy::excessive_precision
)]
pub fn zint(sigl: f64, rolam: f64) -> C {
    const PI: f64 = 3.141_592_6;
    const POT: f64 = 1.570_796_3;
    const TP: f64 = 6.283_185_3;
    const TPCMU: f64 = 2.368_705e3;
    const CMOTP: f64 = 60.0;
    let fj = C::new(0.0, 1.0);
    let cn = C::new(0.707_106_78, 0.707_106_78);
    let cc = [
        C::new(6e-7, 1.9e-6),
        C::new(-3.4e-6, 5.1e-6),
        C::new(-2.52e-5, 0.0),
        C::new(-9.06e-5, -9.01e-5),
        C::new(0.0, -9.765e-4),
        C::new(0.011_048_6, -0.011_048_5),
        C::new(0.0, -0.392_699_1),
        C::new(1.6e-6, -3.2e-6),
        C::new(1.17e-5, -2.4e-6),
        C::new(3.46e-5, 3.38e-5),
        C::new(5e-7, 2.452e-4),
        C::new(-1.3813e-3, 1.3811e-3),
        C::new(-6.250_01e-2, -1e-7),
        C::new(0.707_106_8, 0.707_106_8),
    ];
    let poly = |c: &[C], d: f64| c[1..].iter().fold(c[0], |acc, &k| acc * d + k);
    let th = |d: f64| poly(&cc[0..7], d);
    let ph = |d: f64| poly(&cc[7..14], d);

    let x = (TPCMU * sigl).sqrt() * rolam;
    let br1 = if x > 110.0 {
        C::new(0.707_106_78, -0.707_106_78)
    } else if x > 8.0 {
        let f = (POT / x).sqrt() * (-(cn * x) + th(-8.0 / x)).exp();
        let g = (cn * x + th(8.0 / x)).exp() / (TP * x).sqrt();
        let br2 = fj * f / PI;
        let br1 = g + br2;
        let br2 = g * ph(8.0 / x) - br2 * ph(-8.0 / x);
        cdiv(br1, br2)
    } else {
        let y = x / 8.0;
        let y = y * y;
        let s = y * y;
        let ber = ((((((-9.01e-6 * s + 1.22552e-3) * s - 0.083_496_09) * s + 2.641_914) * s
            - 32.363_456)
            * s
            + 113.777_78)
            * s
            - 64.0)
            * s
            + 1.0;
        let bei = ((((((1.1346e-4 * s - 0.011_036_67) * s + 0.521_856_15) * s - 10.567_658) * s
            + 72.817_777)
            * s
            - 113.777_78)
            * s
            + 16.0)
            * y;
        let br1 = C::new(ber, bei);
        let ber = (((((((-3.94e-6 * s + 4.5957e-4) * s - 0.026_092_53) * s + 0.660_478_49) * s
            - 6.068_148_1)
            * s
            + 14.222_222)
            * s
            - 4.0)
            * y)
            * x;
        let bei = ((((((4.609e-5 * s - 3.79386e-3) * s + 0.146_772_04) * s - 2.311_675_1) * s
            + 11.377_778)
            * s
            - 10.666_667)
            * s
            + 0.5)
            * x;
        cdiv(br1, C::new(ber, bei))
    };
    fj * (CMOTP / sigl).sqrt() * br1 / rolam
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copper_wire_approaches_the_skin_effect_limit() {
        // At HF a copper wire is deep in the skin-effect regime: R' = X' = Rs/(2πa), with
        // Rs = sqrt(ωμ0/(2σ)). 14 MHz, 1 mm radius: Rs = 9.76e-4 Ω, R' = 0.155 Ω/m.
        let lambda = 299.8 / 14.0;
        let a = 1e-3;
        let z = zint(5.8e7 * lambda, a / lambda) / lambda; // ohms per metre
        let rs = (2.0 * std::f64::consts::PI * 14e6 * 4e-7 * std::f64::consts::PI / (2.0 * 5.8e7)).sqrt();
        let expect = rs / (2.0 * std::f64::consts::PI * a);
        assert!((z.re - expect).abs() < 0.01 * expect, "R' = {} vs {expect}", z.re);
        assert!((z.im - expect).abs() < 0.01 * expect, "X' = {} vs {expect}", z.im);
    }

    #[test]
    fn a_lumped_series_resistor_is_spread_over_the_segment() {
        // 50 ohms on a 0.1-wavelength segment is 500 ohms per wavelength.
        let z = LoadKind::SeriesRlc(Rlc { r: 50.0, l: 0.0, c: 0.0 }).per_unit_length(0.1, 1e-4, 10.0);
        assert!((z - C::new(500.0, 0.0)).norm() < 1e-9, "{z}");
    }

    #[test]
    fn parallel_lc_is_open_at_resonance() {
        // L = 1 uH, C = 1/(w^2 L) at 10 MHz: the admittance cancels and the impedance diverges.
        let wlam = 299.8 / 10.0;
        let w = 1.883_698_955e9 / wlam;
        let l = 1e-6;
        let c = 1.0 / (w * w * l);
        let z = LoadKind::ParallelRlc(Rlc { r: 0.0, l, c }).per_unit_length(0.01, 1e-4, wlam);
        assert!(z.norm() > 1e9, "{z}");
    }
}
