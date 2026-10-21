//! The Sommerfeld correction to a segment's field over lossy ground (`EFLD` label 13 onward,
//! `SFLDS`, `ROM2`, `GWAVE`, `FBAR`).
//!
//! Under `GN 2` the image of each source segment is weighted by `FRATI = (ε_c − 1)/(ε_c + 1)`
//! (the quasi-static image), and the remainder of the ground's field is added here:
//!
//! * **Near** (image distance² ≤ 0.95 λ²): interpolate the [`SommerfeldGrid`] at points along
//!   the source segment and integrate over it by Romberg quadrature (`ROM2` over `SFLDS`).
//! * **Far**: K. A. Norton's ground-wave formulas (`GWAVE`) for a current element lumped at the
//!   segment centre.

// NEC's literals are transcribed digit for digit, including single-precision ones written
// with more digits than an f32 holds and truncated values of π-related constants: matching
// the reference engine depends on rounding them exactly as Fortran does, not on their being
// the best approximation of what they stand for.
#![allow(clippy::excessive_precision, clippy::approx_constant)]

use crate::cmath::{cdiv, csqrt, norm_sqr, rdiv, single};
use crate::consts::{PI, TP};
use crate::efld::{CVec3, SegmentField};
use crate::intx::test;
use crate::scaled::WireSeg;
use crate::somnec::SommerfeldGrid;
use num_complex::Complex64;

type C = Complex64;
const ZERO: C = C::new(0.0, 0.0);
const ONE: C = C::new(1.0, 0.0);
const FJ: C = C::new(0.0, 1.0);

/// Sommerfeld-ground constants for one frequency.
#[derive(Debug, Clone, PartialEq)]
pub struct SommerfeldGround {
    /// `FRATI = (ε_c − 1)/(ε_c + 1)`, the weight on the quasi-static image.
    pub frati: C,
    /// `U = 1/√ε_c` (NEC's `ZRATI`, used by the Norton formulas).
    pub u: C,
    /// `U²`.
    pub u2: C,
    /// The interpolation grid.
    pub grid: std::sync::Arc<SommerfeldGrid>,
}

impl SommerfeldGround {
    /// Constants for complex permittivity `epsc` (as NEC's main program computes it), with the
    /// grid built for that ground.
    #[must_use]
    pub fn new(epsc: C, grid: std::sync::Arc<SommerfeldGrid>) -> Self {
        let u = rdiv(1.0, csqrt(epsc));
        Self {
            frati: cdiv(epsc - 1.0, epsc + 1.0),
            u,
            u2: u * u,
            grid,
        }
    }
}

/// `SFLDS`'s view of the source segment and the (displaced) observation point.
struct Sflds<'a> {
    g: &'a SommerfeldGround,
    src: &'a WireSeg,
    /// Displaced observation point `(XO, YO, ZO)`.
    obs: [f64; 3],
    /// Horizontal length of the source's unit direction, and its horizontal unit vector.
    sn: f64,
    xsn: f64,
    ysn: f64,
}

