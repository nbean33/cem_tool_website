//! Which segment ends join which.
//!
//! Connectivity is not bookkeeping — it is what makes the current expansion continuous. The
//! three-term sinusoidal basis on each segment is constrained by current and charge continuity
//! at junctions, so getting this wrong produces a solution that looks plausible and is wrong.
//!
//! # The rule, and where it comes from
//!
//! The NEC-2 manual states the tolerance only loosely:
//!
//! > "Segments will be treated as connected if the separation of their ends is less than
//! > about 10-3 times the length of the shortest segment."
//!
//! The exact procedure below follows NEC-2's `CONECT` routine, and is confirmed against the
//! reference engine's printed connection data by `cargo xtask geometry`:
//!
//! 1. The tolerance for an end of segment *i* is `1e-3 × length(i)` — the segment being
//!    examined, not the pair's minimum.
//! 2. Distance is the **L1** norm, `|dx| + |dy| + |dz|`.
//! 3. Under `GE 1`, ground is tested **first**: an end with `z ≤ tolerance` is bonded to ground
//!    even if another wire also ends there.
//! 4. Otherwise candidates are searched in wrap-around order starting at segment *i + 1*
//!    (`i+1, …, N, 1, …, i−1`), checking each candidate's end 1 before its end 2. NEC records the
//!    **first** match; this module records all of them, in that order, so multi-wire junctions
//!    are known while the first entry still reproduces NEC's printed value.
//!
//! An earlier version used an exact-coincidence tolerance (`1e-9 ×` the model's diagonal). It
//! missed every closed quad loop whose corner coordinates came from different `SY` expressions
//! that agree mathematically but not to the last bit, and reported a different segment than NEC
//! at junctions where looser matching finds more than one candidate.

use crate::geometry::GroundFlag;
use crate::segment::Segment;
use nec_core::Point3;
use std::collections::HashMap;

/// Fraction of a segment's length within which another end counts as coincident (NEC's `SMIN`).
pub const CONNECT_FRACTION: f64 = 1.0e-3;

/// Which end of a segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireEnd {
    /// End 1 (`a`).
    A,
    /// End 2 (`b`).
    B,
}

/// A reference to one end of one segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndRef {
    /// 1-based segment number.
    pub segment: usize,
    /// Which end of that segment.
    pub end: WireEnd,
}

/// What a segment end is attached to.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Junction {
    /// Nothing — an open wire end.
    #[default]
    Free,
    /// Bonded to the ground plane (`GE 1` and the end within tolerance of Z = 0).
    Ground,
    /// Joined to one or more other segment ends, in NEC's search order. More than one means a
    /// multi-wire junction, where Kirchhoff's current law couples three or more segments.
    Joined(Vec<EndRef>),
    /// Ending at the centre of a surface patch, which was split into four for it: the value is
    /// the 1-based number of the first of the four. The wire's current spreads into the surface,
    /// so for the basis functions this end behaves like a ground bond (NEC treats both as a
    /// junction of the segment with itself).
    Patch(usize),
}

impl Junction {
    /// The connected segment number when exactly one segment is joined here.
    #[must_use]
    pub fn single(&self) -> Option<usize> {
        match self {
            Self::Joined(v) if v.len() == 1 => Some(v[0].segment),
            _ => None,
        }
    }

    /// The first joined end in NEC's search order — the one NEC itself records.
    #[must_use]
    pub fn primary(&self) -> Option<EndRef> {
        match self {
            Self::Joined(v) => v.first().copied(),
            _ => None,
        }
    }

    /// Whether this is a junction of three or more wires.
    #[must_use]
    pub fn is_multi(&self) -> bool {
        matches!(self, Self::Joined(v) if v.len() > 1)
    }

    /// The value NEC prints in its `I-` / `I+` column for this end of segment `own`.
    ///
    /// * `0` — free end;
    /// * `own` — bonded to ground (NEC encodes a ground connection as the segment itself);
    /// * `±n` — joined to segment `n`, positive when the current continues in the same sense
    ///   (end 1 meets the other's end 2, or end 2 meets the other's end 1) and negative when the
    ///   segments meet head-to-head or tail-to-tail;
    /// * `10000 + p` — connected to patch `p`.
    #[must_use]
    pub fn nec_code(&self, own: usize, this_end: WireEnd) -> i64 {
        #[allow(clippy::cast_possible_wrap)]
        match self {
            Self::Free => 0,
            Self::Ground => own as i64,
            Self::Patch(p) => 10_000 + *p as i64,
            Self::Joined(v) => match v.first() {
                None => 0,
                Some(r) if r.end == this_end => -(r.segment as i64),
                Some(r) => r.segment as i64,
            },
        }
    }
}

