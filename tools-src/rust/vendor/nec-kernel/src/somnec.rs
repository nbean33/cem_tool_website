//! The Sommerfeld ground interpolation grid (`SOM2D`, the in-engine version of `SOMNEC`).
//!
//! Over a lossy ground, the field reflected from the ground is a Sommerfeld integral with no
//! closed form. NEC evaluates the four field components it needs — `ERV`, `EZV` (vertical
//! source) and `ERH`, `EPH` (horizontal source) — once per ground and frequency on a grid in
//! `(R, θ)`, where `R` is the distance from the image point and `θ` its elevation, and then
//! interpolates during the matrix fill ([`SommerfeldGrid::interpolate`], `INTRP`).
//!
//! Three grid regions:
//!
//! | grid | R (λ) | θ (rad) | points |
//! |---|---|---|---|
//! | 1 | 0 – 0.2, step 0.02 | 0 – π/2, step π/18 | 11 × 10 |
//! | 2 | 0.2 – 1.0, step 0.05 | 0 – π/9, step π/36 | 17 × 5 |
//! | 3 | 0.2 – 1.0, step 0.1 | π/9 – π/2, step π/18 | 9 × 8 |
//!
//! Each grid value is an integral along a contour in the complex λ plane, chosen by `EVLUA`
//! between a Bessel-function form (observer high above the ground relative to its horizontal
//! distance) and a Hankel-function form, integrated by Romberg quadrature (`ROM1`) and
//! accelerated to infinity with Shanks' transformation (`GSHANK`).
//!
//! Single-precision literals are ported with [`single`]; see [`crate::cmath`].

// NEC's literals are transcribed digit for digit, including single-precision ones written
// with more digits than an f32 holds and truncated values of π-related constants: matching
// the reference engine depends on rounding them exactly as Fortran does, not on their being
// the best approximation of what they stand for.
#![allow(clippy::excessive_precision, clippy::approx_constant)]

use crate::cmath::{cdiv, csqrt, norm_sqr, rdiv, single};
use crate::intx::test;
use num_complex::Complex64;
use std::sync::OnceLock;

type C = Complex64;
const ZERO: C = C::new(0.0, 0.0);
const ONE: C = C::new(1.0, 0.0);
const FJ: C = C::new(0.0, 1.0);

/// Grid shapes, `BLOCK DATA SOMSET`: points in R and θ, start and step of each.
const NXA: [usize; 3] = [11, 17, 9];
const NYA: [usize; 3] = [10, 5, 8];
const XSA: [f64; 3] = [0.0, single(0.2), single(0.2)];
const YSA: [f64; 3] = [0.0, 0.0, single(0.349_065_850_4)];
const DXA: [f64; 3] = [single(0.02), single(0.05), single(0.1)];
const DYA: [f64; 3] = [single(0.174_532_925_2), single(0.087_266_462_6), single(0.174_532_925_2)];

/// The four field components at one grid point: `[ERV, EZV, ERH, EPH]`.
pub type Fields = [C; 4];

/// A computed Sommerfeld interpolation grid for one complex ground permittivity.
#[derive(Debug, Clone, PartialEq)]
pub struct SommerfeldGrid {
    /// `ε_c` as `SOM2D` computed it (`EPSCF`).
    pub epscf: C,
    /// Grid values, `grids[k][ir][ith]`, 0-based.
    grids: [Vec<Vec<Fields>>; 3],
}

impl SommerfeldGrid {
    /// `SOM2D(FMHZ, EPR, SIG)`: the grid for relative permittivity `epsr` and conductivity
    /// `sigma` (S/m) at `mhz`. A negative `sigma` is taken directly as the imaginary part of
    /// `ε_c`, as `SOM2D` does.
    #[must_use]
    pub fn new(mhz: f64, epsr: f64, sigma: f64) -> Self {
        let epscf = if sigma < 0.0 {
            C::new(epsr, sigma)
        } else {
            let wlam = single(299.8) / mhz;
            C::new(epsr, -sigma * wlam * single(59.96))
        };
        Self::for_permittivity(epscf)
    }