impl Sflds<'_> {
    /// `SFLDS(T, E)`: the ground field for a current element at `t` along the source segment.
    /// `interpolate` selects the grid (`ISNOR = 1`) over the Norton formulas (`ISNOR = 2`).
    /// Returns the nine components: x, y, z for constant, sine and cosine currents.
    #[allow(clippy::many_single_char_names, clippy::similar_names)]
    fn field(&self, t: f64, interpolate: bool) -> [C; 9] {
        let pot = 1.570_796_327;
        let [cabj, sabj, salpj] = self.src.dir;
        let xt = self.src.center[0] + t * cabj;
        let yt = self.src.center[1] + t * sabj;
        let zt = self.src.center[2] + t * salpj;
        let [xo, yo, zo] = self.obs;
        let mut rhx = xo - xt;
        let mut rhy = yo - yt;
        let rhs = rhx * rhx + rhy * rhy;
        let rho = rhs.sqrt();
        let (phx, phy);
        if rho > 0.0 {
            rhx /= rho;
            rhy /= rho;
            phx = -rhy;
            phy = rhx;
        } else {
            rhx = 1.0;
            rhy = 0.0;
            phx = 0.0;
            phy = 1.0;
        }
        let mut cph = rhx * self.xsn + rhy * self.ysn;
        let mut sph = rhy * self.xsn - rhx * self.ysn;
        if cph.abs() < 1e-10 {
            cph = 0.0;
        }
        if sph.abs() < 1e-10 {
            sph = 0.0;
        }
        let zph = zo + zt;
        let zphs = zph * zph;
        let r2s = rhs + zphs;
        let r2 = r2s.sqrt();
        let rk = r2 * TP;
        let xx2 = C::new(rk.cos(), -rk.sin());
        let sn = self.sn;
        let mut e = [ZERO; 9];

        if interpolate {
            let thet = if rho < 1e-12 { pot } else { (zph / rho).atan() };
            let [erv, ezv, erh, eph] = self.g.grid.interpolate(r2, thet);
            // Combine vertical and horizontal parts, convert to x, y, z, multiply by
            // exp(−jkR)/R.
            let xx2 = xx2 / r2;
            let sfac = sn * cph;
            let erh = xx2 * (salpj * erv + sfac * erh);
            let ezh = xx2 * (salpj * ezv - sfac * erv);
            let eph = sn * sph * xx2 * eph;
            e[0] = erh * rhx + eph * phx;
            e[1] = erh * rhy + eph * phy;
            e[2] = ezh;
            let rk = TP * t;
            let (s, c) = (rk.sin(), rk.cos());
            for i in 0..3 {
                e[3 + i] = e[i] * s;
                e[6 + i] = e[i] * c;
            }
            return e;
        }

        // Norton approximation, current lumped at the segment centre.
        let (mut erv, mut ezv, mut erh, mut ezh, mut eph) =
            gwave(self.g.u, self.g.u2, ZERO, xx2, 1.0, r2, 1.0, zph);
        let et = -(C::new(0.0, single(4.771_34)) * self.g.frati * xx2 / (r2s * r2));
        let er = 2.0 * et * C::new(1.0, rk);
        let et = et * C::new(1.0 - rk * rk, rk);
        let hrv = (er + et) * rho * zph / r2s;
        let hzv = (zphs * er - rhs * et) / r2s;
        let hrh = (rhs * er - zphs * et) / r2s;
        erv -= hrv;
        ezv -= hzv;
        erh += hrh;
        ezh += hrv;
        eph += et;
        erv *= salpj;
        ezv *= salpj;
        erh = erh * sn * cph;
        ezh = ezh * sn * cph;
        eph = eph * sn * sph;
        erh += erv;
        let s = self.src.length;
        e[0] = (erh * rhx + eph * phx) * s;
        e[1] = (erh * rhy + eph * phy) * s;
        e[2] = (ezv + ezh) * s;
        let sfac = PI * s;
        let sfac = sfac.sin() / sfac;
        for i in 0..3 {
            e[6 + i] = e[i] * sfac;
        }
        e
    }
}