/// Connectivity of one segment's two ends.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Connectivity {
    /// What end 1 attaches to. Corresponds to NEC's `I-` column.
    pub end1: Junction,
    /// What end 2 attaches to. Corresponds to NEC's `I+` column.
    pub end2: Junction,
}

fn l1(p: Point3, q: Point3) -> f64 {
    (p.x - q.x).abs() + (p.y - q.y).abs() + (p.z - q.z).abs()
}

/// Build per-segment endpoint connectivity using NEC's rule (see the module docs).
///
/// Endpoints are bucketed on a grid at least as coarse as the largest per-segment tolerance and
/// compared against the 27 neighbouring cells, then sorted into NEC's search order. That gives
/// the same answer as NEC's linear scan at linear rather than quadratic cost, which matters at
/// the 8000-segment ceiling.
#[must_use]
pub fn build_connectivity(segs: &[Segment], ground: GroundFlag) -> Vec<Connectivity> {
    let n = segs.len();
    let mut out = vec![Connectivity::default(); n];
    if n == 0 {
        return out;
    }

    let max_tol = segs
        .iter()
        .map(|s| CONNECT_FRACTION * s.length())
        .fold(0.0_f64, f64::max);
    let cell = max_tol.max(f64::MIN_POSITIVE);
    let key = |p: Point3| -> (i64, i64, i64) {
        #[allow(clippy::cast_possible_truncation)]
        (
            (p.x / cell).floor() as i64,
            (p.y / cell).floor() as i64,
            (p.z / cell).floor() as i64,
        )
    };

    let mut buckets: HashMap<(i64, i64, i64), Vec<(Point3, EndRef)>> = HashMap::new();
    for s in segs {
        for (p, end) in [(s.a, WireEnd::A), (s.b, WireEnd::B)] {
            buckets.entry(key(p)).or_default().push((
                p,
                EndRef {
                    segment: s.index,
                    end,
                },
            ));
        }
    }

    // Position of candidate `ic` in NEC's wrap-around search from segment `i`, then end 1
    // before end 2. Both 1-based.
    let order = |i: usize, r: &EndRef| -> (usize, u8) {
        let pos = (r.segment + n - i - 1) % n;
        (pos, u8::from(r.end == WireEnd::B))
    };

    for s in segs {
        let tol = CONNECT_FRACTION * s.length();
        for (p, this_end) in [(s.a, WireEnd::A), (s.b, WireEnd::B)] {
            // Ground first, exactly as NEC does: an end on the plane is bonded to it even when
            // another wire also ends there.
            let junction = if ground == GroundFlag::GroundedEnds && p.z <= tol {
                Junction::Ground
            } else {
                let mut joined: Vec<EndRef> = Vec::new();
                let (kx, ky, kz) = key(p);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            if let Some(list) = buckets.get(&(kx + dx, ky + dy, kz + dz)) {
                                for (q, r) in list {
                                    // NEC never matches a segment against itself.
                                    if r.segment != s.index && l1(p, *q) <= tol {
                                        joined.push(*r);
                                    }
                                }
                            }
                        }
                    }
                }
                joined.sort_by_key(|r| order(s.index, r));
                joined.dedup();
                if joined.is_empty() {
                    Junction::Free
                } else {
                    Junction::Joined(joined)
                }
            };

            let slot = &mut out[s.index - 1];
            match this_end {
                WireEnd::A => slot.end1 = junction,
                WireEnd::B => slot.end2 = junction,
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Geometry;
    use nec_cards::parse_deck;

    fn build(src: &str) -> Geometry {
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        Geometry::build(&deck).expect("builds")
    }

    fn codes(g: &Geometry, seg: usize) -> (i64, i64) {
        let c = &g.connectivity[seg - 1];
        (
            c.end1.nec_code(seg, WireEnd::A),
            c.end2.nec_code(seg, WireEnd::B),
        )
    }

    #[test]
    fn example1_connection_data_matches_the_engine() {
        // The engine prints, for the nine-segment dipole:
        //   seg 1: I- = 0, I+ = 2        (free at the start)
        //   seg 5: I- = 4, I+ = 6
        //   seg 9: I- = 8, I+ = 0        (free at the end)
        let g = build("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n");
        assert_eq!(codes(&g, 1), (0, 2));
        assert_eq!(codes(&g, 5), (4, 6));
        assert_eq!(codes(&g, 9), (8, 0));
    }

    #[test]
    fn separate_wires_join_where_their_ends_meet() {
        // Two wires meeting at (1,0,0): an L, both running "forward".
        let g = build(
            "GW 1 2 0 0 0 1 0 0 1e-3\n\
             GW 2 2 1 0 0 1 1 0 1e-3\n\
             GE 0\nEN\n",
        );
        assert_eq!(codes(&g, 2).1, 3, "end 2 meets the next wire's end 1: positive");
        assert_eq!(codes(&g, 3).0, 2);
    }

    #[test]
    fn head_to_head_meeting_is_negative() {
        // Two wires whose end 1s coincide at the origin, as in the engine's `-199` rows.
        let g = build(
            "GW 1 1 0 0 0 1 0 0 1e-3\n\
             GW 2 1 0 0 0 -1 0 0 1e-3\n\
             GE 0\nEN\n",
        );
        assert_eq!(codes(&g, 1).0, -2);
        assert_eq!(codes(&g, 2).0, -1);
    }

    #[test]
    fn near_miss_corners_still_connect() {
        // A closed loop whose closing corner is off by 1e-6 m on 0.5 m segments — the shape of
        // the Equations\*Quad.nec failures, where two SY expressions agree only approximately.
        let g = build(
            "GW 1 2 0 0 0 1 0 0 1e-3\n\
             GW 2 2 1 0 0 1 1 0 1e-3\n\
             GW 3 2 1 1 0 0 1 0 1e-3\n\
             GW 4 2 0 1 0 0.000001 0 0 1e-3\n\
             GE 0\nEN\n",
        );
        assert_eq!(g.connectivity[7].end2.single(), Some(1), "loop must close");
        assert_eq!(g.connectivity[0].end1.single(), Some(8));
    }

    #[test]
    fn nec_reports_the_first_match_in_wraparound_order() {
        // Three wires meet at the origin. Searching from segment 3 wraps to 1 before 2.
        let g = build(
            "GW 1 1 0 0 0 1 0 0 1e-3\n\
             GW 2 1 0 0 0 0 1 0 1e-3\n\
             GW 3 1 0 0 0 0 0 1 1e-3\n\
             GE 0\nEN\n",
        );
        assert!(g.connectivity[2].end1.is_multi());
        assert_eq!(g.connectivity[2].end1.primary().map(|r| r.segment), Some(1));
        // And from segment 1 the search reaches 2 first.
        assert_eq!(g.connectivity[0].end1.primary().map(|r| r.segment), Some(2));
    }

    #[test]
    fn closed_loop_has_no_free_ends() {
        let g = build(
            "GW 1 1 0 0 0 1 0 0 1e-3\n\
             GW 2 1 1 0 0 1 1 0 1e-3\n\
             GW 3 1 1 1 0 0 1 0 1e-3\n\
             GW 4 1 0 1 0 0 0 0 1e-3\n\
             GE 0\nEN\n",
        );
        for (i, c) in g.connectivity.iter().enumerate() {
            assert_ne!(c.end1, Junction::Free, "segment {} end 1", i + 1);
            assert_ne!(c.end2, Junction::Free, "segment {} end 2", i + 1);
        }
    }

    #[test]
    fn ground_wins_over_a_coincident_wire_and_prints_as_self() {
        // Two vertical wires both standing on the same ground point. Under GE 1 NEC bonds each
        // to ground rather than to each other, and prints the segment's own number.
        let g = build(
            "GW 1 1 0 0 0 0 0 1 1e-3\n\
             GW 2 1 0 0 0 1 0 1 1e-3\n\
             GE 1\nEN\n",
        );
        assert_eq!(g.connectivity[0].end1, Junction::Ground);
        assert_eq!(codes(&g, 1).0, 1);
        assert_eq!(codes(&g, 2).0, 2);

        // Without a bonded ground the same ends join each other.
        let g = build(
            "GW 1 1 0 0 0 0 0 1 1e-3\n\
             GW 2 1 0 0 0 1 0 1 1e-3\n\
             GE 0\nEN\n",
        );
        assert_eq!(codes(&g, 1).0, -2);
    }

    #[test]
    fn grounded_ends_bond_to_the_plane_only_with_ge_1() {
        let vertical = "GW 1 4 0 0 0 0 0 5 1e-3\n";
        let grounded = build(&format!("{vertical}GE 1\nEN\n"));
        assert_eq!(grounded.connectivity[0].end1, Junction::Ground);
        assert_eq!(grounded.connectivity[3].end2, Junction::Free);

        let floating = build(&format!("{vertical}GE -1\nEN\n"));
        assert_eq!(floating.connectivity[0].end1, Junction::Free);

        let free_space = build(&format!("{vertical}GE 0\nEN\n"));
        assert_eq!(free_space.connectivity[0].end1, Junction::Free);
    }
}