    /// The grid for complex relative permittivity `epscf`.
    #[must_use]
    pub fn for_permittivity(epscf: C) -> Self {
        let ev = Evl::new(epscf);

        // The grid points, enumerated in SOM2D's order with its running sums for R and θ, so
        // the sample positions are bit-identical to NEC's.
        let mut points: Vec<(usize, usize, usize, f64, f64)> = Vec::new();
        for k in 0..3 {
            let (nr, nth, dr, dth) = (NXA[k], NYA[k], DXA[k], DYA[k]);
            let (mut r, irs) = if k == 0 { (XSA[k], 2) } else { (XSA[k] - dr, 1) };
            for ir in irs..=nr {
                r += dr;
                let mut thet = YSA[k] - dth;
                for ith in 1..=nth {
                    thet += dth;
                    points.push((k, ir - 1, ith - 1, r, thet));
                }
            }
        }

        let values: Vec<Fields> = {
            use rayon::prelude::*;
            points
                .par_iter()
                .map(|&(_, _, _, r, thet)| {
                    let mut rho = r * thet.cos();
                    let mut zph = r * thet.sin();
                    if rho < single(1e-7) {
                        rho = single(1e-8);
                    }
                    if zph < single(1e-7) {
                        zph = 0.0;
                    }
                    let [erv, ezv, erh, eph] = ev.evlua(rho, zph);
                    let rk = ev.ck2 * r;
                    let con = -cdiv(C::new(0.0, single(4.771_47)) * r, C::new(rk.cos(), -rk.sin()));
                    [erv * con, ezv * con, erh * con, eph * con]
                })
                .collect()
        };

        let mut grids: [Vec<Vec<Fields>>; 3] =
            std::array::from_fn(|k| vec![vec![[ZERO; 4]; NYA[k]]; NXA[k]]);
        for (&(k, ir, ith, _, _), v) in points.iter().zip(values) {
            grids[k][ir][ith] = v;
        }

        // Grid 1 at R = 0, from the closed-form limit.
        let cl2 = -cdiv(C::new(0.0, single(188.370)) * (epscf - 1.0), epscf + 1.0);
        let cl1 = cdiv(cl2, epscf + 1.0);
        let ezv = epscf * cl1;
        // SOM2D reuses DTH from its last grid loop, which is grid 3's step — the same value as
        // grid 1's.
        let dth = DYA[2];
        let nth = NYA[0];
        let mut thet = -dth;
        for ith in 1..=nth {
            thet += dth;
            let (erv, erh, eph) = if ith == nth {
                let erh = cl2 - 0.5 * cl1;
                (ZERO, erh, -erh)
            } else {
                let mut tfac2 = thet.cos();
                let tfac1 = (1.0 - thet.sin()) / tfac2;
                tfac2 = tfac1 / tfac2;
                (
                    epscf * cl1 * tfac1,
                    cl1 * (tfac2 - 1.0) + cl2,
                    cl1 * tfac2 - cl2,
                )
            };
            grids[0][0][ith - 1] = [erv, ezv, erh, eph];
        }

        Self { epscf, grids }
    }

    /// A grid value, 0-based: grid `k`, R index `ir`, θ index `ith`.
    #[must_use]
    pub fn value(&self, k: usize, ir: usize, ith: usize) -> Fields {
        self.grids[k][ir][ith]
    }

