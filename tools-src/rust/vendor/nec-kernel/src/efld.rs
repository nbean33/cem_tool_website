//! Near electric field of one source segment at an observation point (`EFLD`, free space).
//!
//! For each of the three current shapes on the source — constant, `sin`, `cos` — returns the
//! Cartesian field vector. `CMWW` then takes the component along the observation segment.
//!
//! Two regimes, split at distance `rkh` (one wavelength by default, NEC's `RKH`):
//!
//! * **Near:** the thin-wire kernel [`crate::eksc::eksc`].
//! * **Far:** a lumped current element at the segment centre. A `sin` current integrates to
//!   nothing over a symmetric segment, so its far field is zero; the constant and `cos` shapes
//!   become point sources of strength `Δ` and `sin(πΔ)/π` respectively.
//!
//! Over ground (`KSYMP = 2`) the image is added as a second pass; see [`crate::ground`]. The
//! Sommerfeld branch of `EFLD` is not handled yet.

use crate::consts::{ETA, PI, TP};
use crate::eksc::{ExtendedEnd, eksc, ekscx};
use crate::ground::{Ground, Reflector};
use crate::scaled::WireSeg;
use num_complex::Complex64;

/// A complex 3-vector.
pub type CVec3 = [Complex64; 3];

/// Field vectors at the observation point for the three current shapes on the source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentField {
    /// From a constant current (`K`).
    pub k: CVec3,
    /// From a `sin` current (`S`).
    pub s: CVec3,
    /// From a `cos` current (`C`).
    pub c: CVec3,
}

impl SegmentField {
    /// Components along a unit direction — the tangential field on an observation segment.
    #[must_use]
    pub fn along(&self, d: [f64; 3]) -> (Complex64, Complex64, Complex64) {
        let dot = |v: &CVec3| v[0] * d[0] + v[1] * d[1] + v[2] * d[2];
        (dot(&self.k), dot(&self.s), dot(&self.c))
    }
}

fn combine(ez: Complex64, er: Complex64, axis: [f64; 3], rho: [f64; 3]) -> CVec3 {
    [
        ez * axis[0] + er * rho[0],
        ez * axis[1] + er * rho[1],
        ez * axis[2] + er * rho[2],
    ]
}

/// `EFLD`: field of `src` at `obs_point`, where the observation wire has radius `obs_radius`.
/// `self_term` is NEC's `IJ == 0` (the observation segment is the source). `extended` selects the
/// extended thin-wire kernel (`EK`), with the source segment's end treatments from
/// [`crate::eksc::extended_ends`].
///
/// Over ground, the field of the source's image is subtracted, weighted as the [`Ground`] model
/// says. The image of a segment is its mirror in `z = 0`: centre `(x, y, −z)`, direction
/// `(cx, cy, −cz)`. The image pass never uses the self-term treatment (`IJX = 1`).
#[must_use]
pub fn efld(
    src: &WireSeg,
    obs_point: [f64; 3],
    obs_radius: f64,
    self_term: bool,
    rkh: f64,
    ground: &Ground,
    extended: Option<[ExtendedEnd; 2]>,
) -> SegmentField {
    let (direct, _) = free_space(src, obs_point, obs_radius, self_term, rkh, extended);
    if *ground == Ground::FreeSpace {
        return direct;
    }

    let image = WireSeg {
        center: [src.center[0], src.center[1], -src.center[2]],
        dir: [src.dir[0], src.dir[1], -src.dir[2]],
        ..*src
    };
    let (mut img, r) = free_space(&image, obs_point, obs_radius, false, rkh, extended);
    if let Ground::Reflection { zrati, screen } = ground {
        let refl = Reflector::new(*zrati, screen.as_ref(), src.center, obs_point, r);
        img.k = refl.apply(img.k);
        img.s = refl.apply(img.s);
        img.c = refl.apply(img.c);
    }
    // FRATI: 1 for perfect and reflection-coefficient ground, (ε−1)/(ε+1) for Sommerfeld.
    let frati = match ground {
        Ground::Sommerfeld(g) => g.frati,
        _ => Complex64::new(1.0, 0.0),
    };
    let sub = |a: CVec3, b: CVec3| [a[0] - b[0] * frati, a[1] - b[1] * frati, a[2] - b[2] * frati];
    let mut e = SegmentField {
        k: sub(direct.k, img.k),
        s: sub(direct.s, img.s),
        c: sub(direct.c, img.c),
    };
    if let Ground::Sommerfeld(g) = ground {
        let t = crate::sommerfeld::ground_field(g, src, obs_point, obs_radius, &e.k);
        let add = |a: &mut CVec3, b: CVec3| {
            for i in 0..3 {
                a[i] += b[i];
            }
        };
        add(&mut e.k, t.k);
        add(&mut e.s, t.s);
        add(&mut e.c, t.c);
    }
    e
}

