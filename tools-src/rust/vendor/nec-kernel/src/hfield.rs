//! The magnetic field of wire currents (`HSFLD`, `HSFLX`, `HFK`, `GH`).
//!
//! Used by the patch equations of the matrix fill — a patch's magnetic-field integral equation
//! sees the wires through their H field — and by near-field `NH` requests.

use crate::cmath::{cdiv, csqrt};
use crate::consts::{ETA, TP};
use crate::ground::ImageGround;
use crate::scaled::WireSeg;
use num_complex::Complex64;

type C = Complex64;
const ZERO: C = C::new(0.0, 0.0);

/// `HSFLD`: the H field at `p` of constant, sine and cosine currents on `src`, with the ground's
/// image. Returns `[k, s, c]`, each a vector.
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn hsfld(src: &WireSeg, p: [f64; 3], ai: f64, g: &ImageGround) -> [[C; 3]; 3] {
    let [cabj, sabj, salpj] = src.dir;
    let [xj, yj, zj] = src.center;
    let [xi, yi, zi] = p;
    let xij = xi - xj;
    let yij = yi - yj;
    let mut e = [[ZERO; 3]; 3];
    let passes = if matches!(g, ImageGround::None) { 1 } else { 2 };
    for ip in 1..=passes {
        let rfl = if ip == 1 { 1.0 } else { -1.0 };
        let salpr = salpj * rfl;
        let zij = zi - rfl * zj;
        let zp = xij * cabj + yij * sabj + zij * salpr;
        let mut rho = [xij - cabj * zp, yij - sabj * zp, zij - salpr * zp];
        let rh = (rho[0] * rho[0] + rho[1] * rho[1] + rho[2] * rho[2] + ai * ai).sqrt();
        if rh <= 1e-10 {
            // NEC zeroes all nine components here, discarding the direct field too.
            e = [[ZERO; 3]; 3];
            continue;
        }
        for r in &mut rho {
            *r /= rh;
        }
        let ph = [
            sabj * rho[2] - salpr * rho[1],
            salpr * rho[0] - cabj * rho[2],
            cabj * rho[1] - sabj * rho[0],
        ];
        let hp = hsflx(src.length, rh, zp);
        if ip == 1 {
            for (row, h) in e.iter_mut().zip(hp) {
                *row = [h * ph[0], h * ph[1], h * ph[2]];
            }
            continue;
        }
        let q: [C; 3] = if let ImageGround::Finite { zrati, screen } = *g {
            let mut zratx = zrati;
            let rmag = (zp * zp + rh * rh).sqrt();
            let xymag = (xij * xij + yij * yij).sqrt();
            if let Some(sc) = screen {
                let xspec = (xi * zj + zi * xj) / (zi + zj);
                let yspec = (yi * zj + zi * yj) / (zi + zj);
                let rhospc = (xspec * xspec + yspec * yspec + sc.t2 * sc.t2).sqrt();
                if rhospc <= sc.length {
                    let z = sc.t1 * rhospc * (rhospc / sc.t2).ln();
                    zratx = cdiv(z * zrati, ETA * zrati + z);
                }
            }
            let (px, py, cth, rrv0) = if xymag > 1e-6 {
                let cth = zij / rmag;
                (
                    -yij / xymag,
                    xij / xymag,
                    cth,
                    csqrt(C::new(1.0, 0.0) - zratx * zratx * (1.0 - cth * cth)),
                )
            } else {
                (0.0, 0.0, 1.0, C::new(1.0, 0.0))
            };
            let rrh = zratx * cth;
            let rrh = -cdiv(rrh - rrv0, rrh + rrv0);
            let rrv = zratx * rrv0;
            let rrv = cdiv(cth - rrv, cth + rrv);
            let qy = (ph[0] * px + ph[1] * py) * (rrv - rrh);
            [qy * px + ph[0] * rrh, qy * py + ph[1] * rrh, ph[2] * rrh]
        } else {
            [C::new(ph[0], 0.0), C::new(ph[1], 0.0), C::new(ph[2], 0.0)]
        };
        for (row, h) in e.iter_mut().zip(hp) {
            for k in 0..3 {
                row[k] -= h * q[k];
            }
        }
    }
    e
}