    /// `INTRP`: bivariate cubic interpolation of the four field components at distance `x`
    /// (wavelengths) and elevation angle `y` (radians).
    ///
    /// NEC caches the last 4×4 patch of coefficients between calls; recomputing it gives the
    /// same patch, so this version is stateless.
    #[must_use]
    pub fn interpolate(&self, x: f64, y: f64) -> Fields {
        let g = if x <= XSA[1] {
            0
        } else if y > YSA[2] {
            2
        } else {
            1
        };
        let (dx, dy, xs, ys) = (DXA[g], DYA[g], XSA[g], YSA[g]);
        let nxm2 = NXA[g] - 2;
        let nym2 = NYA[g] - 2;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let ix = ((x - xs) / dx) as i64 + 1;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let iy = ((y - ys) / dy) as i64 + 1;
        // Fortran integer division truncates toward zero, as Rust's does.
        #[allow(clippy::cast_possible_wrap)]
        let ixs = (((ix - 1) / 3) * 3 + 2).max(2).min(nxm2 as i64);
        #[allow(clippy::cast_possible_wrap)]
        let iys = (((iy - 1) / 3) * 3 + 2).max(2).min(nym2 as i64);
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let (ixs, iys) = (ixs as usize, iys as usize);

        let sixth = 0.166_666_666_7;
        let grid = &self.grids[g];
        // coef[comp][row] = (a, b, c, d) of the cubic in x along grid row IYS-2+row (1-based).
        let mut coef = [[(ZERO, ZERO, ZERO, ZERO); 4]; 4];
        for (k, ck) in coef.iter_mut().enumerate() {
            for (i, cell) in ck.iter_mut().enumerate() {
                // 1-based row IYS-1+i, columns IXS-1 .. IXS+2.
                let row = iys - 2 + i;
                let p1 = grid[ixs - 2][row][k];
                let p2 = grid[ixs - 1][row][k];
                let p3 = grid[ixs][row][k];
                let p4 = grid[ixs + 1][row][k];
                *cell = (
                    (p4 - p1 + 3.0 * (p2 - p3)) * sixth,
                    (p1 - 2.0 * p2 + p3) * 0.5,
                    p3 - (2.0 * p1 + 3.0 * p2 + p4) * sixth,
                    p2,
                );
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let xz = (ixs - 1) as f64 * dx + xs;
        #[allow(clippy::cast_precision_loss)]
        let yz = (iys - 1) as f64 * dy + ys;
        let xx = (x - xz) / dx;
        let yy = (y - yz) / dy;

        let mut out = [ZERO; 4];
        for (k, ck) in coef.iter().enumerate() {
            let fx = |(a, b, c, d): (C, C, C, C)| ((a * xx + b) * xx + c) * xx + d;
            let (fx1, fx2, fx3, fx4) = (fx(ck[0]), fx(ck[1]), fx(ck[2]), fx(ck[3]));
            let p1 = fx4 - fx1 + 3.0 * (fx2 - fx3);
            let p2 = 3.0 * (fx1 - 2.0 * fx2 + fx3);
            let p3 = 6.0 * fx3 - 2.0 * fx1 - 3.0 * fx2 - fx4;
            out[k] = ((p1 * yy + p2) * yy + p3) * yy * sixth + fx2;
        }
        out
    }
}

/// `COMMON /EVLCOM/`: the ground constants every integrand evaluation needs.
struct Evl {
    cksm: C,
    ct1: C,
    ct2: C,
    ct3: C,
    ck1: C,
    ck1sq: C,
    ck2: f64,
    ck2sq: f64,
    tkmag: f64,
    tsmag: f64,
    ck1r: f64,
}

/// Where the integrand is evaluated: one grid point, and the integral form in use.
#[derive(Clone, Copy)]
struct Point {
    rho: f64,
    zph: f64,
    /// `JH = 1`: Hankel-function form.
    hankel: bool,
}

impl Evl {
    fn new(epscf: C) -> Self {
        let ck2 = single(6.283_185_308);
        let ck2sq = ck2 * ck2;
        // The Sommerfeld integrals are written for exp(−jωt); NEC uses exp(+jωt), hence the
        // conjugate here and again on the results in EVLUA.
        let ck1sq = ck2sq * epscf.conj();
        let ck1 = csqrt(ck1sq);
        let tkmag = 100.0 * ck1.norm();
        let tsmag = (100.0 * ck1 * ck1.conj()).re;
        let cksm = rdiv(ck2sq, ck1sq + ck2sq);
        let ct1 = 0.5 * (ck1sq - ck2sq);
        let mut erv = ck1sq * ck1sq;
        let mut ezv = ck2sq * ck2sq;
        let ct2 = 0.125 * (erv - ezv);
        erv *= ck1sq;
        ezv *= ck2sq;
        let ct3 = 0.0625 * (erv - ezv);
        Self {
            cksm,
            ct1,
            ct2,
            ct3,
            ck1,
            ck1sq,
            ck2,
            ck2sq,
            tkmag,
            tsmag,
            ck1r: ck1.re,
        }
    }

    /// `EVLUA`: the four field components at one point, choosing the integration contour.
    #[allow(clippy::many_single_char_names)]
    fn evlua(&self, rho: f64, zph: f64) -> Fields {
        let ptp = single(0.628_318_530_8);
        let mut del = if rho > zph { rho } else { zph };
        let mut ans: [C; 6];

        if zph >= 2.0 * rho {
            // Bessel-function form.
            let p = Point { rho, zph, hankel: false };
            del = 1.0 / del;
            let mut sum;
            let b;
            if del > self.tkmag {
                let b1 = C::new(single(0.1) * self.tkmag, -single(0.1) * self.tkmag);
                sum = self.rom1(p, ZERO, b1, 2);
                b = C::new(del, -del);
                let more = self.rom1(p, b1, b, 2);
                for i in 0..6 {
                    sum[i] += more[i];
                }
            } else {
                b = C::new(del, -del);
                sum = self.rom1(p, ZERO, b, 2);
            }
            let delta = C::new(ptp * del, 0.0);
            ans = self.gshank(p, b, delta, sum, false, b, b);
        } else {
            // Hankel-function form.
            let p = Point { rho, zph, hankel: true };
            let ck2 = self.ck2;
            let cp1 = C::new(0.0, single(0.4) * ck2);
            let cp2 = C::new(single(0.6) * ck2, -single(0.2) * ck2);
            let cp3 = C::new(single(1.02) * ck2, -single(0.2) * ck2);
            let s1 = self.rom1(p, cp1, cp2, 2);
            let s2 = self.rom1(p, cp2, cp3, 2);
            let mut sum = [ZERO; 6];
            for i in 0..6 {
                sum[i] = -(s1[i] + s2[i]);
            }
            // Path from the imaginary axis to −∞.
            let mut slope = 1000.0;
            if zph > single(0.001) * rho {
                slope = rho / zph;
            }
            del = ptp / del;
            let delta = C::new(-1.0, slope) * del / (1.0 + slope * slope).sqrt();
            let delta2 = -delta.conj();
            // BK is unset here in NEC, and unused because IBK = 0.
            ans = self.gshank(p, cp1, delta, sum, false, ZERO, ZERO);

            let mut below = true;
            let rmis = rho * (self.ck1.re - ck2);
            if rmis >= 2.0 * ck2 && rho >= single(1e-10) {
                if zph < single(1e-10) {
                    below = false;
                } else {
                    let bk = C::new(-zph, rho) * (self.ck1 - cp3);
                    let rmis = -bk.re / bk.im.abs();
                    below = rmis > 4.0 * rho / zph;
                }
            }

            if below {
                // Integrate below the branch points, then to +∞.
                let mut sum = [ZERO; 6];
                for i in 0..6 {
                    sum[i] = -ans[i];
                }
                let mut rmis = self.ck1.re * single(1.01);
                if ck2 + 1.0 > rmis {
                    rmis = ck2 + 1.0;
                }
                let bk = C::new(rmis, single(0.99) * self.ck1.im);
                let mut delta = bk - cp3;
                delta = delta * del / delta.norm();
                ans = self.gshank(p, cp3, delta, sum, true, bk, delta2);
            } else {
                // Integrate up between the branch cuts, then to +∞.
                let cp1 = self.ck1 - C::new(single(0.1), single(0.2));
                let cp2 = cp1 + single(0.2);
                let bk = C::new(0.0, del);
                let sum = self.gshank(p, cp1, bk, ans, false, bk, bk);
                let mut a = self.rom1(p, cp1, cp2, 1);
                for i in 0..6 {
                    a[i] -= sum[i];
                }
                let sum = self.gshank(p, cp3, bk, a, false, bk, bk);
                ans = self.gshank(p, cp2, delta2, sum, false, bk, bk);
            }
        }

        ans[5] *= self.ck1;
        // Conjugate, since NEC uses exp(+jωt).
        [
            (self.ck1sq * ans[2]).conj(),
            (self.ck1sq * (ans[1] + self.ck2sq * ans[4])).conj(),
            (self.ck2sq * (ans[0] + ans[5])).conj(),
            -(self.ck2sq * (ans[3] + ans[5])).conj(),
        ]
    }

    /// `GSHANK`: integrate from `start` to infinity in steps of `dela`, accelerating the
    /// partial sums with Shanks' transformation. At the break point `bk` (when `ibk`), the step
    /// changes to `delb`.
    #[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
    fn gshank(
        &self,
        p: Point,
        start: C,
        dela: C,
        seed: [C; 6],
        ibk: bool,
        bk: C,
        delb: C,
    ) -> [C; 6] {
        const MAXH: usize = 20;
        let crit = single(1e-4);
        let rbk = bk.re;
        let mut del = dela;
        // IBX = 0: the break point is still ahead.
        let mut ibx = if ibk { 0 } else { 1 };
        let mut ans1 = [ZERO; 6];
        let mut ans2 = seed;
        let mut q1 = [[ZERO; MAXH + 1]; 6];
        let mut q2 = [[ZERO; MAXH + 1]; 6];
        let mut b = start;
        let mut a;

        'restart: loop {
            let mut inx = 0;
            for int in 1..=MAXH {
                inx = int;
                a = b;
                b += del;
                if ibx == 0 && b.re >= rbk {
                    // Hit the break point on the first half-step.
                    ibx = 1;
                    b = bk;
                    del = delb;
                    let sum = self.rom1(p, a, b, 2);
                    for i in 0..6 {
                        ans2[i] += sum[i];
                    }
                    continue 'restart;
                }
                let sum = self.rom1(p, a, b, 2);
                for i in 0..6 {
                    ans1[i] = ans2[i] + sum[i];
                }
                a = b;
                b += del;
                if ibx == 0 && b.re >= rbk {
                    ibx = 2;
                    b = bk;
                    del = delb;
                    let sum = self.rom1(p, a, b, 2);
                    for i in 0..6 {
                        ans2[i] = ans1[i] + sum[i];
                    }
                    continue 'restart;
                }
                let sum = self.rom1(p, a, b, 2);
                for i in 0..6 {
                    ans2[i] = ans1[i] + sum[i];
                }

                // Shanks' transformation.
                let mut den: f64 = 0.0;
                for i in 0..6 {
                    let mut as1 = ans1[i];
                    let mut as2 = ans2[i];
                    for jm in 1..int {
                        let aa = q2[i][jm];
                        let mut a1 = q1[i][jm] + as1 - 2.0 * aa;
                        if a1.re == 0.0 && a1.im == 0.0 {
                            a1 = q1[i][jm];
                        } else {
                            let a2 = aa - q1[i][jm];
                            a1 = q1[i][jm] - cdiv(a2 * a2, a1);
                        }
                        let mut a2 = aa + as2 - 2.0 * as1;
                        if a2.re == 0.0 && a2.im == 0.0 {
                            a2 = aa;
                        } else {
                            a2 = aa - cdiv((as1 - aa) * (as1 - aa), a2);
                        }
                        q1[i][jm] = as1;
                        q2[i][jm] = as2;
                        as1 = a1;
                        as2 = a2;
                    }
                    q1[i][int] = as1;
                    q2[i][int] = as2;
                    let amg = as2.re.abs() + as2.im.abs();
                    if amg > den {
                        den = amg;
                    }
                }
                let denm = single(1e-3) * den * crit;
                let jm = if int > 3 { int - 3 } else { 1 };
                let converged = (jm..=int).all(|j| {
                    (0..6).all(|i| {
                        let a1 = q2[i][j];
                        let mut den = (a1.re.abs() + a1.im.abs()) * crit;
                        if den < denm {
                            den = denm;
                        }
                        let d = q1[i][j] - a1;
                        d.re.abs() + d.im.abs() <= den
                    })
                });
                if converged {
                    break;
                }
                // NEC prints "NO CONVERGENCE IN SUBROUTINE GSHANK" and uses the last estimate.
            }
            let mut out = [ZERO; 6];
            for i in 0..6 {
                out[i] = 0.5 * (q1[i][inx] + q2[i][inx]);
            }
            return out;
        }
    }

    /// `ROM1`: integrate the six Sommerfeld integrands along the straight contour from `a` to
    /// `b` by variable-interval Romberg integration. `nx` is the starting number of intervals.
    #[allow(clippy::many_single_char_names)]
    fn rom1(&self, p: Point, a: C, b: C, nx: u32) -> [C; 6] {
        const NM: u32 = 131_072;
        const NTS: u32 = 4;
        let rx = single(1e-4);
        let f = |t: f64| self.saoa(p, a, b, t);

        let mut z = 0.0;
        let ze = 1.0;
        let s = 1.0;
        let ep = s / (1e4 * f64::from(NM));
        let zend = ze - ep;
        let mut sum = [ZERO; 6];
        let mut ns = nx;
        let mut nt = 0;
        let mut g1 = f(z);
        let (mut g2, mut g3, mut g4, mut g5);
        let mut t01 = [ZERO; 6];
        let mut t10 = [ZERO; 6];
        let mut t20 = [ZERO; 6];

        loop {
            // Label 2: next interval.
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
                // Label 4: three-point Romberg estimate.
                let mut nogo = false;
                for i in 0..6 {
                    let t00 = (g1[i] + g5[i]) * dzot;
                    t01[i] = (t00 + dz * g3[i]) * 0.5;
                    t10[i] = (4.0 * t01[i] - t00) / 3.0;
                    let (tr, ti) = test(t01[i].re, t10[i].re, t01[i].im, t10[i].im, 0.0);
                    if tr > rx || ti > rx {
                        nogo = true;
                    }
                }
                if !nogo {
                    for i in 0..6 {
                        sum[i] += t10[i];
                    }
                    nt += 2;
                    break;
                }
                // Five-point estimate.
                g2 = f(z + dz * 0.25);
                g4 = f(z + dz * 0.75);
                let mut nogo = false;
                for i in 0..6 {
                    let t02 = (t01[i] + dzot * (g2[i] + g4[i])) * 0.5;
                    let t11 = (4.0 * t02 - t01[i]) / 3.0;
                    t20[i] = (16.0 * t11 - t10[i]) / 15.0;
                    let (tr, ti) = test(t11.re, t20[i].re, t11.im, t20[i].im, 0.0);
                    if tr > rx || ti > rx {
                        nogo = true;
                    }
                }
                if nogo {
                    nt = 0;
                }
                if !nogo || ns >= NM {
                    // Converged, or the step can shrink no further (NEC prints "ROM1 -- STEP
                    // SIZE LIMITED" and accepts the estimate).
                    for i in 0..6 {
                        sum[i] += t20[i];
                    }
                    nt += 1;
                    break;
                }
                // Halve the interval and retry.
                ns *= 2;
                dz = s / f64::from(ns);
                dzot = dz * 0.5;
                g5 = g3;
                g3 = g2;
            }
            // Label 11: advance.
            z += dz;
            if z > zend {
                return sum;
            }
            g1 = g5;
            if nt >= NTS && ns > nx {
                ns /= 2;
                nt = 1;
            }
        }
    }