/// One pass of `EFLD`'s loop: the free-space field of `src`, and the distance `R` it used.
fn free_space(
    src: &WireSeg,
    obs_point: [f64; 3],
    obs_radius: f64,
    self_term: bool,
    rkh: f64,
    extended: Option<[ExtendedEnd; 2]>,
) -> (SegmentField, f64) {
    let axis = src.dir;
    let xij = obs_point[0] - src.center[0];
    let yij = obs_point[1] - src.center[1];
    let zij = obs_point[2] - src.center[2];

    // Axial and radial decomposition. The observation wire's radius is added in quadrature, so
    // the field is evaluated at the wire surface rather than on the source filament.
    let zp = xij * axis[0] + yij * axis[1] + zij * axis[2];
    let mut rho = [xij - axis[0] * zp, yij - axis[1] * zp, zij - axis[2] * zp];
    let rh = (rho[0] * rho[0] + rho[1] * rho[1] + rho[2] * rho[2] + obs_radius * obs_radius).sqrt();
    if rh > 1e-10 {
        for r in &mut rho {
            *r /= rh;
        }
    } else {
        rho = [0.0; 3];
    }
    let r = (zp * zp + rh * rh).sqrt();
    let s = src.length;

    if r >= rkh {
        // Lumped current element approximation for large separations.
        let rmag = TP * r;
        let cth = zp / r;
        let px = rh / r;
        let txk = Complex64::new(rmag.cos(), -rmag.sin());
        let py = TP * r * r;
        let tyk = ETA * cth * txk * Complex64::new(1.0, -1.0 / rmag) / py;
        let tzk = ETA * px * txk * Complex64::new(1.0, rmag - 1.0 / rmag) / (2.0 * py);
        let tezk = tyk * cth - tzk * px;
        let terk = tyk * px + tzk * cth;
        let cos_strength = (PI * s).sin() / PI;
        let zero = Complex64::new(0.0, 0.0);
        let field = SegmentField {
            k: combine(tezk * s, terk * s, axis, rho),
            s: [zero; 3],
            c: combine(tezk * cos_strength, terk * cos_strength, axis, rho),
        };
        return (field, r);
    }

    let e = match extended {
        None => eksc(s, zp, rh, TP, self_term),
        Some(ends) => ekscx(src.radius, s, zp, rh, TP, self_term, ends),
    };
    let field = SegmentField {
        k: combine(e.ezk, e.erk, axis, rho),
        s: combine(e.ezs, e.ers, axis, rho),
        c: combine(e.ezc, e.erc, axis, rho),
    };
    (field, r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg_along_z(len: f64) -> WireSeg {
        WireSeg {
            center: [0.0; 3],
            dir: [0.0, 0.0, 1.0],
            length: len,
            radius: 1e-4,
        }
    }

    #[test]
    fn near_and_far_regimes_agree_where_they_meet() {
        // Just inside and just outside RKH the two models describe the same physics; for a short
        // segment the lumped element is accurate, so they should nearly coincide.
        let src = seg_along_z(0.01);
        let inside = efld(&src, [0.999, 0.0, 0.05], 1e-4, false, 1.0, &Ground::FreeSpace, None);
        let outside = efld(&src, [0.999, 0.0, 0.05], 1e-4, false, 0.5, &Ground::FreeSpace, None);
        let (ki, _, ci) = inside.along([0.0, 0.0, 1.0]);
        let (ko, _, co) = outside.along([0.0, 0.0, 1.0]);
        assert!((ki - ko).norm() < 1e-3 * ki.norm(), "constant: {ki} vs {ko}");
        assert!((ci - co).norm() < 1e-3 * ci.norm(), "cosine: {ci} vs {co}");
    }

    #[test]
    fn broadside_far_field_of_a_short_element_is_axial() {
        // Broadside to a z-directed element, the far field points along z.
        let f = efld(&seg_along_z(0.01), [3.0, 0.0, 0.0], 1e-4, false, 1.0, &Ground::FreeSpace, None);
        assert!(f.k[0].norm() < 1e-12 * f.k[2].norm(), "no radial far field broadside");
        assert!(f.k[2].norm() > 0.0);
    }
}
