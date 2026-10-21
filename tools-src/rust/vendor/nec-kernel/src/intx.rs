//! Adaptive Romberg integration of `exp(-jkR)/(kR)` along a segment (`INTX`, `GF`, `TEST`).
//!
//! This is the only numerical integral in the thin-wire kernel: the field of a *constant*
//! current filament has no closed form, whereas sine and cosine currents do (see `eksc`).
//!
//! The algorithm is reproduced step for step rather than replaced by a "better" quadrature, so
//! that the same points are sampled and the result agrees with the engine to rounding rather than
//! merely to the 1e-4 convergence tolerance. Variable names follow the Fortran where that helps
//! cross-checking.
//!
//! All arguments are in units of `k·distance` (radians), because `EKSC` passes `k·z`, `k·ρ`.

/// Integrand parameters, NEC's `COMMON /TMI/ ZPK, RKB2, IJ`.
#[derive(Debug, Clone, Copy)]
pub struct Tmi {
    /// Observation point's axial position along the source, times `k`.
    pub zpk: f64,
    /// `(k·ρ)²`, the squared radial distance.
    pub rkb2: f64,
    /// Whether this is the self term (observation on the source segment). NEC's `IJ == 0`.
    pub self_term: bool,
}

/// `GF`: the integrand `exp(jkR)/(kR)`, split into its cosine and sine parts.
///
/// For the self term the cosine part is `(cos(kR) − 1)/(kR)`: subtracting 1 removes the `1/R`
/// singularity, which `INTX` then restores analytically. Below `kR = 0.2` that difference is
/// evaluated by series to avoid cancellation.
fn gf(zk: f64, t: &Tmi) -> (f64, f64) {
    let zdk = zk - t.zpk;
    let rk = (t.rkb2 + zdk * zdk).sqrt();
    let si = rk.sin() / rk;
    let co = if !t.self_term {
        rk.cos() / rk
    } else if rk < 0.2 {
        let rks = rk * rk;
        ((-1.388_888_89e-3 * rks + 4.166_666_67e-2) * rks - 0.5) * rk
    } else {
        (rk.cos() - 1.0) / rk
    };
    (co, si)
}

/// `TEST`: relative change between two estimates, separately for real and imaginary parts,
/// normalized by the larger magnitude of the newer estimate (or `dmin`).
pub(crate) fn test(f1r: f64, f2r: f64, f1i: f64, f2i: f64, dmin: f64) -> (f64, f64) {
    let den = f2r.abs().max(f2i.abs()).max(dmin);
    if den < 1e-37 {
        return (0.0, 0.0);
    }
    ((f1r - f2r).abs() / den, (f1i - f2i).abs() / den)
}