    /// `SAOA` (with `LAMBDA`): the six integrands at parameter `t` along the contour `a → b`.
    #[allow(clippy::many_single_char_names)]
    fn saoa(&self, p: Point, a: C, b: C, t: f64) -> [C; 6] {
        let dxl = b - a;
        let xl = a + dxl * t;
        let (mut b0, mut b0p, cgam1, cgam2);
        if p.hankel {
            (b0, b0p) = hankel(xl * p.rho);
            let branch = |com: C, k: C| {
                let g = csqrt(xl + k) * csqrt(com);
                if com.re < 0.0 && com.im >= 0.0 { -g } else { g }
            };
            cgam1 = branch(xl - self.ck1, self.ck1);
            cgam2 = branch(xl - self.ck2, C::new(self.ck2, 0.0));
        } else {
            (b0, b0p) = bessel(xl * p.rho);
            b0 *= 2.0;
            b0p *= 2.0;
            let fix = |g: C| if g.re == 0.0 { C::new(0.0, -g.im.abs()) } else { g };
            cgam1 = fix(csqrt(xl * xl - self.ck1sq));
            cgam2 = fix(csqrt(xl * xl - self.ck2sq));
        }

        let xlr = norm_sqr(xl);
        let sign = if xlr >= self.tsmag {
            if xl.im < 0.0 {
                Some(1.0)
            } else if xl.re < self.ck2 {
                Some(-1.0)
            } else if xl.re > self.ck1r {
                Some(1.0)
            } else {
                None
            }
        } else {
            None
        };
        let dgam = match sign {
            // Asymptotic form of γ₂ − γ₁ for large |λ|, avoiding cancellation.
            Some(sign) => {
                let d = rdiv(1.0, xl * xl);
                cdiv(sign * ((self.ct3 * d + self.ct2) * d + self.ct1), xl)
            }
            None => cgam2 - cgam1,
        };

        let den2 = cdiv(
            self.cksm * dgam,
            cgam2 * (self.ck1sq * cgam2 + self.ck2sq * cgam1),
        );
        let den1 = rdiv(1.0, cgam1 + cgam2) - cdiv(self.cksm, cgam2);
        let mut com = dxl * xl * (-(cgam2 * p.zph)).exp();
        let mut ans = [ZERO; 6];
        ans[5] = cdiv(com * b0 * den1, self.ck1);
        com *= den2;
        if p.rho == 0.0 {
            ans[0] = -(com * xl * xl * 0.5);
            ans[3] = ans[0];
        } else {
            b0p /= p.rho;
            ans[0] = -(com * xl * (b0p + b0 * xl));
            ans[3] = com * xl * b0p;
        }
        ans[1] = com * cgam2 * cgam2 * b0;
        ans[2] = -(ans[3] * cgam2 * p.rho);
        ans[4] = com * b0;
        ans
    }
}