/// The Sommerfeld part of `EFLD` for source `src` at observation point `obs` on a wire of
/// radius `ai`. `direct_k` is the constant-current field accumulated so far (direct minus
/// weighted image), which sets the integration's absolute tolerance.
pub(crate) fn ground_field(
    g: &SommerfeldGround,
    src: &WireSeg,
    obs: [f64; 3],
    ai: f64,
    direct_k: &CVec3,
) -> SegmentField {
    let [cabj, sabj, salpj] = src.dir;
    let [xj, yj, zj] = src.center;
    let [xi, yi, zi] = obs;
    let xij = xi - xj;
    let yij = yi - yj;

    let mut sn = (cabj * cabj + sabj * sabj).sqrt();
    let (xsn, ysn);
    if sn < 1e-5 {
        sn = 0.0;
        xsn = 1.0;
        ysn = 0.0;
    } else {
        xsn = cabj / sn;
        ysn = sabj / sn;
    }

    // Displace the observation point by the wire radius for the thin-wire approximation.
    let zij = zi + zj;
    let salpr = -salpj;
    let rhox = sabj * zij - salpr * yij;
    let rhoy = salpr * xij - cabj * zij;
    let rhoz = cabj * yij - sabj * xij;
    let rh = rhox * rhox + rhoy * rhoy + rhoz * rhoz;
    let o = if rh > 1e-10 {
        let mut rh = ai / rh.sqrt();
        if rhoz < 0.0 {
            rh = -rh;
        }
        [xi + rh * rhox, yi + rh * rhoy, zi + rh * rhoz]
    } else {
        [xi - ai * ysn, yi + ai * xsn, zi]
    };
    let sf = Sflds {
        g,
        src,
        obs: o,
        sn,
        xsn,
        ysn,
    };

    let r = xij * xij + yij * yij + zij * zij;
    let mut e;
    if r > single(0.95) {
        // Norton field equations and lumped current element.
        e = sf.field(0.0, false);
    } else {
        // Field from interpolation, integrated over the segment.
        let dmin = single(0.01) * (norm_sqr(direct_k[0]) + norm_sqr(direct_k[1]) + norm_sqr(direct_k[2])).sqrt();
        let shaf = 0.5 * src.length;
        e = rom2(-shaf, shaf, dmin, |t| sf.field(t, true));

        let zp = xij * cabj + yij * sabj + zij * salpr;
        let rh = r - zp * zp;
        let dmin = if rh > 1e-10 { (rh / (rh + ai * ai)).sqrt() } else { 0.0 };
        if dmin <= single(0.95) {
            // Close to the image axis: scale the components across the axis.
            let px = 1.0 - dmin;
            for base in [0, 3, 6] {
                let ter = (e[base] * cabj + e[base + 1] * sabj + e[base + 2] * salpr) * px;
                e[base] = dmin * e[base] + ter * cabj;
                e[base + 1] = dmin * e[base + 1] + ter * sabj;
                e[base + 2] = dmin * e[base + 2] + ter * salpr;
            }
        }
    }
    SegmentField {
        k: [e[0], e[1], e[2]],
        s: [e[3], e[4], e[5]],
        c: [e[6], e[7], e[8]],
    }
}

