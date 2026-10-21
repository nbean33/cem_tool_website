//! Incident-field excitations (`EX` types 1–4, the field branches of `ETMNS`).
//!
//! Instead of a voltage source, the structure is illuminated: by a plane wave (linear, or
//! right- or left-hand elliptic polarization), or by the field of an elementary current source
//! that is not itself part of the structure. The right-hand side is minus the incident field
//! tangential to each segment. Over ground, a plane wave also arrives by reflection, weighted by
//! Fresnel coefficients (or −1 over perfect ground).
//!
//! Surface patches see the incident magnetic field instead: their equations are the MFIE, so the
//! right-hand side there is `t·H` (`= t·(Ŵ × E)/η` for a plane wave).
//!
//! There is no input impedance; the result is the induced current distribution.

use nec_kernel::{SurfacePatch, WireSeg};
use num_complex::Complex64;

type C = Complex64;

/// `DATA TP/6.283185308D+0/` in `ETMNS`.
const TP: f64 = 6.283_185_308;
/// `DATA TA/1.745329252D-02/`: degrees to radians, in the main program.
const TA: f64 = 1.745_329_252e-2;
/// `DATA RETA/2.654420938D-3/` in `ETMNS`: `1/η`.
const RETA: f64 = 2.654_420_938e-3;

/// Polarization of an incident plane wave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarization {
    /// `EX 1`.
    Linear,
    /// `EX 2`: right-hand elliptic.
    RightElliptic,
    /// `EX 3`: left-hand elliptic.
    LeftElliptic,
}

/// One incident field, for one solution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IncidentField {
    /// A plane wave arriving from direction (θ, φ), degrees.
    PlaneWave {
        /// Polarization.
        polarization: Polarization,
        /// Incidence angle θ, degrees.
        theta: f64,
        /// Incidence angle φ, degrees.
        phi: f64,
        /// Polarization angle η, degrees.
        eta: f64,
        /// Axial ratio, for elliptic polarization.
        axial_ratio: f64,
    },
    /// `EX 4`: an elementary current source (current moment `moment` A·m at `position` metres,
    /// pointing at elevation `alpha` and azimuth `beta`, degrees).
    CurrentElement {
        /// Position, metres.
        position: [f64; 3],
        /// Elevation angle α of the element, degrees.
        alpha: f64,
        /// Azimuth angle β of the element, degrees.
        beta: f64,
        /// Current moment, A·m.
        moment: f64,
    },
}

/// What the ground does to a reflected plane wave.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum IncidentGround {
    /// No ground.
    #[default]
    None,
    /// A perfect conductor: both reflection coefficients are −1 (`IPERF = 1`).
    Perfect,
    /// Finite ground with relative wave impedance `ZRATI = 1/√ε_c`. Used for every other ground
    /// type — reflection-coefficient, Sommerfeld, and `GN 3`.
    Finite(C),
}

impl IncidentField {
    /// `ETMNS`: minus the incident tangential field on each segment (V/m), then the two patch
    /// components (`t2`, `t1`) of the incident magnetic field on each patch (A/m).
    #[must_use]
    pub fn rhs(&self, segs: &[WireSeg], patches: &[SurfacePatch], wlam: f64, ground: IncidentGround) -> Vec<C> {
        match *self {
            Self::PlaneWave {
                polarization,
                theta,
                phi,
                eta,
                axial_ratio,
            } => plane_wave(segs, patches, polarization, theta * TA, phi * TA, eta * TA, axial_ratio, ground),
            Self::CurrentElement {
                position,
                alpha,
                beta,
                moment,
            } => current_element(
                segs,
                patches,
                [position[0] / wlam, position[1] / wlam, position[2] / wlam],
                alpha * TA,
                beta * TA,
                moment / (wlam * wlam),
            ),
        }
    }
}

fn phase(arg: f64) -> C {
    C::new(arg.cos(), arg.sin())
}