/// Series tables shared by [`bessel`] and [`hankel`].
struct Tables {
    a1: [f64; 25],
    a2: [f64; 25],
    a3: [f64; 25],
    a4: [f64; 25],
    /// Terms needed in the series for `|z|² < i`, for `BESSEL`.
    m_bessel: [usize; 101],
    /// The same for `HANKEL`, whose test includes the `Y₀` coefficients.
    m_hankel: [usize; 101],
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let gamma = single(0.577_215_664_9);
        let mut t = Tables {
            a1: [0.0; 25],
            a2: [0.0; 25],
            a3: [0.0; 25],
            a4: [0.0; 25],
            m_bessel: [0; 101],
            m_hankel: [0; 101],
        };
        let mut psi = -gamma;
        for k in 1..=25 {
            let kf = f64::from(k);
            let i = (k - 1) as usize;
            t.a1[i] = -0.25 / f64::from(k * k);
            t.a2[i] = 1.0 / (kf + 1.0);
            psi += 1.0 / kf;
            t.a3[i] = psi + psi;
            t.a4[i] = (psi + psi + 1.0 / (kf + 1.0)) / (kf + 1.0);
        }
        for i in 1..=101 {
            let fi = f64::from(i);
            let terms = |weight: &dyn Fn(usize) -> f64| {
                let mut test = 1.0;
                let mut init = 0;
                for k in 1..=24 {
                    init = k;
                    test = -test * fi * t.a1[k - 1];
                    if test * weight(k - 1) < 1e-6 {
                        break;
                    }
                }
                init
            };
            t.m_bessel[(i - 1) as usize] = terms(&|_| 1.0);
            t.m_hankel[(i - 1) as usize] = terms(&|k| t.a3[k]);
        }
        t
    })
}