/// `ROM2`: integrate the nine field components over `[a, b]` by variable-interval Romberg
/// integration, testing convergence on the magnitude of the constant-current field.
#[allow(clippy::many_single_char_names)]
fn rom2(a: f64, b: f64, dmin: f64, f: impl Fn(f64) -> [C; 9]) -> [C; 9] {
    const NM: u32 = 65_536;
    const NTS: u32 = 4;
    const NX: u32 = 1;
    let rx = 1e-4;
    let mut z = a;
    let ze = b;
    let s = b - a;
    // NEC stops with "B LESS THAN A IN ROM2"; the callers always pass a ≤ b.
    debug_assert!(s >= 0.0);
    let ep = s / (1e4 * f64::from(NM));
    let zend = ze - ep;
    let mut sum = [ZERO; 9];
    let mut ns = NX;
    let mut nt = 0;
    let mut g1 = f(z);
    let (mut g2, mut g3, mut g4, mut g5);
    let mut t01 = [ZERO; 9];
    let mut t10 = [ZERO; 9];
    let mut t20 = [ZERO; 9];
    // NEC accumulates TMAG = TMAG + TR*TR + TI*TI, left to right.
    let mag = |v: &[C; 9]| v[..3].iter().fold(0.0, |acc, c| acc + c.re * c.re + c.im * c.im).sqrt();

    loop {
        let mut dz = s / f64::from(ns);
        if z + dz > ze {
            dz = ze - z;
            if dz <= ep {
                return sum;
            }
        }
        let mut dzot = dz * 0.5;
        g3 = f(z + dzot);
        g5 = f(z + dz);
        loop {
            // Three-point estimate.
            for i in 0..9 {
                let t00 = (g1[i] + g5[i]) * dzot;
                t01[i] = (t00 + dz * g3[i]) * 0.5;
                t10[i] = (4.0 * t01[i] - t00) / 3.0;
            }
            let (tr, _) = test(mag(&t01), mag(&t10), 0.0, 0.0, dmin);
            if tr <= rx {
                for i in 0..9 {
                    sum[i] += t10[i];
                }
                nt += 2;
                break;
            }
            // Five-point estimate.
            g2 = f(z + dz * 0.25);
            g4 = f(z + dz * 0.75);
            let mut t11 = [ZERO; 9];
            for i in 0..9 {
                let t02 = (t01[i] + dzot * (g2[i] + g4[i])) * 0.5;
                t11[i] = (4.0 * t02 - t01[i]) / 3.0;
                t20[i] = (16.0 * t11[i] - t10[i]) / 15.0;
            }
            let (tr, _) = test(mag(&t11), mag(&t20), 0.0, 0.0, dmin);
            if tr > rx {
                nt = 0;
            }
            if tr <= rx || ns >= NM {
                // Converged, or step-limited ("ROM2 -- STEP SIZE LIMITED"): accept.
                for i in 0..9 {
                    sum[i] += t20[i];
                }
                nt += 1;
                break;
            }
            ns *= 2;
            dz = s / f64::from(ns);
            dzot = dz * 0.5;
            g5 = g3;
            g3 = g2;
        }
        z += dz;
        if z > zend {
            return sum;
        }
        g1 = g5;
        if nt >= NTS && ns > NX {
            ns /= 2;
            nt = 1;
        }
    }
}

