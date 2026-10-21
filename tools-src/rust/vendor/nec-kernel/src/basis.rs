//! NEC-2's current basis functions (`TBF`, `SBF`, `TRIO`).
//!
//! On every segment the current is expanded as
//!
//! ```text
//!     I(s) = A + B·sin(k·s) + C·cos(k·s),     |s| ≤ Δ/2, s measured from the segment centre
//! ```
//!
//! Basis function *i* is centred on segment *i* with `A = -1`, and extends "tails" onto every
//! segment meeting either end of it. The `B` and `C` coefficients, and the weights on the tails,
//! are chosen so that current and charge are continuous across each junction, with each wire's
//! share of the junction charge set by the Wu–King factor `1 / (ln(1/(π a)) − γ)` (radius `a` in
//! wavelengths). Free ends get an end-cap condition.
//!
//! # Correspondence with the Fortran
//!
//! `TBF(I, ICAP)` computes all pieces of basis function `I`; this is [`basis_function`].
//! `TRIO(J)` gathers, for one segment `J`, the piece of every basis function that touches it,
//! using `SBF` — which always applies the end cap. [`pieces_by_segment`] produces the same thing
//! for every segment at once by inverting the `TBF` pieces, which is equivalent and avoids
//! recomputing each basis function once per segment it touches.
//!
//! `TBF` walks each junction as a ring of `ICON` links, flipping two sign flags (`SIG`, `JEND`)
//! at each hop. Those flips always move together, which reduces to an absolute rule used here:
//! a tail's sign is `-e_i · e_n`, where `e_i` is −1 or +1 for whether the junction is at the
//! basis segment's end 1 or end 2, and `e_n` the same for the neighbour. A ground-bonded end is
//! a junction with the segment's own image: a single member with sign +1 and `B` negated. So is an
//! end on a surface patch, where the current flows on into the surface.

use crate::consts::{EULER_GAMMA, PI};
use crate::scaled::WireSeg;
use nec_geom::{Connectivity, Junction, WireEnd};

/// One basis function's coefficients on one segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    /// 0-based segment index the coefficients apply to.
    pub segment: usize,
    /// Constant coefficient `A`.
    pub a: f64,
    /// Sine coefficient `B`.
    pub b: f64,
    /// Cosine coefficient `C`.
    pub c: f64,
}

/// A contribution of basis function `basis` to some segment, as gathered by [`pieces_by_segment`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contribution {
    /// 0-based basis function (equivalently, its centre segment).
    pub basis: usize,
    /// Constant coefficient `A`.
    pub a: f64,
    /// Sine coefficient `B`.
    pub b: f64,
    /// Cosine coefficient `C`.
    pub c: f64,
}

/// Per-segment quantities every junction calculation needs. `D = π·Δ` is half the segment's
/// electrical length in radians (`k·Δ/2` with `k = 2π`).
struct SegTerms {
    sdh: f64,
    cdh: f64,
    sd: f64,
    omc: f64,
    aj: f64,
}

fn seg_terms(s: &WireSeg) -> SegTerms {
    let d = PI * s.length;
    let (sdh, cdh) = d.sin_cos();
    let sd = 2.0 * sdh * cdh;
    // 1 - cos(2D), by series for short segments where the subtraction would cancel. The
    // coefficients are 1/720 and 1/24, as TBF writes them.
    let omc = if d > 0.015 {
        1.0 - cdh * cdh + sdh * sdh
    } else {
        let x = 4.0 * d * d;
        ((1.388_888_9e-3 * x - 4.166_666_666_7e-2) * x + 0.5) * x
    };
    SegTerms {
        sdh,
        cdh,
        sd,
        omc,
        aj: wu_king(s.radius),
    }
}

/// The charge-share factor `1 / (ln(1/(π a)) − γ)`, radius in wavelengths.
fn wu_king(radius: f64) -> f64 {
    1.0 / ((1.0 / (PI * radius)).ln() - EULER_GAMMA)
}

/// End-cap factor `ξ`, used at a free end when the end cap is enabled.
fn end_cap(radius: f64) -> f64 {
    let q = PI * radius;
    let q2 = q * q;
    q * (1.0 - 0.5 * q2) / (1.0 - q2)
}

/// One member of the junction at one end of a basis segment.
struct Member {
    segment: usize,
    sig: f64,
    ground: bool,
}