/// Coefficients of the large-argument expansions, shared by `BESSEL` and `HANKEL`.
const C3: f64 = single(0.797_884_560_8);
const P10: f64 = single(0.070_312_5);
const P20: f64 = single(0.112_152_099_6);
const Q10: f64 = single(0.125);
const Q20: f64 = single(0.073_242_187_5);
const P11: f64 = single(0.117_187_5);
const P21: f64 = single(0.144_195_556_6);
const Q11: f64 = single(0.375);
const Q21: f64 = single(0.102_539_062_5);
const POF: f64 = single(0.785_398_163_5);

/// The `P` and `Q` polynomials of the asymptotic expansions, `(P0, P1, Q0, Q1)`.
fn asymptotic_pq(z: C) -> (C, C, C, C, C) {
    let zi = rdiv(1.0, z);
    let zi2 = zi * zi;
    let p0z = ONE + (P20 * zi2 - P10) * zi2;
    let p1z = ONE + (P11 - P21 * zi2) * zi2;
    let q0z = (Q20 * zi2 - Q10) * zi;
    let q1z = (Q11 - Q21 * zi2) * zi;
    (zi, p0z, p1z, q0z, q1z)
}

/// `BESSEL`: `J₀(z)` and `J₀'(z)` for complex `z`.
#[must_use]
pub fn bessel(z: C) -> (C, C) {
    let t = tables();
    let zms = norm_sqr(z);
    if zms <= single(1e-12) {
        return (ONE, -0.5 * z);
    }
    let mut blend = None;
    if zms <= single(37.21) {
        // Series expansion.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let iz = (1.0 + zms) as usize;
        let miz = t.m_bessel[iz - 1];
        let mut j0 = ONE;
        let mut j0p = ONE;
        let mut zk = ONE;
        let zi = z * z;
        for k in 0..miz {
            zk = zk * t.a1[k] * zi;
            j0 += zk;
            j0p += t.a2[k] * zk;
        }
        j0p = -(0.5 * z * j0p);
        if zms <= 36.0 {
            return (j0, j0p);
        }
        blend = Some((j0, j0p));
    }
    // Asymptotic expansion.
    let (zi, p0z, p1z, q0z, q1z) = asymptotic_pq(z);
    let zk = (FJ * (z - POF)).exp();
    let zi2 = rdiv(1.0, zk);
    let cz = 0.5 * (zk + zi2);
    let sz = FJ * 0.5 * (zi2 - zk);
    let zk = C3 * csqrt(zi);
    let mut j0 = zk * (p0z * cz - q0z * sz);
    let mut j0p = -(zk * (p1z * sz + q1z * cz));
    if let Some((j0x, j0px)) = blend {
        // 36 < |z|² ≤ 37.21: blend the two forms smoothly.
        let w = ((zms.sqrt() - 6.0) * single(31.415_926_54)).cos();
        j0 = 0.5 * (j0x * (1.0 + w) + j0 * (1.0 - w));
        j0p = 0.5 * (j0px * (1.0 + w) + j0p * (1.0 - w));
    }
    (j0, j0p)
}

