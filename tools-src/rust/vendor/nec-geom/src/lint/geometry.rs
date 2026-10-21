//! Checks between pairs of segments: wires that cross without connecting, wire volumes that
//! intersect, wires laid over each other, junctions where one segment sits largely inside
//! another, close parallel wires segmented differently, and wires too low over the ground.
//!
//! Pairs are found through a uniform spatial grid, so the checks stay close to linear in the
//! number of segments up to the 8000-segment ceiling.

use super::{AUXILIARY_TAG, Finding, LintContext, LintGround, Severity};
use crate::Segment;
use crate::connect::Junction;
use crate::geometry::Geometry;
use nec_core::Frequency;
use nec_core::geom::Point3;
use std::collections::HashMap;

/// Two segments closer than this fraction of the shorter one's length, away from any joint,
/// cross or touch without being connected: 4nec2's geometry check ("the distance between two or
/// more segments is below seg-length/1000 and the connection point is not a wire-end or a
/// segment junction"), and NEC's own connection tolerance.
pub const CROSSING_FRACTION: f64 = 1.0e-3;

/// At a junction, how much of a segment's length may lie inside the other's wire volume: "check
/// if a segments volume does not lay more than about 1/3 in another segments volume" (4nec2 help,
/// Geometry form).
pub const JUNCTION_OVERLAP_FRACTION: f64 = 1.0 / 3.0;

/// Parallel wires closer than this (wavelengths) and overlapping by more than
/// [`PARALLEL_OVERLAP_WAVELENGTHS`] should be segmented alike: 4nec2's check ("parallel wires
/// within 1/20 wavelength distance and having an overlap length of more than 1/10 wavelength").
pub const PARALLEL_DISTANCE_WAVELENGTHS: f64 = 1.0 / 20.0;
/// See [`PARALLEL_DISTANCE_WAVELENGTHS`].
pub const PARALLEL_OVERLAP_WAVELENGTHS: f64 = 1.0 / 10.0;

/// Wires within this angle (degrees) count as parallel for the segmentation check (4nec2's
/// geometry check, `FUN_0048e0b0`: `angle < 2.0`).
pub const PARALLEL_ANGLE_DEGREES: f64 = 2.0;

/// The reflection-coefficient ground "can yield reasonable accuracy if the structure is a[t]
/// least several tenths of a wavelength above the ground" (NEC-2 manual, part III). 4nec2's
/// geometry check (`FUN_0048e0b0`) warns below 0.2 λ, or 0.1 λ for a wire within 10° of
/// horizontal ("Warn.: Height for Wire … below 0.2" / "0.1").
pub const REFLECTION_MIN_HEIGHT_WAVELENGTHS: f64 = 0.2;
/// See [`REFLECTION_MIN_HEIGHT_WAVELENGTHS`]: the height for a nearly horizontal wire.
pub const REFLECTION_MIN_HEIGHT_HORIZONTAL: f64 = 0.1;
/// Wires within this elevation (degrees) count as horizontal for the height check.
pub const HORIZONTAL_DEGREES: f64 = 10.0;
/// Over a Sommerfeld or perfect ground a wire end lower than this many radii (and not bonded to
/// the ground) is too close to its image for the thin-wire approximation (4nec2: "below 3 *
/// radius").
pub const SOMMERFELD_MIN_HEIGHT_RADII: f64 = 3.0;

/// Closest approach of segments `p0→p1` and `q0→q1`: the distance and the parameters
/// `(s, t)` in `[0, 1]` along each where it happens.
fn closest(p0: Point3, p1: Point3, q0: Point3, q1: Point3) -> (f64, f64, f64) {
    let d1 = p0.to(p1);
    let d2 = q0.to(q1);
    let r = q0.to(p0);
    let (a, e, f) = (d1.dot(d1), d2.dot(d2), d2.dot(r));
    let (mut s, mut t);
    if a <= 1e-300 && e <= 1e-300 {
        return (p0.distance(q0), 0.0, 0.0);
    }
    if a <= 1e-300 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(r);
        if e <= 1e-300 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = d1.dot(d2);
            let denom = a * e - b * b;
            s = if denom > 1e-300 * a * e { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
            t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            }
        }
    }
    let at = |o: Point3, d: Point3, k: f64| Point3::new(o.x + d.x * k, o.y + d.y * k, o.z + d.z * k);
    let pc = at(p0, d1, s);
    let qc = at(q0, d2, t);
    (pc.distance(qc), s, t)
}