/// `HSFLX`: the H field (in the φ direction) of constant, sine and cosine currents on a segment of
/// length `s`, at radial distance `rh` and axial offset `zpx`. Returns `[HPK, HPS, HPC]`.
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn hsflx(s: f64, rh: f64, zpx: f64) -> [C; 3] {
    let fj = C::new(0.0, 1.0);
    let fjk = C::new(0.0, -TP);
    let pi8 = 25.132_741_23;
    if rh < 1e-10 {
        return [ZERO; 3];
    }
    let (zp, hss) = if zpx < 0.0 { (-zpx, -1.0) } else { (zpx, 1.0) };
    let dh = 0.5 * s;
    let z1 = zp + dh;
    let z2 = zp - dh;
    let rhz = if z2 < 1e-7 { 1.0 } else { rh / z2 };
    let dk = TP * dh;
    let (sdk, cdk) = dk.sin_cos();
    let (hkr, hki) = hfk(-dk, dk, rh * TP, zp * TP);
    let hpk = C::new(hkr, hki);
    if rhz < 1e-3 {
        let ekr1 = C::new(cdk, sdk) / (z2 * z2);
        let ekr2 = C::new(cdk, -sdk) / (z1 * z1);
        let t1 = TP * (1.0 / z1 - 1.0 / z2);
        let t2 = (fjk * zp).exp() * rh / pi8;
        let hps = t2 * (t1 + (ekr1 + ekr2) * sdk) * hss;
        let hpc = t2 * (-(fj * t1) + (ekr1 - ekr2) * cdk);
        return [hpk, hps, hpc];
    }
    let rh2 = rh * rh;
    let r1 = (rh2 + z1 * z1).sqrt();
    let r2 = (rh2 + z2 * z2).sqrt();
    let ekr1 = (fjk * r1).exp();
    let ekr2 = (fjk * r2).exp();
    let t1 = z1 * ekr1 / r1;
    let t2 = z2 * ekr2 / r2;
    let hps = (cdk * (ekr2 - ekr1) - fj * sdk * (t2 + t1)) * hss;
    let hpc = -sdk * (ekr2 + ekr1) - fj * cdk * (t2 - t1);
    let cons = -fj / (2.0 * TP * rh);
    [hpk, cons * hps, cons * hpc]
}

/// `HFK`: the H field of a uniform current filament, by variable-interval Romberg integration of
/// [`gh`] from `el1` to `el2`.
#[allow(clippy::many_single_char_names)]
fn hfk(el1: f64, el2: f64, rhk: f64, zpk: f64) -> (f64, f64) {
    const NX: u32 = 1;
    const NM: u32 = 65_536;
    const NTS: u32 = 4;
    let rx = 1e-4;
    let rhks = rhk * rhk;
    let g = |zk: f64| gh(zk, zpk, rhks);
    let test = |f1r: f64, f2r: f64, f1i: f64, f2i: f64| {
        let den = f2r.abs().max(f2i.abs());
        if den < 1e-37 {
            (0.0, 0.0)
        } else {
            ((f1r - f2r).abs() / den, (f1i - f2i).abs() / den)
        }
    };
    let mut z = el1;
    let ze = el2;
    let s = ze - z;
    let ep = s / (10.0 * f64::from(NM));
    let zend = ze - ep;
    let (mut sgr, mut sgi) = (0.0, 0.0);
    let mut ns = NX;
    let mut nt = 0;
    let mut g1 = g(z);
    loop {
        let mut dz = s / f64::from(ns);
        if z + dz > ze {
            dz = ze - z;
            if dz.abs() <= ep {
                break;
            }
        }
        let mut dzot = dz * 0.5;
        let mut g3 = g(z + dzot);
        let mut g5 = g(z + dz);
        loop {
            let t00 = ((g1.0 + g5.0) * dzot, (g1.1 + g5.1) * dzot);
            let t01 = ((t00.0 + dz * g3.0) * 0.5, (t00.1 + dz * g3.1) * 0.5);
            let t10 = ((4.0 * t01.0 - t00.0) / 3.0, (4.0 * t01.1 - t00.1) / 3.0);
            let (te1r, te1i) = test(t01.0, t10.0, t01.1, t10.1);
            if te1i <= rx && te1r <= rx {
                sgr += t10.0;
                sgi += t10.1;
                nt += 2;
                break;
            }
            let g2 = g(z + dz * 0.25);
            let g4 = g(z + dz * 0.75);
            let t02 = (
                (t01.0 + dzot * (g2.0 + g4.0)) * 0.5,
                (t01.1 + dzot * (g2.1 + g4.1)) * 0.5,
            );
            let t11 = ((4.0 * t02.0 - t01.0) / 3.0, (4.0 * t02.1 - t01.1) / 3.0);
            let t20 = ((16.0 * t11.0 - t10.0) / 15.0, (16.0 * t11.1 - t10.1) / 15.0);
            let (te2r, te2i) = test(t11.0, t20.0, t11.1, t20.1);
            if (te2i <= rx && te2r <= rx) || ns >= NM {
                if !(te2i <= rx && te2r <= rx) {
                    nt = 0;
                }
                sgr += t20.0;
                sgi += t20.1;
                nt += 1;
                break;
            }
            nt = 0;
            ns *= 2;
            dz = s / f64::from(ns);
            dzot = dz * 0.5;
            g5 = g3;
            g3 = g2;
        }
        z += dz;
        if z >= zend {
            break;
        }
        g1 = g5;
        if nt >= NTS && ns > NX {
            ns /= 2;
            nt = 1;
        }
    }
    (sgr * rhk * 0.5, sgi * rhk * 0.5)
}

/// `GH`: the integrand for the H field of a wire.
fn gh(zk: f64, zpk: f64, rhks: f64) -> (f64, f64) {
    let rs = zk - zpk;
    let rs = rhks + rs * rs;
    let r = rs.sqrt();
    let (skr, ckr) = r.sin_cos();
    let rr2 = 1.0 / rs;
    let rr3 = rr2 / r;
    (skr * rr2 + ckr * rr3, ckr * rr2 - skr * rr3)
}