/// The members of the junction at `end` of segment `i` (0-based).
fn members(i: usize, end: WireEnd, conn: &Connectivity) -> Vec<Member> {
    let (junction, e_i) = match end {
        WireEnd::A => (&conn.end1, -1.0),
        WireEnd::B => (&conn.end2, 1.0),
    };
    match junction {
        Junction::Free => Vec::new(),
        // A wire ending on a patch is treated like a ground bond: TBF and SBF both replace
        // `ICON > 10000` with the segment itself.
        Junction::Ground | Junction::Patch(_) => vec![Member {
            segment: i,
            sig: 1.0,
            ground: true,
        }],
        Junction::Joined(ends) => ends
            .iter()
            .map(|r| {
                let e_n = match r.end {
                    WireEnd::A => -1.0,
                    WireEnd::B => 1.0,
                };
                Member {
                    segment: r.segment - 1,
                    sig: -e_i * e_n,
                    ground: false,
                }
            })
            .collect(),
    }
}

/// Basis function `i` (0-based): every segment it touches, with its coefficients there.
///
/// The tails come first (end 1 then end 2), followed by the centre piece on segment `i` — the
/// same order `TBF` leaves in `AX/BX/CX/JCO`. `cap` corresponds to `TBF`'s `ICAP`; the matrix fill
/// always uses the end cap, because `SBF` applies it unconditionally.
#[must_use]
pub fn basis_function(i: usize, segs: &[WireSeg], conn: &[Connectivity], cap: bool) -> Vec<Piece> {
    basis_function_with(i, segs, &conn[i], cap)
}

/// [`basis_function`] with segment `i`'s own connectivity given explicitly.
///
/// `QDSRC` and `CABC` call `TBF` with `ICON1(I)` temporarily zeroed, freeing end 1 of the
/// source segment, to build the current a charge-discontinuity source (`EX 5`) injects.
#[must_use]
pub fn basis_function_with(
    i: usize,
    segs: &[WireSeg],
    own: &Connectivity,
    cap: bool,
) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();

    let tails = |end: WireEnd, pieces: &mut Vec<Piece>| -> f64 {
        let mut p = 0.0;
        for m in members(i, end, own) {
            let t = seg_terms(&segs[m.segment]);
            p -= t.omc / t.sd * t.aj;
            let b = t.aj / (2.0 * t.cdh);
            pieces.push(Piece {
                segment: m.segment,
                a: t.aj / t.sd * m.sig,
                b: if m.ground { -b } else { b },
                c: -t.aj / (2.0 * t.sdh) * m.sig,
            });
        }
        p
    };

    let pm = -tails(WireEnd::A, &mut pieces);
    let n1 = pieces.len();
    let pp = tails(WireEnd::B, &mut pieces);
    let n2 = pieces.len() - n1;

    // The centre segment.
    let t = seg_terms(&segs[i]);
    let (sdh, cdh, sd, omc) = (t.sdh, t.cdh, t.sd, t.omc);
    let cd = cdh * cdh - sdh * sdh;
    let (aj, ap) = (t.aj, t.aj);
    let xxi = if cap { end_cap(segs[i].radius) } else { 0.0 };

    let (b_center, c_center) = match (n1 > 0, n2 > 0) {
        // Both ends joined: solve for the weights on each junction.
        (true, true) => {
            let den = sd * (pm * pp + aj * ap) + cd * (pm * ap - pp * aj);
            let qm = (ap * omc - pp * sd) / den;
            let qp = -(aj * omc + pm * sd) / den;
            for pc in &mut pieces[..n1] {
                pc.a *= qm;
                pc.b *= qm;
                pc.c *= qm;
            }
            for pc in &mut pieces[n1..] {
                pc.a = -pc.a * qp;
                pc.b *= qp;
                pc.c = -pc.c * qp;
            }
            (
                (aj * qm + ap * qp) * sdh / sd,
                (aj * qm - ap * qp) * cdh / sd,
            )
        }
        // End 1 free, end 2 joined.
        (false, true) => {
            let qp = -(omc + xxi * sd) / (sd * (ap + xxi * pp) + cd * (xxi * ap - pp));
            let dd = cd - xxi * sd;
            for pc in &mut pieces[..n2] {
                pc.a = -pc.a * qp;
                pc.b *= qp;
                pc.c = -pc.c * qp;
            }
            (
                (sdh + ap * qp * (cdh - xxi * sdh)) / dd,
                (cdh + ap * qp * (sdh + xxi * cdh)) / dd,
            )
        }
        // End 1 joined, end 2 free.
        (true, false) => {
            let qm = (omc + xxi * sd) / (sd * (aj - xxi * pm) + cd * (pm + xxi * aj));
            let dd = cd - xxi * sd;
            for pc in &mut pieces[..n1] {
                pc.a *= qm;
                pc.b *= qm;
                pc.c *= qm;
            }
            (
                (aj * qm * (cdh - xxi * sdh) - sdh) / dd,
                (cdh - aj * qm * (sdh + xxi * cdh)) / dd,
            )
        }
        // An isolated single segment.
        (false, false) => (0.0, 1.0 / (cdh - xxi * sdh)),
    };

    pieces.push(Piece {
        segment: i,
        a: -1.0,
        b: b_center,
        c: c_center,
    });
    pieces
}

