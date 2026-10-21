//! Complex arithmetic with Fortran's semantics, and NEC's single-precision literals.
//!
//! `num_complex` and gfortran agree on `+`, `−`, `×` and on scaling by a real, but not on the
//! rest. gfortran compiles complex division with Smith's algorithm (`-fcx-fortran-rules`), and
//! its `SQRT` is the conventional `√((|x| + |z|)/2)` formula rather than a polar
//! decomposition. The Sommerfeld routines lean on both, so they are written here once and used
//! everywhere a Fortran expression divides by, or takes the root of, a complex number.

use num_complex::Complex64;

/// A Fortran literal written without a `D` exponent: stored in single precision, then widened.
///
/// `DATA CK2/6.283185308/` holds `6.2831854820…`, not `6.283185308`. Porting such a literal as
/// the f64 it looks like would differ from NEC in the eighth digit.
#[must_use]
#[allow(clippy::cast_lossless)]
pub const fn single(x: f32) -> f64 {
    x as f64
}

/// `a / b` by Smith's algorithm, as gfortran evaluates complex division.
#[must_use]
pub fn cdiv(a: Complex64, b: Complex64) -> Complex64 {
    if b.re.abs() < b.im.abs() {
        let ratio = b.re / b.im;
        let div = b.re * ratio + b.im;
        Complex64::new((a.re * ratio + a.im) / div, (a.im * ratio - a.re) / div)
    } else {
        let ratio = b.im / b.re;
        let div = b.im * ratio + b.re;
        Complex64::new((a.im * ratio + a.re) / div, (a.im - a.re * ratio) / div)
    }
}

/// `x / b` for real `x` — Fortran promotes `x` to complex and divides.
#[must_use]
pub fn rdiv(x: f64, b: Complex64) -> Complex64 {
    cdiv(Complex64::new(x, 0.0), b)
}

/// Principal square root, real part non-negative, branch cut on the negative real axis.
#[must_use]
pub fn csqrt(z: Complex64) -> Complex64 {
    if z.re == 0.0 && z.im == 0.0 {
        return Complex64::new(0.0, z.im);
    }
    let t = (0.5 * (z.re.abs() + z.re.hypot(z.im))).sqrt();
    if z.re >= 0.0 {
        Complex64::new(t, z.im / (2.0 * t))
    } else {
        Complex64::new(z.im.abs() / (2.0 * t), t.copysign(z.im))
    }
}

/// `|z|²` as Fortran computes `DREAL(Z*DCONJG(Z))`.
#[must_use]
pub fn norm_sqr(z: Complex64) -> f64 {
    z.re * z.re + z.im * z.im
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smith_division_agrees_with_the_definition() {
        let a = Complex64::new(1.5, -2.25);
        for b in [
            Complex64::new(3.0, 0.5),
            Complex64::new(0.25, -4.0),
            Complex64::new(-2.0, 2.0),
        ] {
            let q = cdiv(a, b);
            assert!((q * b - a).norm() < 1e-14, "{q} * {b} != {a}");
        }
    }

    #[test]
    fn square_root_is_principal() {
        for z in [
            Complex64::new(4.0, 0.0),
            Complex64::new(-4.0, 0.0),
            Complex64::new(-4.0, -0.0),
            Complex64::new(3.0, -4.0),
            Complex64::new(-3.0, 4.0),
        ] {
            let r = csqrt(z);
            assert!(r.re >= 0.0);
            assert!((r * r - z).norm() < 1e-14, "sqrt({z}) = {r}");
        }
        assert!(
            (csqrt(Complex64::new(-4.0, -0.0)).im + 2.0).abs() < 1e-15,
            "sign of zero picks the side"
        );
    }

    #[test]
    #[allow(clippy::excessive_precision)] // NEC's literal, verbatim
    fn single_literals_widen() {
        assert!((single(6.283_185_308) - 6.283_185_482_025_146_5).abs() < 1e-15);
    }
}