/// The 1-based segments joined to segment `i`'s ends (including itself).
fn neighbours(geom: &Geometry, i: usize) -> Vec<usize> {
    let mut v = vec![i + 1];
    let c = &geom.connectivity[i];
    for j in [&c.end1, &c.end2] {
        if let Junction::Joined(ends) = j {
            v.extend(ends.iter().map(|e| e.segment));
        }
    }
    v
}

/// Grid cells (integer triples) a segment's bounding box, grown by `pad`, touches.
fn cells(s: &Segment, size: f64, pad: f64) -> Vec<(i64, i64, i64)> {
    #[allow(clippy::cast_possible_truncation)]
    let c = |v: f64| (v / size).floor() as i64;
    let lo = [s.a.x.min(s.b.x) - pad, s.a.y.min(s.b.y) - pad, s.a.z.min(s.b.z) - pad];
    let hi = [s.a.x.max(s.b.x) + pad, s.a.y.max(s.b.y) + pad, s.a.z.max(s.b.z) + pad];
    let mut out = Vec::new();
    for x in c(lo[0])..=c(hi[0]) {
        for y in c(lo[1])..=c(hi[1]) {
            for z in c(lo[2])..=c(hi[2]) {
                out.push((x, y, z));
            }
        }
    }
    out
}