/// `INTX`: integrate `GF` from `el1` to `el2` by variable-interval Romberg integration.
///
/// Returns `(∫ cos-part, ∫ sin-part)`. For the self term only `[el1, 0]` is integrated and the
/// result doubled by symmetry, then the singular part `2·ln((√(b²+s²) + s)/b)` is added back,
/// where `b = k·a` is the radius term and `s` the half-length.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::too_many_lines)]
pub fn intx(el1: f64, el2: f64, b: f64, t: &Tmi) -> (f64, f64) {
    const NX: i64 = 1;
    const NM: i64 = 65_536;
    const NTS: i64 = 4;
    const RX: f64 = 1.0e-4;

    let mut z = el1;
    let ze = if t.self_term { 0.0 } else { el2 };
    let s = ze - z;
    #[allow(clippy::cast_precision_loss)]
    let ep = s / (10.0 * NM as f64);
    let zend = ze - ep;

    let (mut sgr, mut sgi) = (0.0, 0.0);
    let mut ns: i64 = NX;
    let mut nt: i64 = 0;
    let (mut g1r, mut g1i) = gf(z, t);

    // Label 1 of the Fortran: start a step of width s/ns.
    'step: loop {
        #[allow(clippy::cast_precision_loss)]
        let mut dz = s / ns as f64;
        if z + dz > ze {
            dz = ze - z;
            if dz.abs() <= ep {
                break 'step;
            }
        }
        let mut dzot = dz * 0.5;
        let (mut g3r, mut g3i) = gf(z + dzot, t);
        let (mut g5r, mut g5i) = gf(z + dz, t);

        // Label 4: refine the current step until it converges.
        let (add_r, add_i) = loop {
            let t00r = (g1r + g5r) * dzot;
            let t00i = (g1i + g5i) * dzot;
            let t01r = (t00r + dz * g3r) * 0.5;
            let t01i = (t00i + dz * g3i) * 0.5;
            let t10r = (4.0 * t01r - t00r) / 3.0;
            let t10i = (4.0 * t01i - t00i) / 3.0;

            // Three-point Romberg converged?
            let (te1r, te1i) = test(t01r, t10r, t01i, t10i, 0.0);
            if te1i <= RX && te1r <= RX {
                nt += 2;
                break (t10r, t10i);
            }

            let (g2r, g2i) = gf(z + dz * 0.25, t);
            let (g4r, g4i) = gf(z + dz * 0.75, t);
            let t02r = (t01r + dzot * (g2r + g4r)) * 0.5;
            let t02i = (t01i + dzot * (g2i + g4i)) * 0.5;
            let t11r = (4.0 * t02r - t01r) / 3.0;
            let t11i = (4.0 * t02i - t01i) / 3.0;
            let t20r = (16.0 * t11r - t10r) / 15.0;
            let t20i = (16.0 * t11i - t10i) / 15.0;

            // Five-point Romberg converged?
            let (te2r, te2i) = test(t11r, t20r, t11i, t20i, 0.0);
            if te2i <= RX && te2r <= RX {
                nt += 1;
                break (t20r, t20i);
            }

            // Not converged: halve the step, reusing the samples already taken (label 14/16).
            nt = 0;
            if ns >= NM {
                // NEC prints "STEP SIZE LIMITED AT Z=" and accepts the five-point value.
                nt += 1;
                break (t20r, t20i);
            }
            ns *= 2;
            #[allow(clippy::cast_precision_loss)]
            {
                dz = s / ns as f64;
            }
            dzot = dz * 0.5;
            g5r = g3r;
            g5i = g3i;
            g3r = g2r;
            g3i = g2i;
        };

        // Label 8/9/10: accept and advance.
        sgr += add_r;
        sgi += add_i;
        z += dz;
        if z >= zend {
            break 'step;
        }
        g1r = g5r;
        g1i = g5i;
        // Label 11/12/13: after NTS consecutive easy steps, try doubling the step size.
        if nt >= NTS && ns > NX {
            ns /= 2;
            nt = 1;
        }
    }

    if t.self_term {
        // Restore the singular part removed in GF, and double for the other half.
        sgr = 2.0 * (sgr + (((b * b + s * s).sqrt() + s) / b).ln());
        sgi *= 2.0;
    }
    (sgr, sgi)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Direct reference integral by a fine composite Simpson rule, for checking `intx`.
    fn simpson(el1: f64, el2: f64, t: &Tmi) -> (f64, f64) {
        let n = 200_000;
        #[allow(clippy::cast_precision_loss)]
        let h = (el2 - el1) / n as f64;
        let mut sr = 0.0;
        let mut si = 0.0;
        for k in 0..=n {
            #[allow(clippy::cast_precision_loss)]
            let x = el1 + h * k as f64;
            let w = if k == 0 || k == n {
                1.0
            } else if k % 2 == 1 {
                4.0
            } else {
                2.0
            };
            let (c, s) = gf(x, t);
            sr += w * c;
            si += w * s;
        }
        (sr * h / 3.0, si * h / 3.0)
    }

    #[test]
    fn off_diagonal_integral_matches_a_fine_reference_quadrature() {
        let t = Tmi {
            zpk: 0.7,
            rkb2: 0.3_f64.powi(2),
            self_term: false,
        };
        let (r, i) = intx(-0.17, 0.17, 0.3, &t);
        let (rr, ri) = simpson(-0.17, 0.17, &t);
        // INTX converges to a relative 1e-4 per step; it should comfortably beat that here.
        assert!((r - rr).abs() < 1e-6 * rr.abs(), "real {r} vs {rr}");
        assert!((i - ri).abs() < 1e-6 * ri.abs(), "imag {i} vs {ri}");
    }

    #[test]
    fn self_term_restores_the_singular_part() {
        // For the self term INTX integrates (cos R - 1)/R over half the segment, doubles it,
        // and adds 2*ln((sqrt(b^2+s^2)+s)/b). The total must equal the full integral of
        // cos(R)/R, which a fine quadrature can evaluate directly since b > 0.
        let b = 0.01;
        let s = 0.17;
        let t_self = Tmi {
            zpk: 0.0,
            rkb2: b * b,
            self_term: true,
        };
        let (r, i) = intx(-s, s, b, &t_self);
        let t_full = Tmi {
            zpk: 0.0,
            rkb2: b * b,
            self_term: false,
        };
        let (rr, ri) = simpson(-s, s, &t_full);
        // INTX converges each step to a relative 1e-4 (its RX). Here it lands about 6e-6 from the
        // exact value — NEC's own accuracy, reproduced deliberately — so the check is held to the
        // algorithm's contract rather than to a tolerance it never promised.
        assert!((r - rr).abs() < 1e-4 * rr.abs(), "real {r} vs {rr}");
        assert!((i - ri).abs() < 1e-4 * ri.abs(), "imag {i} vs {ri}");
    }

    #[test]
    fn series_branch_agrees_with_the_direct_formula_at_its_boundary() {
        let t = Tmi {
            zpk: 0.0,
            rkb2: 0.0,
            self_term: true,
        };
        let (series, _) = gf(0.199_999, &t);
        let direct = ((0.199_999_f64).cos() - 1.0) / 0.199_999;
        assert!((series - direct).abs() < 1e-9, "{series} vs {direct}");
    }
}