/// For every segment, the contribution of each basis function that touches it — `TRIO` for all
/// segments at once, reproducing `TRIO`/`SBF` exactly.
///
/// For segment `J`, `TRIO` walks the junction at each end of `J` and calls `SBF(I, J)` once for
/// every neighbour `I` it meets, then once more for `J`'s own basis function. Normally that is
/// the same as gathering the [`basis_function`] pieces that land on `J`. It differs when one basis
/// function has tails on segment `J` from *both* of its ends — which happens only when two
/// segments are joined at both ends, i.e. coincident (duplicated) wires:
///
/// - `SBF` *assigns* the tail coefficients rather than accumulating them, so it returns only the
///   tail from the last junction it walks (end 2), not the sum of both tails; and
/// - `TRIO` meets the neighbour once in each of `J`'s junctions and so lists it twice.
///
/// The fill therefore sees twice the end-2 tail where the basis function itself (`TBF`, which
/// `CABC` uses to reconstruct currents) has end-1 tail + end-2 tail. That is an inconsistency
/// inside NEC-2; it is reproduced here so results match the reference engine. Coincident wires
/// are invalid input in any case (the manual forbids overlapping segments).
///
/// A ground-bonded end is a tail on the segment itself; `SBF` adds the centre piece to it, so
/// for `I = J` the result is the last tail on `J` (if any) plus the centre.
#[must_use]
pub fn pieces_by_segment(segs: &[WireSeg], conn: &[Connectivity]) -> Vec<Vec<Contribution>> {
    let bases: Vec<Vec<Piece>> =
        (0..segs.len()).map(|i| basis_function(i, segs, conn, true)).collect();

    // SBF(I, J): the last tail of basis I on segment J, plus the centre piece when I = J.
    let sbf = |i: usize, j: usize| -> Contribution {
        let pieces = &bases[i];
        let (centre, tails) = pieces.split_last().expect("a basis function has a centre piece");
        let mut c = tails
            .iter()
            .rev()
            .find(|p| p.segment == j)
            .map_or(Contribution { basis: i, a: 0.0, b: 0.0, c: 0.0 }, |p| Contribution {
                basis: i,
                a: p.a,
                b: p.b,
                c: p.c,
            });
        if i == j {
            c.a += centre.a;
            c.b += centre.b;
            c.c += centre.c;
        }
        c
    };

    (0..segs.len())
        .map(|j| {
            let mut list: Vec<Contribution> = [WireEnd::A, WireEnd::B]
                .into_iter()
                .flat_map(|end| members(j, end, &conn[j]))
                .filter(|m| m.segment != j)
                .map(|m| sbf(m.segment, j))
                .collect();
            list.push(sbf(j, j));
            list
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaled::scale;
    use nec_cards::parse_deck;
    use nec_core::Frequency;
    use nec_geom::Geometry;

    fn setup(src: &str, mhz: f64) -> (Vec<WireSeg>, Vec<Connectivity>) {
        let mut d = parse_deck(src).expect("parses");
        d.resolve().expect("resolves");
        let g = Geometry::build(&d).expect("builds");
        let w = scale(&g, Frequency::from_mhz(mhz).expect("valid"));
        (w, g.connectivity)
    }

    /// Current at a point `s` along a segment from one piece.
    fn current(p: &Piece, s: f64) -> f64 {
        let k = crate::consts::TP;
        p.a + p.b * (k * s).sin() + p.c * (k * s).cos()
    }

    #[test]
    fn interior_basis_function_spans_its_neighbours() {
        let (w, c) = setup("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n", 300.0);
        let f = basis_function(4, &w, &c, true);
        // Tail on segment 4 (index 3), tail on segment 6 (index 5), centre on segment 5.
        assert_eq!(f.iter().map(|p| p.segment).collect::<Vec<_>>(), vec![3, 5, 4]);
        assert!((f[2].a + 1.0).abs() < 1e-15, "centre A is -1");
    }

    #[test]
    fn current_is_continuous_across_each_junction() {
        // The defining property of the basis: the centre piece and each tail agree on the
        // current where they meet.
        let (w, c) = setup("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n", 300.0);
        let f = basis_function(4, &w, &c, true);
        let (left, right, centre) = (f[0], f[1], f[2]);
        let h = |i: usize| w[i].length / 2.0;
        // Centre's end 1 meets the left neighbour's end 2.
        let at_left = current(&centre, -h(4));
        let tail_left = current(&left, h(3));
        assert!((at_left - tail_left).abs() < 1e-12, "{at_left} vs {tail_left}");
        // Centre's end 2 meets the right neighbour's end 1.
        let at_right = current(&centre, h(4));
        let tail_right = current(&right, -h(5));
        assert!((at_right - tail_right).abs() < 1e-12, "{at_right} vs {tail_right}");
    }

    #[test]
    fn tails_vanish_at_their_far_ends() {
        // Each tail carries current only between the junction and its own far end.
        let (w, c) = setup("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n", 300.0);
        let f = basis_function(4, &w, &c, true);
        let h = |i: usize| w[i].length / 2.0;
        assert!(current(&f[0], -h(3)).abs() < 1e-12, "left tail at its end 1");
        assert!(current(&f[1], h(5)).abs() < 1e-12, "right tail at its end 2");
    }

    #[test]
    fn every_segment_sees_itself_and_its_neighbours() {
        let (w, c) = setup("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n", 300.0);
        let by_seg = pieces_by_segment(&w, &c);
        // An interior segment carries its own basis plus both neighbours' tails.
        let mut b: Vec<usize> = by_seg[4].iter().map(|x| x.basis).collect();
        b.sort_unstable();
        assert_eq!(b, vec![3, 4, 5]);
        // An end segment carries only its own basis and one neighbour's tail.
        let mut b0: Vec<usize> = by_seg[0].iter().map(|x| x.basis).collect();
        b0.sort_unstable();
        assert_eq!(b0, vec![0, 1]);
    }

    #[test]
    fn coincident_wires_reproduce_sbfs_overwrite() {
        // Two copies of a 3-segment wire: segment k (0-based) of the first coincides with
        // segment k+3 of the second, so their interiors are joined at both ends. NEC-2's own
        // TRIO, compiled from the published source and run on a model with such a pair, lists
        // the duplicate basis twice on the segment, each time with only its end-2 tail.
        let (w, c) = setup(
            "GW 1 3 0 -.1 0 0 .1 0 1e-3\nGW 2 3 0 -.1 0 0 .1 0 1e-3\nGE 0\nEN\n",
            300.0,
        );
        let by_seg = pieces_by_segment(&w, &c);
        let on_1: Vec<&Contribution> = by_seg[1].iter().filter(|x| x.basis == 4).collect();
        assert_eq!(on_1.len(), 2, "basis 5 meets segment 2 at both of segment 2's ends");
        assert_eq!(on_1[0], on_1[1]);

        // And each copy is the end-2 tail of the basis function, not the sum of both tails.
        let f = basis_function(4, &w, &c, true);
        let tails: Vec<&Piece> = f[..f.len() - 1].iter().filter(|p| p.segment == 1).collect();
        assert_eq!(tails.len(), 2, "basis 5 has a tail on segment 2 from each end");
        let last = tails[1];
        assert_eq!((on_1[0].a, on_1[0].b, on_1[0].c), (last.a, last.b, last.c));
        // Here the tails are mirror images (their B terms have opposite signs), so their sum has
        // no sine term, while NEC-2's doubled end-2 tail does.
        assert!((tails[0].b + last.b).abs() < 1e-12 && last.b.abs() > 1e-6, "{tails:?}");
    }
}
