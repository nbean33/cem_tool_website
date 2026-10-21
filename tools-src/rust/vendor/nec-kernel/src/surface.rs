//! Fields of surface-patch currents (`UNERE`, `HINTG`, `PCINT`).
//!
//! A patch carries a uniform surface current density over its area, treated as a single current
//! element at its centre. Its electric field ([`unere`]) drives the wire equations; its magnetic
//! field ([`hintg`]) drives the patch equations of NEC's magnetic-field integral equation. Where a
//! wire ends on a patch, the four quarter patches around the junction are integrated over a
//! 10 × 10 grid instead, together with the singular radial current that carries the wire's
//! current into the surface ([`pcint`]).
//!
//! Units are the kernel's: lengths in wavelengths, areas in square wavelengths, `k = 2π`.

use crate::cmath::{cdiv, csqrt};
use crate::ground::ImageGround;
use nec_core::Frequency;
use nec_geom::Geometry;
use num_complex::Complex64;

type C = Complex64;
const ZERO: C = C::new(0.0, 0.0);

/// One surface patch in kernel units, as NEC's `/DATA/` arrays hold it after frequency scaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfacePatch {
    /// Centre, wavelengths.
    pub center: [f64; 3],
    /// First unit tangent.
    pub t1: [f64; 3],
    /// Second unit tangent.
    pub t2: [f64; 3],
    /// Area, square wavelengths.
    pub area: f64,
    /// `+1`, or `−1` on a reflected patch (see [`nec_geom::Patch::salp`]).
    pub salp: f64,
}

/// Convert a geometry's patches to kernel units at one frequency: `X·FR` and `BI·FR²`.
#[must_use]
pub fn scale_patches(geom: &Geometry, f: Frequency) -> Vec<SurfacePatch> {
    let fr = 1.0 / f.wavelength_m();
    geom.patches
        .iter()
        .map(|p| SurfacePatch {
            center: [p.center.x * fr, p.center.y * fr, p.center.z * fr],
            t1: [p.t1.x, p.t1.y, p.t1.z],
            t2: [p.t2.x, p.t2.y, p.t2.z],
            area: p.area * fr * fr,
            salp: p.salp,
        })
        .collect()
}

/// A field vector for unit current along each tangent: `[t1 field, t2 field]`.
pub type PatchField = [[C; 3]; 2];

/// `UNERE`: the electric field at `p` of unit surface current along `t1` and along `t2` on `src`.
///
/// With `image` set this is the field of the patch's image in the ground instead (`IPGND = 2`):
/// negated for perfect ground, otherwise weighted by Fresnel coefficients at the specular angle.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn unere(src: &SurfacePatch, p: [f64; 3], image: bool, ground: &ImageGround) -> PatchField {
    const TPI: f64 = 6.283_185_308;
    // ETA/(8π²).
    const CONST: f64 = 4.771_341_188;
    let [t1x, t1y, mut t1z] = src.t1;
    let [t2x, t2y, mut t2z] = src.t2;
    let mut zr = src.center[2];
    if image {
        zr = -zr;
        t1z = -t1z;
        t2z = -t2z;
    }
    let rx = p[0] - src.center[0];
    let ry = p[1] - src.center[1];
    let rz = p[2] - zr;
    let r2 = rx * rx + ry * ry + rz * rz;
    if r2 <= 1e-20 {
        return [[ZERO; 3]; 2];
    }
    let r = r2.sqrt();
    let tt1 = -TPI * r;
    let tt2 = tt1 * tt1;
    let rt = r2 * r;
    let er = C::new(tt1.sin(), -tt1.cos()) * (CONST * src.area);
    let q1 = C::new(tt2 - 1.0, tt1) * er / rt;
    let q2 = C::new(3.0 - tt2, -3.0 * tt1) * er / (rt * r2);
    let along = |tx: f64, ty: f64, tz: f64| {
        let e = q2 * (tx * rx + ty * ry + tz * rz);
        [q1 * tx + e * rx, q1 * ty + e * ry, q1 * tz + e * rz]
    };
    let mut f = [along(t1x, t1y, t1z), along(t2x, t2y, t2z)];
    if !image {
        return f;
    }
    match *ground {
        ImageGround::None => f,
        ImageGround::Perfect => {
            for v in f.iter_mut().flatten() {
                *v = -*v;
            }
            f
        }
        ImageGround::Finite { zrati, .. } => {
            let xymag = (rx * rx + ry * ry).sqrt();
            let (px, py, cth, rrv) = if xymag > 1e-6 {
                let cth = rz / (xymag * xymag + rz * rz).sqrt();
                (
                    -ry / xymag,
                    rx / xymag,
                    cth,
                    csqrt(C::new(1.0, 0.0) - zrati * zrati * (1.0 - cth * cth)),
                )
            } else {
                (0.0, 0.0, 1.0, C::new(1.0, 0.0))
            };
            let rrh = zrati * cth;
            let rrh = cdiv(rrh - rrv, rrh + rrv);
            let rrv = zrati * rrv;
            let rrv = -cdiv(cth - rrv, cth + rrv);
            for v in &mut f {
                let edp = (v[0] * px + v[1] * py) * (rrh - rrv);
                *v = [v[0] * rrv + edp * px, v[1] * rrv + edp * py, v[2] * rrv];
            }
            f
        }
    }
}