#[allow(clippy::many_single_char_names, clippy::similar_names, clippy::too_many_arguments)]
fn plane_wave(
    segs: &[WireSeg],
    patches: &[SurfacePatch],
    polarization: Polarization,
    p1: f64,
    p2: f64,
    p3: f64,
    p6: f64,
    ground: IncidentGround,
) -> Vec<C> {
    let (sth, cth) = p1.sin_cos();
    let (sph, cph) = p2.sin_cos();
    let (set, cet) = p3.sin_cos();
    // P: polarization (E) direction; W: propagation direction; Q = W × P.
    let p = [cth * cph * cet - sph * set, cth * sph * cet + cph * set, -sth * cet];
    let w = [-sth * cph, -sth * sph, -cth];
    let q = [w[1] * p[2] - w[2] * p[1], w[2] * p[0] - w[0] * p[2], w[0] * p[1] - w[1] * p[0]];

    let reflection = match ground {
        IncidentGround::None => None,
        IncidentGround::Perfect => Some((C::new(-1.0, 0.0), C::new(-1.0, 0.0))),
        IncidentGround::Finite(zrati) => {
            let rrv = nec_kernel::cmath::csqrt(C::new(1.0, 0.0) - zrati * zrati * sth * sth);
            let rrh = zrati * cth;
            let rrh = nec_kernel::cmath::cdiv(rrh - rrv, rrh + rrv);
            let rrv = zrati * rrv;
            let rrv = -nec_kernel::cmath::cdiv(cth - rrv, cth + rrv);
            Some((rrv, rrh))
        }
    };

    // The complex field direction: P for linear polarization, P ± j·r·Q for elliptic.
    let tt1 = match polarization {
        Polarization::Linear => C::new(0.0, 0.0),
        Polarization::RightElliptic => -C::new(0.0, 1.0) * p6,
        Polarization::LeftElliptic => C::new(0.0, 1.0) * p6,
    };
    let c = [p[0] + tt1 * q[0], p[1] + tt1 * q[1], p[2] + tt1 * q[2]];

    let dot = |v: &[C; 3], d: [f64; 3]| v[0] * d[0] + v[1] * d[1] + v[2] * d[2];
    let mut e: Vec<C> = segs
        .iter()
        .map(|s| {
            let arg = -TP * (w[0] * s.center[0] + w[1] * s.center[1] + w[2] * s.center[2]);
            -(dot(&c, s.dir) * phase(arg))
        })
        .collect();

    if let Some((rrv, rrh)) = reflection {
        let tt = (c[1] * cph - c[0] * sph) * (rrh - rrv);
        let r = [rrv * c[0] - tt * sph, rrv * c[1] + tt * cph, -(rrv * c[2])];
        for (ei, s) in e.iter_mut().zip(segs) {
            let arg = -TP * (w[0] * s.center[0] + w[1] * s.center[1] - w[2] * s.center[2]);
            *ei -= dot(&r, s.dir) * phase(arg);
        }
    }

    // Patches: the magnetic field direction Q − j·r·P (the same rotation as E's), and its
    // reflection with the roles of the two coefficients exchanged.
    let h = [q[0] - tt1 * p[0], q[1] - tt1 * p[1], q[2] - tt1 * p[2]];
    let reflected = reflection.map(|(rrv, rrh)| {
        let tt = (h[1] * cph - h[0] * sph) * (rrv - rrh);
        [-(rrh * h[0] - tt * sph), -(rrh * h[1] + tt * cph), rrh * h[2]]
    });
    for pt in patches {
        let [x, y, z] = pt.center;
        let add = |v: &[C; 3], arg: f64| {
            let tt = phase(arg) * pt.salp * RETA;
            [dot(v, pt.t2) * tt, dot(v, pt.t1) * tt]
        };
        let mut pair = add(&h, -TP * (w[0] * x + w[1] * y + w[2] * z));
        if let Some(r) = &reflected {
            let im = add(r, -TP * (w[0] * x + w[1] * y - w[2] * z));
            pair = [pair[0] + im[0], pair[1] + im[1]];
        }
        e.extend(pair);
    }
    e
}

