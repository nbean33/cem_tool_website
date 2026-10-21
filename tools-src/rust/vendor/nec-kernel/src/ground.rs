//! The ground under the antenna, as the matrix fill sees it at one frequency.
//!
//! NEC-2 selects ground with the `GN` card (`IPERF`), never with `GE` — `GE` only decides which
//! wire ends are bonded to the ground plane, which is connectivity and lives in `nec-geom`.
//!
//! Every ground option adds the field of the structure's *image* below `z = 0` (`EFLD`'s second
//! pass, `KSYMP = 2`). They differ in how that image is weighted:
//!
//! * **Perfect** (`IPERF = 1`): the image as it stands, a perfect mirror.
//! * **Reflection coefficient** (`IPERF = 0`): the image scaled by plane-wave Fresnel
//!   coefficients evaluated at the specular angle — an approximation good for structures not too
//!   close to the ground. Optionally with a radial-wire ground screen modifying the surface
//!   impedance under the antenna.
//! * **Sommerfeld** (`IPERF = 2`): the image weighted by `(ε_c − 1)/(ε_c + 1)`, plus the rest of
//!   the ground's field from the Sommerfeld integrals — interpolated from a precomputed grid near
//!   the source, Norton's ground-wave formulas further away. See [`crate::sommerfeld`].

use crate::cmath::{cdiv, csqrt, rdiv};
use crate::consts::ETA;
use crate::sommerfeld::SommerfeldGround;
use num_complex::Complex64;

/// `59.96` as NEC writes it — `η₀/(2π)`, to convert conductivity to the imaginary part of the
/// relative permittivity: `ε_c = ε_r − j·σ·λ·59.96`. Written without a `D` exponent, so it is a
/// single-precision literal widened to double, like [`ETA`].
#[allow(clippy::cast_lossless)]
pub const SIG_TO_EPS: f64 = 59.96_f32 as f64;

/// `2367.067`: the radial-screen impedance constant, `T1 = j·2367.067 / N` in NEC's main program.
pub const SCREEN_CONSTANT: f64 = 2367.067;

/// A radial-wire ground screen centred on the origin, in wavelength units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadialScreen {
    /// `T1 = j·2367.067 / N`.
    pub t1: Complex64,
    /// `T2 = a·N`, wire radius times the number of radials, in wavelengths.
    pub t2: f64,
    /// Radial length in wavelengths (`SCRWL`).
    pub length: f64,
}

impl RadialScreen {
    /// A screen of `count` radials of `length_m` and wire radius `radius_m`, at wavelength
    /// `wavelength_m`.
    #[must_use]
    pub fn new(count: u32, length_m: f64, radius_m: f64, wavelength_m: f64) -> Self {
        let n = f64::from(count);
        Self {
            t1: Complex64::new(0.0, SCREEN_CONSTANT / n),
            t2: radius_m / wavelength_m * n,
            length: length_m / wavelength_m,
        }
    }
}

/// The ground model for one frequency.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Ground {
    /// No ground (`KSYMP = 1`).
    #[default]
    FreeSpace,
    /// A perfectly conducting plane at `z = 0`.
    Perfect,
    /// Finite ground by the reflection-coefficient approximation.
    Reflection {
        /// `ZRATI = 1/√ε_c`, the ground's wave impedance relative to free space.
        zrati: Complex64,
        /// A radial-wire screen, if one is specified.
        screen: Option<RadialScreen>,
    },
    /// Finite ground by the Sommerfeld integrals.
    Sommerfeld(SommerfeldGround),
}

impl Ground {
    /// Reflection-coefficient ground of relative permittivity `epsr` and conductivity `sigma`
    /// (S/m) at wavelength `wavelength_m`, as NEC's main program sets it up.
    #[must_use]
    pub fn reflection(epsr: f64, sigma: f64, wavelength_m: f64, screen: Option<RadialScreen>) -> Self {
        Self::Reflection {
            zrati: rdiv(1.0, csqrt(complex_permittivity(epsr, sigma, wavelength_m))),
            screen,
        }
    }
}