/// Candidate pairs `(i, j)`, `i < j` (0-based), whose boxes grown by `pad` share a cell.
fn pairs(geom: &Geometry, pad: f64, shown: &dyn Fn(&Segment) -> bool) -> Vec<(usize, usize)> {
    let segs = &geom.segments;
    let longest = segs.iter().map(Segment::length).fold(0.0, f64::max);
    let size = (longest + 2.0 * pad).max(1e-9);
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    for (i, s) in segs.iter().enumerate().filter(|(_, s)| shown(s)) {
        for c in cells(s, size, pad) {
            grid.entry(c).or_default().push(i);
        }
    }
    let mut out: Vec<(usize, usize)> = Vec::new();
    for v in grid.values() {
        for (k, &i) in v.iter().enumerate() {
            for &j in &v[k + 1..] {
                out.push(if i < j { (i, j) } else { (j, i) });
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// The pairwise checks.
pub(super) fn check(geom: &Geometry, ctx: &LintContext, shown: &dyn Fn(&Segment) -> bool) -> Vec<Finding> {
    let mut out = Vec::new();
    let segs = &geom.segments;
    if segs.is_empty() {
        return out;
    }
    let lambda = ctx.frequency.map(Frequency::wavelength_m);
    let max_r = segs.iter().map(|s| s.radius).fold(0.0, f64::max);
    let pad = lambda.map_or(2.0 * max_r, |l| (l * PARALLEL_DISTANCE_WAVELENGTHS).max(2.0 * max_r));
    let mut parallel_pairs: Vec<(i32, i32, usize, usize)> = Vec::new();

    for (i, j) in pairs(geom, pad, shown) {
        let (a, b) = (&segs[i], &segs[j]);
        let joined = neighbours(geom, i).contains(&(j + 1));
        let (d, s, t) = closest(a.a, a.b, b.a, b.b);
        let short = a.length().min(b.length());
        let at_end = |u: f64| u <= 1e-6 || u >= 1.0 - 1e-6;

        if !joined
            && let Some(l) = lambda
            && a.tag != b.tag
            && let (Some(u), Some(v)) = (a.a.to(a.b).normalized(), b.a.to(b.b).normalized())
        {
            // Close parallel wires segmented differently.
            let angle = u.dot(v).abs().min(1.0).acos().to_degrees();
            if angle < PARALLEL_ANGLE_DEGREES && d < l * PARALLEL_DISTANCE_WAVELENGTHS {
                parallel_pairs.push((a.tag, b.tag, i, j));
            }
        }
        if !joined {
            // Wires that cross or touch away from a joint.
            if d < CROSSING_FRACTION * short && !(at_end(s) && at_end(t)) {
                out.push(Finding {
                    severity: Severity::Error,
                    kind: "crossing-wires",
                    segment: Some(a.index),
                    message: format!(
                        "segment {} and segment {} cross or touch away from their ends, without \
                         being connected; NEC needs wires to meet at segment ends",
                        a.index, b.index
                    ),
                });
                continue;
            }
            // Wire volumes that intersect.
            if d < a.radius + b.radius && d >= CROSSING_FRACTION * short {
                let dir = a.a.to(a.b).normalized().zip(b.a.to(b.b).normalized());
                let parallel = dir.is_some_and(|(u, v)| u.dot(v).abs() > 0.999_999);
                let overlap_len = if parallel {
                    // Overlap of the two segments' projections on a's axis.
                    let u = a.a.to(a.b).normalized().unwrap_or_default();
                    let (p0, p1) = (0.0_f64, a.length());
                    let (q0, q1) = (a.a.to(b.a).dot(u), a.a.to(b.b).dot(u));
                    (p1.min(q0.max(q1)) - p0.max(q0.min(q1))).max(0.0)
                } else {
                    0.0
                };
                if parallel && overlap_len > CROSSING_FRACTION * short {
                    out.push(Finding {
                        severity: Severity::Error,
                        kind: "overlaid-wires",
                        segment: Some(a.index),
                        message: format!(
                            "segment {} and segment {} lie along each other for {overlap_len:.4} m; \
                             NEC cannot divide current between overlapping wires",
                            a.index, b.index
                        ),
                    });
                } else {
                    out.push(Finding {
                        severity: Severity::Warning,
                        kind: "intersecting-wires",
                        segment: Some(a.index),
                        message: format!(
                            "segment {} and segment {} come within {d:.4} m, inside the sum of \
                             their radii; the wire volumes intersect",
                            a.index, b.index
                        ),
                    });
                }
                continue;
            }
        } else if i != j {
            // At a junction: how much of each segment lies inside the other's volume.
            let (Some(u), Some(v)) = (a.a.to(a.b).normalized(), b.a.to(b.b).normalized()) else { continue };
            // Direction away from the shared point along each segment.
            let shared = [a.a, a.b]
                .into_iter()
                .flat_map(|p| [b.a, b.b].into_iter().map(move |q| (p, q)))
                .min_by(|x, y| x.0.distance(x.1).total_cmp(&y.0.distance(y.1)));
            let Some((pa, pb)) = shared else { continue };
            let ua = if pa == a.a { u } else { u.scaled(-1.0) };
            let vb = if pb == b.a { v } else { v.scaled(-1.0) };
            let sin = ua.cross(vb).norm();
            let cos = ua.dot(vb);
            // Folded back on each other (cos near 1) or a sharp bend: the length of one segment
            // within the other's radius is r / sin(angle).
            if cos > 0.0 && sin > 1e-12 {
                for (x, y) in [(a, b), (b, a)] {
                    let inside = (y.radius / sin).min(x.length());
                    let frac = inside / x.length();
                    if frac > JUNCTION_OVERLAP_FRACTION {
                        out.push(Finding {
                            severity: Severity::Warning,
                            kind: "sharp-junction",
                            segment: Some(x.index),
                            message: format!(
                                "{:.0}% of segment {} lies inside segment {}'s wire volume at their \
                                 {:.0}° junction; too sharp an angle or too short and fat a segment",
                                frac * 100.0,
                                x.index,
                                y.index,
                                cos.acos().to_degrees()
                            ),
                        });
                        break;
                    }
                }
            }
        }
    }

    out.extend(parallel_segmentation(geom, lambda, &parallel_pairs));
    out.extend(heights(geom, ctx, lambda, shown));
    out
}

/// Close parallel wires (pairs of tags found segment by segment) that overlap by more than a
/// tenth of a wavelength but are segmented differently. 4nec2 compares one rounded integer per
/// wire; here it is the wire's segments per wavelength.
fn parallel_segmentation(geom: &Geometry, lambda: Option<f64>, pairs: &[(i32, i32, usize, usize)]) -> Vec<Finding> {
    let Some(l) = lambda else { return Vec::new() };
    let mut out = Vec::new();
    let mut seen: Vec<(i32, i32)> = Vec::new();
    for &(ta, tb, i, _) in pairs {
        let key = (ta.min(tb), ta.max(tb));
        if seen.contains(&key) {
            continue;
        }
        // Overlap: the close segments of the first tag, measured along it.
        let close: f64 = pairs
            .iter()
            .filter(|p| (p.0, p.1) == (ta, tb))
            .map(|p| p.2)
            .collect::<std::collections::BTreeSet<usize>>()
            .into_iter()
            .map(|k| geom.segments[k].length())
            .sum();
        if close <= l * PARALLEL_OVERLAP_WAVELENGTHS {
            continue;
        }
        seen.push(key);
        let density = |tag: i32| {
            let (n, len) = geom.by_tag(tag).fold((0usize, 0.0), |(n, s), seg| (n + 1, s + seg.length()));
            #[allow(clippy::cast_precision_loss)]
            let d = n as f64 * l / len.max(1e-300);
            d.round_ties_even()
        };
        let (da, db) = (density(ta), density(tb));
        if (da - db).abs() >= 1.0 {
            out.push(Finding {
                severity: Severity::Warning,
                kind: "unequal-parallel-segmentation",
                segment: Some(geom.segments[i].index),
                message: format!(
                    "wires {ta} and {tb} run parallel within λ/20 for {:.2} λ but have {da} and {db} \
                     segments per wavelength; segment close parallel wires alike",
                    close / l
                ),
            });
        }
    }
    out
}

/// Wires too low for the ground model, or below it.
fn heights(geom: &Geometry, ctx: &LintContext, lambda: Option<f64>, shown: &dyn Fn(&Segment) -> bool) -> Vec<Finding> {
    let mut out = Vec::new();
    if ctx.ground == LintGround::FreeSpace {
        return out;
    }
    let mut seen_tags: Vec<i32> = Vec::new();
    for s in geom.segments.iter().filter(|s| shown(s) && s.tag < AUXILIARY_TAG) {
        let low = s.a.z.min(s.b.z);
        if low < -1e-9 * (1.0 + s.length()) {
            out.push(Finding {
                severity: Severity::Error,
                kind: "below-ground",
                segment: Some(s.index),
                message: "this segment is below the ground surface (Z < 0), which NEC-2 cannot model".into(),
            });
            continue;
        }
        let elevation = s.a.to(s.b).normalized().map_or(90.0, |u| u.z.abs().min(1.0).asin().to_degrees());
        if ctx.ground == LintGround::Reflection
            && let Some(l) = lambda
            && !seen_tags.contains(&s.tag)
        {
            let limit = if elevation < HORIZONTAL_DEGREES { REFLECTION_MIN_HEIGHT_HORIZONTAL } else { REFLECTION_MIN_HEIGHT_WAVELENGTHS };
            if low / l < limit {
                seen_tags.push(s.tag);
                out.push(Finding {
                    severity: Severity::Warning,
                    kind: "low-over-reflection-ground",
                    segment: Some(s.index),
                    message: format!(
                        "wire {} comes within {:.3} wavelengths of the ground (under {limit}); the \
                         reflection-coefficient ground wants several tenths of a wavelength, so use \
                         Sommerfeld ground (GN 2)",
                        s.tag,
                        low / l
                    ),
                });
            }
        }
        if matches!(ctx.ground, LintGround::Sommerfeld | LintGround::Perfect | LintGround::MiniNec)
            && low > 1e-9 * (1.0 + s.length())
            && low < SOMMERFELD_MIN_HEIGHT_RADII * s.radius
            && !seen_tags.contains(&s.tag)
        {
            seen_tags.push(s.tag);
            out.push(Finding {
                severity: Severity::Warning,
                kind: "too-close-to-ground",
                segment: Some(s.index),
                message: format!(
                    "wire {} is {:.2} radii above the ground, closer than 3 radii to its image, which \
                     the thin-wire approximation cannot represent; bond it with GE 1 or raise it",
                    s.tag,
                    low / s.radius
                ),
            });
        }
    }
    out
}
