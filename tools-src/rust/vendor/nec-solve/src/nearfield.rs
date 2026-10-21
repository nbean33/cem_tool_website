//! Near fields at points in space (`NE`, `NH`; NEC's `NFPAT`, `NEFLD`, `NHFLD`, `HSFLD`, `HSFLX`,
//! `HFK`, `GH`).
//!
//! The electric field at a point is the sum over segments of each segment's field (the same
//! `EFLD` the matrix fill uses, ground and extended kernel included) weighted by its current
//! coefficients. The magnetic field has its own segment kernel, with the ground's image weighted
//! by Fresnel coefficients; over Sommerfeld ground NEC instead takes a finite-difference curl of
//! the electric field.

use crate::farfield::FarFieldGround;
use nec_kernel::cmath::single;
use nec_kernel::hfield::hsfld;
use nec_kernel::surface::{hintg, unere};
use nec_kernel::{ExtendedEnd, Ground, ImageGround, SurfacePatch, WireSeg, efld};
use num_complex::Complex64;

type C = Complex64;
const ZERO: C = C::new(0.0, 0.0);
const TA: f64 = 1.745_329_252e-2;

/// What an `NE` or `NH` card asks for: `NE I1 NX NY NZ  X Y Z  DX DY DZ`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearFieldRequest {
    /// `NH`: the magnetic field; otherwise (`NE`) the electric field.
    pub magnetic: bool,
    /// `I1 = 1`: spherical coordinates `(r, φ, θ)` in metres and degrees; otherwise `(x, y, z)`.
    pub spherical: bool,
    /// Number of points along each coordinate.
    pub count: [usize; 3],
    /// First point.
    pub start: [f64; 3],
    /// Step along each coordinate.
    pub step: [f64; 3],
}

impl NearFieldRequest {
    /// The observation points in metres, in NEC's order: the third coordinate outermost, the
    /// first innermost, each accumulated by repeated addition.
    #[must_use]
    pub fn points(&self) -> Vec<[f64; 3]> {
        let mut out = Vec::new();
        let mut c3 = self.start[2] - self.step[2];
        for _ in 0..self.count[2].max(1) {
            c3 += self.step[2];
            let (cth, sth) = ((TA * c3).cos(), (TA * c3).sin());
            let mut c2 = self.start[1] - self.step[1];
            for _ in 0..self.count[1].max(1) {
                c2 += self.step[1];
                let (cph, sph) = ((TA * c2).cos(), (TA * c2).sin());
                let mut c1 = self.start[0] - self.step[0];
                for _ in 0..self.count[0].max(1) {
                    c1 += self.step[0];
                    out.push(if self.spherical {
                        [c1 * sth * cph, c1 * sth * sph, c1 * cth]
                    } else {
                        [c1, c2, c3]
                    });
                }
            }
        }
        out
    }
}

/// What the near-field routines need from a solution.
pub struct NearFieldContext<'a> {
    /// Segments in wavelengths.
    pub segs: &'a [WireSeg],
    /// Current coefficients `(A, B, C)` per segment, kernel units.
    pub abc: &'a [[C; 3]],
    /// Surface patches, kernel units.
    pub patches: &'a [SurfacePatch],
    /// Current density on each patch, `(x, y, z)`, A/m.
    pub patch_currents: &'a [[C; 3]],
    /// The ground for patch fields.
    pub image: &'a ImageGround,
    /// The ground, as the fill sees it (for `E`).
    pub ground: &'a Ground,
    /// The ground, as the far-field routines see it (for `H`'s image).
    pub far_ground: &'a FarFieldGround,
    /// Extended-kernel end treatments, if `EK`.
    pub extended: Option<&'a [[ExtendedEnd; 2]]>,
    /// Lumped-approximation distance (`KH`).
    pub rkh: f64,
}

