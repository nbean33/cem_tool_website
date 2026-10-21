//! The thin-wire kernel (`EKSC`, `GX`): fields of sine, cosine and constant current filaments.
//!
//! A current `sin(ks)` or `cos(ks)` along a straight filament has a field expressible entirely in
//! terms of the free-space Green's function evaluated at the filament's two ends — the classical
//! result that makes sinusoidal bases attractive. Only the constant-current field needs a
//! numerical integral, done by [`crate::intx::intx`].
//!
//! Coordinates are local to the source segment: `z` is the observation point's position along the
//! segment axis (measured from its centre) and `rh` its radial distance. In the thin-wire
//! approximation the source current sits on the axis and the field is evaluated at radius `rh`,
//! which `EFLD` sets to include the *observation* wire's radius — so the self term is evaluated
//! on the wire surface, not on the singular axis.
//!
//! Each component comes back as an axial (`ez*`) and radial (`er*`) part; `EFLD` turns them into
//! Cartesian vectors.
//!
//! With `EK`, NEC uses the extended thin-wire kernel instead ([`ekscx`], `EKSCX`, `GXX`): the
//! source current is spread over the wire surface rather than concentrated on the axis, which
//! stays accurate for fatter segments (length/radius down to about 0.5 rather than 2).

use crate::consts::CON;
use crate::intx::{Tmi, intx};
use crate::scaled::WireSeg;
use nec_geom::{Connectivity, WireEnd};
use num_complex::Complex64;

/// The axial and radial field components for the three current shapes on one source segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eksc {
    /// Axial field of a `sin` current.
    pub ezs: Complex64,
    /// Radial field of a `sin` current.
    pub ers: Complex64,
    /// Axial field of a `cos` current.
    pub ezc: Complex64,
    /// Radial field of a `cos` current.
    pub erc: Complex64,
    /// Axial field of a constant current.
    pub ezk: Complex64,
    /// Radial field of a constant current.
    pub erk: Complex64,
}

/// `GX`: the Green's function `exp(-jkR)/R` and its radial derivative factor at axial offset `zz`.
///
/// Returns `(gz, gzp)` with `gz = exp(-jkR)/R` and `gzp = -(1 + jkR)·gz/R²`.
#[must_use]
pub fn gx(zz: f64, rh: f64, xk: f64) -> (Complex64, Complex64) {
    let r2 = zz * zz + rh * rh;
    let r = r2.sqrt();
    let rk = xk * r;
    let gz = Complex64::new(rk.cos(), -rk.sin()) / r;
    let gzp = -Complex64::new(1.0, rk) * gz / r2;
    (gz, gzp)
}

/// `EKSC`: fields of the three current shapes on a segment of length `s`, at an observation point
/// `z` along and `rh` away from its axis. `xk` is the wavenumber (`TP`, since lengths are in
/// wavelengths); `self_term` is NEC's `IJ == 0`.
#[must_use]
pub fn eksc(s: f64, z: f64, rh: f64, xk: f64, self_term: bool) -> Eksc {
    let con = Complex64::new(0.0, CON);
    let tmi = Tmi {
        zpk: xk * z,
        rkb2: (xk * rh) * (xk * rh),
        self_term,
    };
    let rhk = xk * rh;
    let sh = 0.5 * s;
    let shk = xk * sh;
    let (ss, cs) = shk.sin_cos();

    // Offsets from the observation point to segment end 1 and end 2.
    let z2 = sh - z;
    let z1 = -(sh + z);
    let (gz1, gp1) = gx(z1, rh, xk);
    let (gz2, gp2) = gx(z2, rh, xk);
    let mut gzp1 = gp1 * z1;
    let mut gzp2 = gp2 * z2;

    let ezs = con * ((gz2 - gz1) * cs * xk - (gzp2 + gzp1) * ss);
    let ezc = -con * ((gz2 + gz1) * ss * xk + (gzp2 - gzp1) * cs);
    let erk = con * (gp2 - gp1) * rh;

    let (cint, sint) = intx(-shk, shk, rhk, &tmi);
    let ezk = -con * (gzp2 - gzp1 + xk * xk * Complex64::new(cint, -sint));

    gzp1 *= z1;
    gzp2 *= z2;
    let (ers, erc) = if rh < 1e-10 {
        (Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0))
    } else {
        (
            -con * ((gzp2 + gzp1 + gz2 + gz1) * ss - (gz2 * z2 - gz1 * z1) * cs * xk) / rh,
            -con * ((gzp2 - gzp1 + gz2 - gz1) * cs + (gz2 * z2 + gz1 * z1) * ss * xk) / rh,
        )
    };

    Eksc {
        ezs,
        ers,
        ezc,
        erc,
        ezk,
        erk,
    }
}