/// `HANKEL`: `H₀⁽¹⁾(z)` and its derivative for complex `z ≠ 0`.
#[must_use]
pub fn hankel(z: C) -> (C, C) {
    let pi = single(3.141_592_654);
    let c1 = single(-0.024_578_509_5);
    let c2 = single(0.367_466_905_2);
    let t = tables();
    let zms = norm_sqr(z);
    // NEC stops with "HANKEL NOT VALID FOR Z=0"; EVLUA never calls it there.
    debug_assert!(zms != 0.0, "HANKEL not valid for z = 0");
    let mut blend = None;
    if zms <= single(16.81) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let iz = (1.0 + zms) as usize;
        let miz = t.m_hankel[iz - 1];
        let mut j0 = ONE;
        let mut j0p = ONE;
        let mut y0 = ZERO;
        let mut y0p = ZERO;
        let mut zk = ONE;
        let zi = z * z;
        for k in 0..miz {
            zk = zk * t.a1[k] * zi;
            j0 += zk;
            j0p += t.a2[k] * zk;
            y0 += t.a3[k] * zk;
            y0p += t.a4[k] * zk;
        }
        j0p = -(0.5 * z * j0p);
        let clogz = (0.5 * z).ln();
        y0 = (2.0 * j0 * clogz - y0) / pi + c2;
        y0p = (rdiv(2.0, z) + 2.0 * j0p * clogz + 0.5 * y0p * z) / pi + c1 * z;
        let h0 = j0 + FJ * y0;
        let h0p = j0p + FJ * y0p;
        if zms <= 16.0 {
            return (h0, h0p);
        }
        blend = Some((h0, h0p));
    }
    let (zi, p0z, p1z, q0z, q1z) = asymptotic_pq(z);
    let zk = (FJ * (z - POF)).exp() * csqrt(zi) * C3;
    let mut h0 = zk * (p0z + FJ * q0z);
    let mut h0p = FJ * zk * (p1z + FJ * q1z);
    if let Some((y0, y0p)) = blend {
        let w = ((zms.sqrt() - 4.0) * single(31.415_926_54)).cos();
        h0 = 0.5 * (y0 * (1.0 + w) + h0 * (1.0 - w));
        h0p = 0.5 * (y0p * (1.0 + w) + h0p * (1.0 - w));
    }
    (h0, h0p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bessel_matches_known_values() {
        // J0(1) = 0.7651976866, J0'(1) = -J1(1) = -0.4400505857.
        let (j0, j0p) = bessel(C::new(1.0, 0.0));
        assert!((j0.re - 0.765_197_686_6).abs() < 1e-7, "{j0}");
        assert!((j0p.re + 0.440_050_585_7).abs() < 1e-7, "{j0p}");
        // Asymptotic branch: J0(10) = -0.2459357645.
        let (j0, _) = bessel(C::new(10.0, 0.0));
        assert!((j0.re + 0.245_935_764_5).abs() < 1e-6, "{j0}");
    }

    #[test]
    fn hankel_matches_known_values() {
        // H0(1) = J0(1) + j Y0(1) = 0.7651976866 + j 0.0882569642.
        let (h0, _) = hankel(C::new(1.0, 0.0));
        assert!((h0 - C::new(0.765_197_686_6, 0.088_256_964_2)).norm() < 1e-7, "{h0}");
        // Asymptotic: H0(8) = 0.1716508071 + j 0.2235214894. NEC keeps two terms of the
        // expansion, good to about 2e-6 here.
        let (h0, _) = hankel(C::new(8.0, 0.0));
        assert!((h0 - C::new(0.171_650_807_1, 0.223_521_489_4)).norm() < 5e-6, "{h0}");
    }

    #[test]
    fn grid_matches_necs_som2d() {
        // NEC-2's own SOM2D, compiled from the published source with gfortran, for
        // epsr = 3, sigma = 0.0002 S/m at 50 MHz. This port reproduces all 1068 grid values bit
        // for bit there; a few from each region and integral form are pinned here.
        let g = SommerfeldGrid::new(50.0, 3.0, 0.0002);
        assert!((g.epscf - C::new(3.0, -7.190_402_797_436_529e-2)).norm() < 1e-15);
        let cases: [(usize, usize, usize, usize, C); 8] = [
            // Grid 1 at R = 0: the closed-form limit.
            (0, 0, 2, 2, C::new(-1.692_592_927_827_350_2, -88.212_612_904_473_93)),
            (0, 0, 2, 3, C::new(1.692_726_958_552_050_8, 76.664_381_154_536_64)),
            // Grid 1 interior.
            (0, 5, 2, 0, C::new(10.645_742_736_387_309, -62.852_884_342_576_77)),
            (0, 5, 2, 1, C::new(-48.208_907_985_251_24, -51.139_114_137_901_25)),
            // Grid 2: low elevation, the Hankel-function form.
            (1, 9, 1, 0, C::new(-57.940_522_582_638_88, -101.977_514_301_909_6)),
            (1, 9, 1, 2, C::new(-52.969_341_010_009_6, 13.355_115_100_298_79)),
            // Grid 3: high elevation.
            (2, 5, 6, 1, C::new(-17.791_224_585_518_19, -4.815_368_690_908_586)),
            (2, 5, 6, 3, C::new(9.862_348_676_666_706, 45.647_230_153_968_86)),
        ];
        for (k, ir, ith, comp, nec) in cases {
            let ours = g.value(k, ir, ith)[comp];
            assert!(
                (ours - nec).norm() <= 1e-13 * nec.norm(),
                "grid {} R {} theta {} component {}: {ours} vs NEC {nec}",
                k + 1,
                ir + 1,
                ith + 1,
                comp + 1
            );
        }
    }

    #[test]
    fn interpolation_reproduces_grid_points() {
        let g = SommerfeldGrid::new(14.0, 13.0, 0.005);
        // An interior node of grid 1: R = 0.1 (ir 5), θ = 4·π/18 (ith 4).
        let r = 5.0 * DXA[0];
        let th = 4.0 * DYA[0];
        let v = g.interpolate(r, th);
        let node = g.value(0, 5, 4);
        for k in 0..4 {
            assert!((v[k] - node[k]).norm() < 1e-6 * node[k].norm().max(1e-12), "{k}");
        }
    }
}