impl NearFieldContext<'_> {
    /// The radius to observe with: a segment's radius when the point lies inside that segment's
    /// wire (within 0.5001 of its half-length along it and 0.9 of its radius² across), else 0.
    fn observation_radius(&self, p: [f64; 3]) -> f64 {
        for s in self.segs {
            let d = [p[0] - s.center[0], p[1] - s.center[1], p[2] - s.center[2]];
            let zp = s.dir[0] * d[0] + s.dir[1] * d[1] + s.dir[2] * d[2];
            if zp.abs() > single(0.5001) * s.length {
                continue;
            }
            let across = d[0] * d[0] + d[1] * d[1] + d[2] * d[2] - zp * zp;
            if across > single(0.9) * s.radius * s.radius {
                continue;
            }
            return s.radius;
        }
        0.0
    }

    /// `NEFLD`: the electric field at `p` (wavelengths), volts per metre.
    #[must_use]
    pub fn electric(&self, p: [f64; 3]) -> [C; 3] {
        let ax = self.observation_radius(p);
        let mut e = [ZERO; 3];
        for (j, (s, abc)) in self.segs.iter().zip(self.abc).enumerate() {
            let f = efld(s, p, ax, false, self.rkh, self.ground, self.extended.map(|x| x[j]));
            for (k, ek) in e.iter_mut().enumerate() {
                *ek += f.k[k] * abc[0] + f.s[k] * abc[1] + f.c[k] * abc[2];
            }
        }
        let images: &[bool] = if matches!(self.image, ImageGround::None) { &[false] } else { &[false, true] };
        for (pt, j) in self.patches.iter().zip(self.patch_currents) {
            let (acx, bcx) = tangential(pt, j);
            for &image in images {
                let [fk, fs] = unere(pt, p, image, self.image);
                for k in 0..3 {
                    e[k] += acx * fk[k] + bcx * fs[k];
                }
            }
        }
        e
    }

    /// `NHFLD`: the magnetic field at `p` (wavelengths), amperes per metre.
    #[must_use]
    pub fn magnetic(&self, p: [f64; 3]) -> [C; 3] {
        if let Ground::Sommerfeld(_) = self.ground {
            // A central-difference curl of E: H = j/(ωμ)·∇×E, CON = j/(2π·η) per wavelength.
            let delt = single(1e-3);
            let con = C::new(0.0, single(4.2246e-4));
            let at = |dx: f64, dy: f64, dz: f64| self.electric([p[0] + dx, p[1] + dy, p[2] + dz]);
            let (px, mx) = (at(delt, 0.0, 0.0), at(-delt, 0.0, 0.0));
            let (py, my) = (at(0.0, delt, 0.0), at(0.0, -delt, 0.0));
            let (pz, mz) = (at(0.0, 0.0, delt), at(0.0, 0.0, -delt));
            return [
                con * (py[2] - my[2] - pz[1] + mz[1]) / (2.0 * delt),
                con * (pz[0] - mz[0] - px[2] + mx[2]) / (2.0 * delt),
                con * (px[1] - mx[1] - py[0] + my[0]) / (2.0 * delt),
            ];
        }
        let ax = self.observation_radius(p);
        let image = self.far_ground.image();
        let mut h = [ZERO; 3];
        for (s, abc) in self.segs.iter().zip(self.abc) {
            let f = hsfld(s, p, ax, &image);
            for (k, hk) in h.iter_mut().enumerate() {
                *hk += f[0][k] * abc[0] + f[1][k] * abc[1] + f[2][k] * abc[2];
            }
        }
        for (pt, j) in self.patches.iter().zip(self.patch_currents) {
            let (acx, bcx) = tangential(pt, j);
            let [hk, hs] = hintg(pt, p, self.image);
            for k in 0..3 {
                h[k] += acx * hk[k] + bcx * hs[k];
            }
        }
        h
    }
}

/// A patch current's components along the patch's tangents.
fn tangential(pt: &SurfacePatch, j: &[C; 3]) -> (C, C) {
    let along = |t: [f64; 3]| t[0] * j[0] + t[1] * j[1] + t[2] * j[2];
    (along(pt.t1), along(pt.t2))
}