/// How the extended thin-wire kernel treats one end of a source segment (`IND1`, `IND2`).
///
/// The extended kernel models the current as spread over the wire surface rather than along its
/// axis. Its end terms (`GXX`) are valid only where the segment runs smoothly into another of the
/// same radius, or ends freely; at a bend, a multi-wire junction or a radius step NEC falls back to
/// the thin-wire end term (`GX`) at that end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedEnd {
    /// `IND = 0`: continues straight into a segment of the same radius (or is a vertical end bonded
    /// to ground).
    Continuing,
    /// `IND = 1`: a free end.
    Free,
    /// `IND = 2`: anything else — use the thin-wire end term.
    ThinWire,
}

/// `GXX`: the extended thin-wire kernel's contribution from one segment end, at axial offset `zz`
/// and radial distance `rh`, for a current tube of radius `a` (`a2 = a²`). `ira` is set when the
/// observation point lies inside the source tube, in which case `EKSCX` has swapped the two radii.
///
/// Returns `(G1, G1P, G2, G2P, G3, GZP)` as NEC names them.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn gxx(
    zz: f64,
    rh: f64,
    a: f64,
    a2: f64,
    xk: f64,
    ira: bool,
) -> (Complex64, Complex64, Complex64, Complex64, Complex64, Complex64) {
    let r2 = zz * zz + rh * rh;
    let r = r2.sqrt();
    let r4 = r2 * r2;
    let rk = xk * r;
    let rk2 = rk * rk;
    let rh2 = rh * rh;
    let t1 = 0.25 * a2 * rh2 / r4;
    let t2 = 0.5 * a2 / r2;
    let c1 = Complex64::new(1.0, rk);
    let c2 = 3.0 * c1 - rk2;
    let c3 = Complex64::new(6.0, rk) * rk2 - 15.0 * c1;
    let mut gz = Complex64::new(rk.cos(), -rk.sin()) / r;
    let mut g2 = gz * (1.0 + t1 * c2);
    let g1 = g2 - t2 * c1 * gz;
    gz /= r2;
    let mut g2p = gz * (t1 * c3 - c1);
    let mut gzp = t2 * c2 * gz;
    let mut g3 = g2p + gzp;
    let g1p = g3 * zz;
    if ira {
        let t2 = 0.5 * a;
        g2 = -(t2 * c1 * gz);
        g2p = t2 * gz * c2 / r2;
        g3 = rh2 * g2p - a * gz * c1;
        g2p *= zz;
        gzp = -(zz * c1 * gz);
        return (g1, g1p, g2, g2p, g3, gzp);
    }
    g3 = (g3 + gzp) * rh;
    gzp = -(zz * c1 * gz);
    if rh > 1e-10 {
        g2 /= rh;
        g2p = g2p * zz / rh;
    } else {
        g2 = Complex64::new(0.0, 0.0);
        g2p = Complex64::new(0.0, 0.0);
    }
    (g1, g1p, g2, g2p, g3, gzp)
}