/// `HINTG`: the magnetic field at `p` of unit surface current along `t1` and `t2` on `src`, its
/// ground image included.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn hintg(src: &SurfacePatch, p: [f64; 3], ground: &ImageGround) -> PatchField {
    const FPI: f64 = 12.566_370_62;
    const TP: f64 = 6.283_185_308;
    let rx = p[0] - src.center[0];
    let ry = p[1] - src.center[1];
    let passes = if matches!(ground, ImageGround::None) {
        1
    } else {
        2
    };
    let mut h = [[ZERO; 3]; 2];
    for ip in 1..=passes {
        let rfl = if ip == 1 { 1.0 } else { -1.0 };
        let rz = p[2] - src.center[2] * rfl;
        let rsq = rx * rx + ry * ry + rz * rz;
        if rsq < 1e-20 {
            continue;
        }
        let r = rsq.sqrt();
        let rk = TP * r;
        let (sr, cr) = rk.sin_cos();
        let gam = -(C::new(cr, -sr) + rk * C::new(sr, cr)) / (FPI * rsq * r) * src.area;
        let (exc, eyc, ezc) = (gam * rx, gam * ry, gam * rz);
        let curl = |t: [f64; 3]| {
            let tz = t[2] * rfl;
            [
                eyc * tz - ezc * t[1],
                ezc * t[0] - exc * tz,
                exc * t[1] - eyc * t[0],
            ]
        };
        let mut f = [curl(src.t1), curl(src.t2)];
        if ip == 2 {
            match *ground {
                ImageGround::None => {}
                ImageGround::Perfect => {
                    for v in f.iter_mut().flatten() {
                        *v = -*v;
                    }
                }
                ImageGround::Finite { zrati, .. } => {
                    let xymag = (rx * rx + ry * ry).sqrt();
                    let (px, py, cth, rrv) = if xymag > 1e-6 {
                        let cth = rz / r;
                        (
                            -ry / xymag,
                            rx / xymag,
                            cth,
                            csqrt(C::new(1.0, 0.0) - zrati * zrati * (1.0 - cth * cth)),
                        )
                    } else {
                        (0.0, 0.0, 1.0, C::new(1.0, 0.0))
                    };
                    let rrh = zrati * cth;
                    let rrh = cdiv(rrh - rrv, rrh + rrv);
                    let rrv = zrati * rrv;
                    let rrv = -cdiv(cth - rrv, cth + rrv);
                    for v in &mut f {
                        let g = (v[0] * px + v[1] * py) * (rrv - rrh);
                        *v = [v[0] * rrh + g * px, v[1] * rrh + g * py, v[2] * rrh];
                    }
                }
            }
        }
        for (acc, v) in h.iter_mut().zip(f) {
            for k in 0..3 {
                acc[k] += v[k];
            }
        }
    }
    h
}