/// `GWAVE`: field of a current element over ground including the ground wave, by K. A.
/// Norton's formulas (Proc. IRE, Sept. 1937). Returns `(ERV, EZV, ERH, EZH, EPH)`.
#[allow(
    clippy::too_many_arguments,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::too_many_lines
)]
pub fn gwave(u: C, u2: C, xx1: C, xx2: C, r1: f64, r2: f64, zmh: f64, zph: f64) -> (C, C, C, C, C) {
    let tpj = C::new(0.0, TP);
    let econ = C::new(0.0, single(-188.367));
    let sppp = zmh / r1;
    let sppp2 = sppp * sppp;
    let mut cppp2 = 1.0 - sppp2;
    if cppp2 < 1e-20 {
        cppp2 = 1e-20;
    }
    let cppp = cppp2.sqrt();
    let spp = zph / r2;
    let spp2 = spp * spp;
    let mut cpp2 = 1.0 - spp2;
    if cpp2 < 1e-20 {
        cpp2 = 1e-20;
    }
    let cpp = cpp2.sqrt();
    let rk1 = -(tpj * r1);
    let rk2 = -(tpj * r2);
    let t1 = ONE - u2 * cpp2;
    let t2 = csqrt(t1);
    let t3 = cdiv(ONE - rdiv(1.0, rk1), rk1);
    let t4 = cdiv(ONE - rdiv(1.0, rk2), rk2);
    let p1 = rk2 * u2 * t1 / (2.0 * cpp2);
    let rv = cdiv(spp - u * t2, spp + u * t2);
    let omr = ONE - rv;
    let w = rdiv(1.0, omr);
    let w = C::new(4.0, 0.0) * p1 * w * w;
    let f = fbar(w);
    let q1 = cdiv(rk2 * t1, 2.0 * u2 * cpp2);
    let rh = cdiv(t2 - u * spp, t2 + u * spp);
    let v = rdiv(1.0, 1.0 + rh);
    let v = C::new(4.0, 0.0) * q1 * v * v;
    let g = fbar(v);
    let xr1 = xx1 / r1;
    let xr2 = xx2 / r2;

    let x1 = cppp2 * xr1;
    let x2 = rv * cpp2 * xr2;
    let x3 = omr * cpp2 * f * xr2;
    let x4 = cdiv(u * t2 * spp * 2.0 * xr2, rk2);
    let x5 = xr1 * t3 * (1.0 - 3.0 * sppp2);
    let x6 = xr2 * t4 * (1.0 - 3.0 * spp2);
    let ezv = (x1 + x2 + x3 - x4 - x5 - x6) * econ;

    let x1 = sppp * cppp * xr1;
    let x2 = rv * spp * cpp * xr2;
    let x3 = cpp * omr * u * t2 * f * xr2;
    let x4 = cdiv(spp * cpp * omr * xr2, rk2);
    let x5 = 3.0 * sppp * cppp * t3 * xr1;
    let x6 = cdiv(cpp * u * t2 * omr * xr2, rk2) * 0.5;
    let x7 = 3.0 * spp * cpp * t4 * xr2;
    let erv = -((x1 + x2 - x3 + x4 - x5 + x6 - x7) * econ);
    let ezh = -((x1 - x2 + x3 - x4 - x5 - x6 + x7) * econ);

    let inv_rk2 = rdiv(1.0, rk2);
    let x1 = sppp2 * xr1;
    let x2 = rv * spp2 * xr2;
    let x4 = u2 * t1 * omr * f * xr2;
    let x5 = t3 * (1.0 - 3.0 * cppp2) * xr1;
    let x6 = t4 * (1.0 - 3.0 * cpp2) * (ONE - u2 * (1.0 + rv) - u2 * omr * f) * xr2;
    let x7 = u2 * cpp2 * omr * (ONE - inv_rk2) * (f * (u2 * t1 - spp2 - inv_rk2) + inv_rk2) * xr2;
    let erh = (x1 - x2 - x4 - x5 + x6 + x7) * econ;

    let x1 = xr1;
    let x2 = rh * xr2;
    let x3 = (rh + 1.0) * g * xr2;
    let x4 = t3 * xr1;
    let x5 = t4 * (ONE - u2 * (1.0 + rv) - u2 * omr * f) * xr2;
    let x6 = cdiv(
        0.5 * u2 * omr * (f * (u2 * t1 - spp2 - inv_rk2) + inv_rk2) * xr2,
        rk2,
    );
    let eph = -((x1 - x2 + x3 - x4 + x5 + x6) * econ);
    (erv, ezv, erh, ezh, eph)
}

/// `FBAR`: Sommerfeld's attenuation function for numerical distance `p`.
fn fbar(p: C) -> C {
    let tosp = 1.128_379_167;
    let accs = 1e-12;
    let sp = 1.772_453_851;
    let mut z = FJ * csqrt(p);
    if z.norm() <= 3.0 {
        // Series expansion.
        let zs = z * z;
        let mut sum = z;
        let mut pow = z;
        for i in 1..=100 {
            pow = -(pow * zs / f64::from(i));
            let term = pow / (2.0 * f64::from(i) + 1.0);
            sum += term;
            if norm_sqr(term) / norm_sqr(sum) < accs {
                break;
            }
        }
        return ONE - (ONE - sum * tosp) * z * zs.exp() * sp;
    }
    // Asymptotic expansion.
    let minus = z.re < 0.0;
    if minus {
        z = -z;
    }
    let zs = rdiv(0.5, z * z);
    let mut sum = ZERO;
    let mut term = ONE;
    for i in 1..=6 {
        term = -(term * (2.0 * f64::from(i) - 1.0) * zs);
        sum += term;
    }
    if minus {
        sum -= 2.0 * sp * z * (z * z).exp();
    }
    -sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fbar_is_one_at_zero_distance_and_decays() {
        // F(0) = 1; |F| falls toward the asymptotic -1/(2p) at large numerical distance.
        assert!((fbar(C::new(1e-12, 0.0)) - ONE).norm() < 1e-5);
        let far = fbar(C::new(200.0, 50.0));
        assert!(far.norm() < 0.01, "{far}");
    }
}