/// `EKSCX`: [`eksc`] by the extended thin-wire approximation, for a source segment of radius
/// `bx`. `rhx` is the radial distance as `EFLD` computed it (with the observation wire's radius
/// folded in); `ends` the treatment of the source segment's two ends.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::similar_names, clippy::too_many_arguments)]
pub fn ekscx(
    bx: f64,
    s: f64,
    z: f64,
    rhx: f64,
    xk: f64,
    self_term: bool,
    ends: [ExtendedEnd; 2],
) -> Eksc {
    let con = Complex64::new(0.0, CON);
    // Inside the source tube the roles of the two radii swap.
    let (rh, b, ira) = if rhx < bx { (bx, rhx, true) } else { (rhx, bx, false) };
    let sh = 0.5 * s;
    let rhk = xk * rh;
    let tmi = Tmi {
        zpk: xk * z,
        rkb2: rhk * rhk,
        self_term,
    };
    let shk = xk * sh;
    let (ss, cs) = shk.sin_cos();
    let z2 = sh - z;
    let z1 = -(sh + z);
    let a2 = b * b;

    let end = |zz: f64, treatment: ExtendedEnd| {
        if treatment == ExtendedEnd::ThinWire {
            let (gz, grk) = gx(zz, rhx, xk);
            let gzp = grk * zz;
            (gz, gzp, gz / rhx, gzp / rhx, grk * rhx, Complex64::new(0.0, 0.0))
        } else {
            gxx(zz, rh, b, a2, xk, ira)
        }
    };
    let (gz1, gzp1, gr1, grp1, grk1, gzz1) = end(z1, ends[0]);
    let (gz2, gzp2, gr2, grp2, grk2, gzz2) = end(z2, ends[1]);

    let ezs = con * ((gz2 - gz1) * cs * xk - (gzp2 + gzp1) * ss);
    let ezc = -con * ((gz2 + gz1) * ss * xk + (gzp2 - gzp1) * cs);
    let ers = -con * ((z2 * grp2 + z1 * grp1 + gr2 + gr1) * ss - (z2 * gr2 - z1 * gr1) * cs * xk);
    let erc = -con * ((z2 * grp2 - z1 * grp1 + gr2 - gr1) * cs + (z2 * gr2 + z1 * gr1) * ss * xk);
    let erk = con * (grk2 - grk1);
    let (cint, sint) = intx(-shk, shk, rhk, &tmi);
    let bk = b * xk;
    let bk2 = bk * bk * 0.25;
    let ezk = -con
        * (gzp2 - gzp1 + xk * xk * (1.0 - bk2) * Complex64::new(cint, -sint) - bk2 * (gzz2 - gzz1));
    Eksc {
        ezs,
        ers,
        ezc,
        erc,
        ezk,
        erk,
    }
}

/// `CMWW`'s choice of end treatment for source segment `j` (0-based) under the extended kernel,
/// for the interaction matrix.
///
/// An end is [`ExtendedEnd::Continuing`] when it joins exactly one other segment end — the two
/// point at each other in NEC's connection codes — that is parallel to within 1e-6 and of the same
/// radius to within 1e-6, or when it is a vertical end bonded to ground. An end on a surface patch
/// (`ICON > 10000`) is treated unevenly, as `CMWW` codes it: continuing at end 1
/// (`GO TO 5`, `IND1 = 0`) but thin-wire at end 2 (`GO TO 15`, `IND2 = 2`).
#[must_use]
pub fn extended_ends(j: usize, segs: &[WireSeg], conn: &[Connectivity]) -> [ExtendedEnd; 2] {
    end_treatment(j, segs, conn, [ExtendedEnd::Continuing, ExtendedEnd::ThinWire])
}

/// The same choice as `NEFLD` (near fields) and `QDSRC` (`EX 5` sources) make it: as
/// [`extended_ends`], except that an end on a surface patch is thin-wire at both ends.
#[must_use]
pub fn extended_ends_off_matrix(j: usize, segs: &[WireSeg], conn: &[Connectivity]) -> [ExtendedEnd; 2] {
    end_treatment(j, segs, conn, [ExtendedEnd::ThinWire, ExtendedEnd::ThinWire])
}