/// The ground as NEC's image-based routines see it: the wire H field (`HSFLD`), patch fields
/// (`UNERE`, `HINTG`) and plane-wave excitation of patches. These always reflect the structure in
/// the plane `z = 0` — never through the Sommerfeld integrals — selected by `KSYMP` and
/// `IPERF = 1` alone, so a `GN 2` or `GN 3` ground appears here as reflection coefficients.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ImageGround {
    /// No ground (`KSYMP = 1`).
    #[default]
    None,
    /// Perfect ground (`IPERF = 1`): the image negated.
    Perfect,
    /// Any finite ground: Fresnel coefficients from `ZRATI = 1/√ε_c`.
    Finite {
        /// `ZRATI`.
        zrati: Complex64,
        /// The radial-wire screen, which `HSFLD` (but not the patch routines) folds in.
        screen: Option<RadialScreen>,
    },
}

/// `ε_c = ε_r − j·σ·λ·59.96`.
#[must_use]
pub fn complex_permittivity(epsr: f64, sigma: f64, wavelength_m: f64) -> Complex64 {
    Complex64::new(epsr, -sigma * wavelength_m * SIG_TO_EPS)
}

/// Fresnel weighting of the image field, `EFLD` labels 7–9.
///
/// `src` and `obs` are the source-segment centre and observation point in wavelengths; `r` is the
/// distance from the image source to the observation point as the image pass computed it (with
/// the observation wire's radius folded in).
pub(crate) struct Reflector {
    refs: Complex64,
    refps: Complex64,
    px: f64,
    py: f64,
}

impl Reflector {
    pub(crate) fn new(
        zrati: Complex64,
        screen: Option<&RadialScreen>,
        src: [f64; 3],
        obs: [f64; 3],
        r: f64,
    ) -> Self {
        let (xi, yi, zi) = (obs[0], obs[1], obs[2]);
        let (xj, yj, zj) = (src[0], src[1], src[2]);
        let xij = xi - xj;
        let yij = yi - yj;
        let zij = zi + zj;

        let mut zratx = zrati;
        if let Some(sc) = screen {
            // The screen's impedance at the specular point, in parallel with the ground's.
            let xspec = (xi * zj + zi * xj) / (zi + zj);
            let yspec = (yi * zj + zi * yj) / (zi + zj);
            let rhospc = (xspec * xspec + yspec * yspec + sc.t2 * sc.t2).sqrt();
            if rhospc <= sc.length {
                let zscrn = sc.t1 * rhospc * (rhospc / sc.t2).ln();
                zratx = cdiv(zscrn * zrati, ETA * zrati + zscrn);
            }
        }

        let xymag = (xij * xij + yij * yij).sqrt();
        let (px, py, cth, zrsin) = if xymag > 1e-6 {
            let cth = zij / r;
            let zrsin = csqrt(Complex64::new(1.0, 0.0) - zratx * zratx * (1.0 - cth * cth));
            (-yij / xymag, xij / xymag, cth, zrsin)
        } else {
            (0.0, 0.0, 1.0, Complex64::new(1.0, 0.0))
        };
        let refs = cdiv(cth - zratx * zrsin, cth + zratx * zrsin);
        let refps = -cdiv(zratx * cth - zrsin, zratx * cth + zrsin) - refs;
        Self { refs, refps, px, py }
    }

    /// Weight one image field vector: the vertical-plane part by `REFS`, and the horizontal
    /// component perpendicular to the plane of incidence by `REFS + REFPS`.
    pub(crate) fn apply(&self, t: [Complex64; 3]) -> [Complex64; 3] {
        let epy = t[0] * self.px + t[1] * self.py;
        [
            self.refs * t[0] + self.refps * (epy * self.px),
            self.refs * t[1] + self.refps * (epy * self.py),
            self.refs * t[2],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_perfect_conductor_reflects_like_a_mirror() {
        // As sigma grows, ZRATI -> 0, so REFS -> 1 and REFPS -> 0: the image field is used
        // unweighted, exactly as the perfect-ground path does.
        let g = Ground::reflection(1.0, 1e12, 1.0, None);
        let Ground::Reflection { zrati, .. } = g else { unreachable!() };
        let r = Reflector::new(zrati, None, [0.0, 0.0, 0.3], [0.5, 0.2, 0.4], 0.9);
        assert!((r.refs - 1.0).norm() < 1e-4, "REFS {}", r.refs);
        assert!(r.refps.norm() < 1e-4, "REFPS {}", r.refps);
    }

    #[test]
    fn complex_permittivity_uses_necs_literal() {
        // models: "good ground" 13, 0.005 S/m at 14 MHz.
        let lambda = 299.8 / 14.0;
        let e = complex_permittivity(13.0, 0.005, lambda);
        assert!((e.re - 13.0).abs() < 1e-15);
        assert!((e.im + 0.005 * lambda * 59.96).abs() < 1e-6);
    }
}