/// `PCINT`: the field along `dir` at `p` of the four quarter patches around a wire–surface
/// junction, `first` being the first quarter (the (+t1, +t2) one).
///
/// Returns NEC's nine values: the field of unit `t1` current on quarters 1–4, of unit `t2`
/// current on quarters 1–4, and of the singular radial current (unit total current flowing out
/// of the junction). The integration runs over a 10 × 10 grid covering the whole original patch,
/// each cell's current shared among the quarters by bilinear weights.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn pcint(first: &SurfacePatch, p: [f64; 3], dir: [f64; 3]) -> [C; 9] {
    const TPI: f64 = 6.283_185_308;
    const NINT: i32 = 10;
    let d = first.area.sqrt() * 0.5;
    let ds = 4.0 * d / f64::from(NINT);
    let da = ds * ds;
    let gcon = 1.0 / first.area;
    let fcon = 1.0 / (2.0 * TPI * d);
    let [t1x, t1y, t1z] = first.t1;
    let [t2x, t2y, t2z] = first.t2;
    let mut s1 = d + ds * 0.5;
    let mut xss = [
        first.center[0] + s1 * (t1x + t2x),
        first.center[1] + s1 * (t1y + t2y),
        first.center[2] + s1 * (t1z + t2z),
    ];
    s1 += d;
    let s2x = s1;
    let mut e = [ZERO; 9];
    let mut cell = SurfacePatch { area: da, ..*first };
    for _ in 0..NINT {
        s1 -= ds;
        let mut s2 = s2x;
        xss = [xss[0] - ds * t1x, xss[1] - ds * t1y, xss[2] - ds * t1z];
        cell.center = xss;
        for _ in 0..NINT {
            s2 -= ds;
            cell.center = [
                cell.center[0] - ds * t2x,
                cell.center[1] - ds * t2y,
                cell.center[2] - ds * t2z,
            ];
            let [fk, fs] = unere(&cell, p, false, &ImageGround::None);
            let exk = fk[0] * dir[0] + fk[1] * dir[1] + fk[2] * dir[2];
            let exs = fs[0] * dir[0] + fs[1] * dir[1] + fs[2] * dir[2];
            let g1 = (d + s1) * (d + s2) * gcon;
            let g2 = (d - s1) * (d + s2) * gcon;
            let g3 = (d - s1) * (d - s2) * gcon;
            let g4 = (d + s1) * (d - s2) * gcon;
            let f2 = (s1 * s1 + s2 * s2) * TPI;
            let f1 = s1 / f2 - (g1 - g2 - g3 + g4) * fcon;
            let f2 = s2 / f2 - (g1 + g2 - g3 - g4) * fcon;
            e[0] += exk * g1;
            e[1] += exk * g2;
            e[2] += exk * g3;
            e[3] += exk * g4;
            e[4] += exs * g1;
            e[5] += exs * g2;
            e[6] += exs * g3;
            e[7] += exs * g4;
            e[8] += exk * f1 + exs * f2;
        }
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(z: f64) -> SurfacePatch {
        SurfacePatch {
            center: [0.0, 0.0, z],
            t1: [1.0, 0.0, 0.0],
            t2: [0.0, 1.0, 0.0],
            area: 0.01,
            salp: 1.0,
        }
    }

    #[test]
    fn a_patch_field_is_that_of_a_current_element() {
        // Broadside to an x-directed element at distance r, far out: |E| = η·J·A/(2λr) with
        // CONST = η/(8π²) folded into the kernel's normalization: |E| ≈ 4π²·CONST·A/r.
        let r = 50.0;
        let [ek, _] = unere(&square(0.0), [0.0, r, 0.0], false, &ImageGround::None);
        let expected = 4.0 * std::f64::consts::PI.powi(2) * 4.771_341_188 * 0.01 / r;
        assert!(
            (ek[0].norm() / expected - 1.0).abs() < 1e-3,
            "{} vs {expected}",
            ek[0].norm()
        );
        assert!(ek[1].norm() < 1e-12 && ek[2].norm() < 1e-12);
    }

    #[test]
    fn a_perfect_ground_image_is_the_mirror() {
        let src = square(0.3);
        let p = [0.2, 0.1, 0.5];
        let image = unere(&src, p, true, &ImageGround::Perfect);
        let mirrored = SurfacePatch {
            center: [0.0, 0.0, -0.3],
            ..src
        };
        let direct = unere(&mirrored, p, false, &ImageGround::None);
        for (a, b) in image.iter().flatten().zip(direct.iter().flatten()) {
            assert!((*a + *b).norm() < 1e-12);
        }
    }

    #[test]
    fn the_h_field_is_perpendicular_to_the_current() {
        let [hk, hs] = hintg(&square(0.0), [0.3, 0.4, 0.5], &ImageGround::None);
        assert!(hk[0].norm() < 1e-15, "t1 = x current has no Hx");
        assert!(hs[1].norm() < 1e-15, "t2 = y current has no Hy");
    }

    #[test]
    fn junction_weights_split_the_current_among_the_quarters() {
        // Far out along the normal (where every part of the patch is equally distant) the
        // quarters' fields add up to the whole patch's.
        let quarter = SurfacePatch {
            center: [0.025, 0.025, 0.0],
            area: 0.0025,
            ..square(0.0)
        };
        let p = [0.0, 0.0, 10.0];
        let e = pcint(&quarter, p, [1.0, 0.0, 0.0]);
        let whole = unere(&square(0.0), p, false, &ImageGround::None)[0][0];
        let sum = e[0] + e[1] + e[2] + e[3];
        assert!((sum / whole - 1.0).norm() < 1e-3, "{sum} vs {whole}");
    }
}