#[allow(clippy::many_single_char_names)]
fn current_element(segs: &[WireSeg], patches: &[SurfacePatch], at: [f64; 3], p4: f64, p5: f64, p6: f64) -> Vec<C> {
    let w = [p4.cos() * p5.cos(), p4.cos() * p5.sin(), p4.sin()];
    // `DS=P6*59.958`: a single-precision literal in the source.
    let ds = p6 * nec_kernel::cmath::single(59.958);
    let dsh = p6 / (2.0 * TP);
    // The element's geometry seen from a point: unit vector from the element, cos and sin of the
    // angle off its axis, and the direction normal to the axis in the plane of the two.
    let view = |c: [f64; 3]| -> Option<(f64, f64, f64, [f64; 3])> {
        let mut p = [c[0] - at[0], c[1] - at[1], c[2] - at[2]];
        let rs = p[0] * p[0] + p[1] * p[1] + p[2] * p[2];
        if rs < 1e-30 {
            return None;
        }
        let r = rs.sqrt();
        for v in &mut p {
            *v /= r;
        }
        let cth = p[0] * w[0] + p[1] * w[1] + p[2] * w[2];
        let sth = (1.0 - cth * cth).sqrt();
        let mut q = [p[0] - w[0] * cth, p[1] - w[1] * cth, p[2] - w[2] * cth];
        let qn = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
        if qn < 1e-30 {
            q = [1.0, 0.0, 0.0];
        } else {
            for v in &mut q {
                *v /= qn;
            }
        }
        Some((r, cth, sth, q))
    };
    let mut e: Vec<C> = segs.iter()
        .map(|s| {
            let mut p = [s.center[0] - at[0], s.center[1] - at[1], s.center[2] - at[2]];
            let rs = p[0] * p[0] + p[1] * p[1] + p[2] * p[2];
            if rs < 1e-30 {
                // NEC leaves this element of E untouched (zero here).
                return C::new(0.0, 0.0);
            }
            let r = rs.sqrt();
            for v in &mut p {
                *v /= r;
            }
            let cth = p[0] * w[0] + p[1] * w[1] + p[2] * w[2];
            let sth = (1.0 - cth * cth).sqrt();
            let mut q = [p[0] - w[0] * cth, p[1] - w[1] * cth, p[2] - w[2] * cth];
            let qn = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
            if qn < 1e-30 {
                q = [1.0, 0.0, 0.0];
            } else {
                for v in &mut q {
                    *v /= qn;
                }
            }
            let tt1 = phase(-TP * r);
            let tt2 = C::new(1.0, -1.0 / (r * TP)) / rs;
            let er = ds * tt1 * tt2 * cth;
            let et = 0.5 * ds * tt1 * (C::new(0.0, TP / r) + tt2) * sth;
            let ezh = er * cth - et * sth;
            let erh = er * sth + et * cth;
            let c = [ezh * w[0] + erh * q[0], ezh * w[1] + erh * q[1], ezh * w[2] + erh * q[2]];
            -(c[0] * s.dir[0] + c[1] * s.dir[1] + c[2] * s.dir[2])
        })
        .collect();
    // Patches: the element's magnetic field, circling its axis.
    for pt in patches {
        let pair = view(pt.center).map_or([C::new(0.0, 0.0); 2], |(r, _, sth, q)| {
            let ph = [w[1] * q[2] - w[2] * q[1], w[2] * q[0] - w[0] * q[2], w[0] * q[1] - w[1] * q[0]];
            let tt2 = dsh * phase(-TP * r) * C::new(1.0 / r, TP) / r * sth * pt.salp;
            let c = [tt2 * ph[0], tt2 * ph[1], tt2 * ph[2]];
            let dot = |t: [f64; 3]| c[0] * t[0] + c[1] * t[1] + c[2] * t[2];
            [dot(pt.t2), dot(pt.t1)]
        });
        e.extend(pair);
    }
    e
}