fn end_treatment(j: usize, segs: &[WireSeg], conn: &[Connectivity], on_patch: [ExtendedEnd; 2]) -> [ExtendedEnd; 2] {
    #[allow(clippy::cast_possible_wrap)]
    let me = (j + 1) as i64;
    let code = |k: usize, end: WireEnd| match end {
        WireEnd::A => conn[k].end1.nec_code(k + 1, WireEnd::A),
        WireEnd::B => conn[k].end2.nec_code(k + 1, WireEnd::B),
    };
    let similar = |k: usize| {
        let (d, e) = (segs[j].dir, segs[k].dir);
        let xi = (d[0] * e[0] + d[1] * e[1] + d[2] * e[2]).abs();
        xi >= 0.999_999 && (segs[k].radius / segs[j].radius - 1.0).abs() <= 1e-6
    };
    let horizontal = segs[j].dir[0] * segs[j].dir[0] + segs[j].dir[1] * segs[j].dir[1] > 1e-8;
    let classify = |this: WireEnd, other_same: WireEnd, other_opposite: WireEnd| {
        let ipr = code(j, this);
        let on_patch = on_patch[usize::from(this == WireEnd::B)];
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let k = (ipr.unsigned_abs() as usize).wrapping_sub(1);
        if ipr == 0 {
            ExtendedEnd::Free
        } else if ipr > 10_000 {
            on_patch
        } else if ipr == me {
            // Bonded to ground: fine for a vertical segment, not for a slanted one.
            if horizontal { ExtendedEnd::ThinWire } else { ExtendedEnd::Continuing }
        } else if (ipr < 0 && code(k, other_same) != -me) || (ipr > 0 && code(k, other_opposite) != me) {
            // The neighbour does not point back: a junction of three or more.
            ExtendedEnd::ThinWire
        } else if similar(k) {
            ExtendedEnd::Continuing
        } else {
            ExtendedEnd::ThinWire
        }
    };
    [
        classify(WireEnd::A, WireEnd::A, WireEnd::B),
        classify(WireEnd::B, WireEnd::B, WireEnd::A),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TP;

    fn ends(src: &str) -> Vec<[ExtendedEnd; 2]> {
        let mut d = nec_cards::parse_deck(src).expect("parses");
        d.resolve().expect("resolves");
        let g = nec_geom::Geometry::build(&d).expect("builds");
        let segs = crate::scale(&g, nec_core::Frequency::from_mhz(300.0).expect("valid"));
        (0..segs.len()).map(|j| extended_ends(j, &segs, &g.connectivity)).collect()
    }

    #[test]
    fn extended_kernel_end_treatments_follow_cmww() {
        use ExtendedEnd::{Continuing, Free, ThinWire};
        // A straight wire: free at its two ends, continuing everywhere else.
        let e = ends("GW 1 3 0 -0.25 0.6 0 0.25 0.6 0.01\nGE 0\nEN\n");
        assert_eq!(e, vec![[Free, Continuing], [Continuing, Continuing], [Continuing, Free]]);
        // A bend falls back to the thin-wire term on both sides of it.
        let e = ends("GW 1 2 0 -0.25 0.6 0 0 0.6 0.01\nGW 2 2 0 0 0.6 0 0.2 0.75 0.01\nGE 0\nEN\n");
        assert_eq!(e[1][1], ThinWire);
        assert_eq!(e[2][0], ThinWire);
        // So does a radius step on a straight line.
        let e = ends("GW 1 2 0 -0.25 0.6 0 0 0.6 0.01\nGW 2 2 0 0 0.6 0 0.25 0.6 0.005\nGE 0\nEN\n");
        assert_eq!((e[1][1], e[2][0]), (ThinWire, ThinWire));
        // A vertical end bonded to ground counts as continuing; a slanted one does not.
        let e = ends("GW 1 2 0 0 0 0 0 0.25 0.01\nGE 1\nEN\n");
        assert_eq!(e[0][0], Continuing);
        let e = ends("GW 1 2 0 0 0 0.1 0 0.24 0.01\nGE 1\nEN\n");
        assert_eq!(e[0][0], ThinWire);
    }

    #[test]
    fn gx_is_the_free_space_greens_function() {
        let (gz, gzp) = gx(0.3, 0.4, TP); // R = 0.5
        let expect = Complex64::new((TP * 0.5).cos(), -(TP * 0.5).sin()) / 0.5;
        assert!((gz - expect).norm() < 1e-14);
        let expect_p = -Complex64::new(1.0, TP * 0.5) * expect / 0.25;
        assert!((gzp - expect_p).norm() < 1e-13);
    }

    #[test]
    fn fields_are_symmetric_about_the_segment_centre() {
        // Mirroring the observation point through the centre (z -> -z) leaves the axial field of
        // an even current (cos, constant) unchanged and flips the odd one (sin).
        let a = eksc(0.05, 0.13, 0.02, TP, false);
        let b = eksc(0.05, -0.13, 0.02, TP, false);
        assert!((a.ezc - b.ezc).norm() < 1e-9 * a.ezc.norm());
        assert!((a.ezk - b.ezk).norm() < 1e-9 * a.ezk.norm());
        assert!((a.ezs + b.ezs).norm() < 1e-9 * a.ezs.norm());
    }

    #[test]
    fn radial_field_vanishes_on_axis() {
        let f = eksc(0.05, 0.2, 0.0, TP, false);
        assert_eq!(f.ers, Complex64::new(0.0, 0.0));
        assert_eq!(f.erc, Complex64::new(0.0, 0.0));
    }
}
