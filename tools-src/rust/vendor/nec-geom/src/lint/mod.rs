//! Model validity checks.
//!
//! NEC's approximations have documented limits, and exceeding them does not produce an error —
//! it produces a plausible-looking wrong answer. 4nec2 lets you build such a model with no
//! indication, which is the single biggest usability gap this project aims to close.
//!
//! Every threshold here is quoted from the NEC-2 user guide (`C:\4nec2\Nec2.doc`) or read from
//! 4nec2's own checks (its segment check, `FUN_004cba00` in `4nec2.exe`, decompiled with REA;
//! see `docs/4NEC2-ANALYSIS.md`) rather than invented, and the source is kept next to the
//! constant so it can be checked.
//!
//! Wires tagged [`AUXILIARY_TAG`] or above are left out: they are dummy wires that carry a
//! current source or a matching network, not part of the antenna (4nec2: ">= 9800 is
//! invisible").

mod geometry;

use crate::connect::Junction;
use crate::geometry::{Geometry, GroundFlag};
use nec_core::Frequency;

/// Tags from here up are dummy wires, hidden and not checked (4nec2's rule).
pub const AUXILIARY_TAG: i32 = 9800;

/// Segment length above which the segment is too long to trust at all, in wavelengths: 4nec2's
/// segment check errors above `wlen/5` (and its guidelines allow up to λ/5 only "on long
/// straight segments").
pub const MAX_SEGMENT_WAVELENGTHS_HARD: f64 = 0.2;

/// Radius above which a segment is fat for the thin-wire approximation, in wavelengths: 4nec2
/// warns above `wlen/100` and errors above `wlen/30` (`FUN_004cba00`). The manual's condition
/// is that `ka` be small: "the wire radius ... small compared to the wavelength".
pub const RADIUS_WAVELENGTHS_WARN: f64 = 1.0 / 100.0;
/// See [`RADIUS_WAVELENGTHS_WARN`].
pub const RADIUS_WAVELENGTHS_ERROR: f64 = 1.0 / 30.0;

/// With the extended thin-wire kernel the length/radius limits drop: "With the extended
/// thin-wire kernel, Delta/a may be as small as 2 for the same accuracy" (1 %), and "down to 0.5"
/// for reasonable currents.
pub const EK_RATIO_GOOD: f64 = 2.0;
/// See [`EK_RATIO_GOOD`].
pub const EK_RATIO_FLOOR: f64 = 0.5;

/// Adjacent segments whose lengths differ by more than this factor (4nec2: "differs more than
/// 5 * seg-len").
pub const JUNCTION_LENGTH_RATIO: f64 = 5.0;

/// Adjacent segments whose radii differ by more than these factors: a warning above 5, an error
/// above 10 (4nec2: "differs more than 5 * radius", "differs more than 10 * radius").
pub const JUNCTION_RADIUS_WARN: f64 = 5.0;
/// See [`JUNCTION_RADIUS_WARN`].
pub const JUNCTION_RADIUS_ERROR: f64 = 10.0;

/// At a junction, the shorter segment's length over the fatter one's radius: a warning below 6,
/// an error below 2 (4nec2's "Len/rad at junction" check).
pub const JUNCTION_LEN_RAD_WARN: f64 = 6.0;
/// See [`JUNCTION_LEN_RAD_WARN`].
pub const JUNCTION_LEN_RAD_ERROR: f64 = 2.0;

/// The largest surface patch, in square wavelengths: 4nec2 errors when "A > (wl*wl)/25", the
/// 0.2 λ × 0.2 λ of its patch guideline ("Maximum patch size may not exceed 0.2 * 0.2 WL").
pub const MAX_PATCH_AREA_WAVELENGTHS2: f64 = 1.0 / 25.0;

/// The ground a model is checked against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LintGround {
    /// Free space.
    #[default]
    FreeSpace,
    /// Perfectly conducting ground.
    Perfect,
    /// Finite ground by the reflection-coefficient approximation (`GN 0`).
    Reflection,
    /// Finite ground by the Sommerfeld/Norton method (`GN 2`).
    Sommerfeld,
    /// 4nec2's "MiniNec" ground (`GN 3`): perfect for the currents, finite for the far field.
    MiniNec,
}

/// What the checks need to know beyond the structure.
#[derive(Debug, Clone, Default)]
pub struct LintContext {
    /// The frequency, for the wavelength rules (skipped without one).
    pub frequency: Option<Frequency>,
    /// Whether the extended thin-wire kernel (`EK`) is in use.
    pub extended_kernel: bool,
    /// The ground.
    pub ground: LintGround,
    /// Segments (1-based) carrying a voltage source.
    pub sources: Vec<usize>,
    /// Segments (1-based) carrying a lumped load.
    pub loads: Vec<usize>,
    /// Segments (1-based) that are transmission-line or network ports. A source there on a short
    /// isolated wire is the usual way to feed a line, not a mistake.
    pub ports: Vec<usize>,
}

/// Segment length above which the manual's rule of thumb is violated, in wavelengths.
///
/// > "As a rule of thumb, segment lengths should be less than 0.1 wavelength at the desired
/// > frequency. Somewhat longer segments may be used on long wires with no abrupt changes,
/// > while shorter segments, 0.05 wavelength or less, may be required in modeling critical
/// > regions of an [antenna]."
pub const MAX_SEGMENT_WAVELENGTHS: f64 = 0.1;

/// The tighter length suggested for critical regions, in wavelengths. See
/// [`MAX_SEGMENT_WAVELENGTHS`] for the quotation.
pub const CRITICAL_SEGMENT_WAVELENGTHS: f64 = 0.05;

/// Segment length below which the current expansion becomes ill-conditioned, in wavelengths.
///
/// > "Extremely short segments, less than about 10-3 Lambda, should also be avoided since the
/// > similarity of the constant and cosine components of the current expansion [degrades the
/// > solution]."
pub const MIN_SEGMENT_WAVELENGTHS: f64 = 1.0e-3;

/// Length-to-radius ratio needed for about 1% error with the standard thin-wire kernel.
///
/// > "Studies of the computed field on a segment due to its own current have shown that with
/// > the thin-wire kernel, Delta/a must be greater than about 8 for errors of less than 1%.
/// > With the extended thin-wire kernel, Delta/a may be as small as 2 for the same accuracy."
pub const THIN_WIRE_RATIO_GOOD: f64 = 8.0;

/// Length-to-radius ratio below which the standard kernel should not be used at all.
///
/// > "Reasonable current solutions have been obtained with the thin-wire kernel for Delta/a
/// > down to about 2 and with the extended thin-wire kernel for Delta/a down to 0.5."
pub const THIN_WIRE_RATIO_FLOOR: f64 = 2.0;

/// Height, as a fraction of segment length, below which `GE 1` bonds a wire end to ground.
///
/// > "If the height of a horizontal wire is less than 10-3 times the segment length, I1 equal
/// > to 1 will connect the end of every segment in the wire to ground. I1 should be -1 to
/// > avoid this disaster."
///
/// This is both a connectivity rule and a trap worth warning about: the manual's own word for
/// the outcome is "disaster".
pub const GROUND_BOND_HEIGHT_FRACTION: f64 = 1.0e-3;

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Worth knowing, not wrong.
    Info,
    /// Results are likely degraded.
    Warning,
    /// Results should not be trusted.
    Error,
}

/// One validity finding.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// How serious it is.
    pub severity: Severity,
    /// Short machine-readable kind, for grouping in the UI.
    pub kind: &'static str,
    /// The segment number this concerns, if it is segment-specific.
    pub segment: Option<usize>,
    /// Human-readable explanation, including the measured value.
    pub message: String,
}

/// Check a structure, optionally at a frequency.
///
/// Wavelength-dependent rules are skipped when `frequency` is `None`, since segment length in
/// wavelengths is meaningless without one. Ratio and connectivity rules always apply.
#[must_use]
pub fn lint(geom: &Geometry, frequency: Option<Frequency>) -> Vec<Finding> {
    lint_with(geom, &LintContext { frequency, ..LintContext::default() })
}

/// Check a structure with what else is known about the model: ground, kernel, sources and loads.
#[must_use]
pub fn lint_with(geom: &Geometry, ctx: &LintContext) -> Vec<Finding> {
    let mut out = Vec::new();
    let lambda = ctx.frequency.map(Frequency::wavelength_m);
    let shown = |s: &crate::Segment| s.tag < AUXILIARY_TAG;

    for seg in geom.segments.iter().filter(|s| shown(s)) {
        let len = seg.length();

        if let Some(lambda) = lambda {
            let in_waves = len / lambda;
            if in_waves > MAX_SEGMENT_WAVELENGTHS_HARD {
                out.push(Finding {
                    severity: Severity::Error,
                    kind: "segment-too-long",
                    segment: Some(seg.index),
                    message: format!(
                        "segment is {in_waves:.3} wavelengths long, over the {MAX_SEGMENT_WAVELENGTHS_HARD} \
                         wavelengths beyond which the current expansion cannot follow the current"
                    ),
                });
            } else if in_waves > MAX_SEGMENT_WAVELENGTHS {
                out.push(Finding {
                    severity: Severity::Warning,
                    kind: "segment-too-long",
                    segment: Some(seg.index),
                    message: format!(
                        "segment is {in_waves:.3} wavelengths long; the guideline is under \
                         {MAX_SEGMENT_WAVELENGTHS} (and {CRITICAL_SEGMENT_WAVELENGTHS} in \
                         critical regions)"
                    ),
                });
            }
            if in_waves < MIN_SEGMENT_WAVELENGTHS {
                out.push(Finding {
                    severity: Severity::Warning,
                    kind: "segment-too-short",
                    segment: Some(seg.index),
                    message: format!(
                        "segment is only {in_waves:.2e} wavelengths long; below \
                         {MIN_SEGMENT_WAVELENGTHS:.0e} the constant and cosine parts of the \
                         current expansion become too similar and the solution degrades"
                    ),
                });
            }
        }

        if let Some(lambda) = lambda {
            let r = seg.radius / lambda;
            if r > RADIUS_WAVELENGTHS_WARN {
                out.push(Finding {
                    severity: if r > RADIUS_WAVELENGTHS_ERROR { Severity::Error } else { Severity::Warning },
                    kind: "radius-too-large",
                    segment: Some(seg.index),
                    message: format!(
                        "radius is {r:.4} wavelengths; the thin-wire approximation wants it under \
                         λ/100 (and fails past λ/30)"
                    ),
                });
            }
        }

        let ratio = seg.length_over_radius();
        let (good, floor) = if ctx.extended_kernel {
            (EK_RATIO_GOOD, EK_RATIO_FLOOR)
        } else {
            (THIN_WIRE_RATIO_GOOD, THIN_WIRE_RATIO_FLOOR)
        };
        let kernel = if ctx.extended_kernel { "extended thin-wire" } else { "thin-wire" };
        if ratio < floor {
            out.push(Finding {
                severity: Severity::Error,
                kind: "thin-wire-ratio-unusable",
                segment: Some(seg.index),
                message: format!(
                    "length/radius is {ratio:.2}; the {kernel} kernel is not usable below about {floor}{}",
                    if ctx.extended_kernel { "" } else { " (the extended kernel, EK, goes to 0.5)" }
                ),
            });
        } else if ratio < good {
            out.push(Finding {
                severity: Severity::Warning,
                kind: "thin-wire-ratio-low",
                segment: Some(seg.index),
                message: format!(
                    "length/radius is {ratio:.2}; about {good} is needed for 1% accuracy with the \
                     {kernel} kernel"
                ),
            });
        }

        if seg.radius <= 0.0 {
            out.push(Finding {
                severity: Severity::Error,
                kind: "zero-radius",
                segment: Some(seg.index),
                message: "wire radius is zero or negative".to_string(),
            });
        }
    }

    // The manual's "disaster": a nominally horizontal wire sitting just above Z = 0 under
    // GE 1 gets every segment end shorted to ground.
    if geom.ground == GroundFlag::GroundedEnds {
        for seg in &geom.segments {
            let limit = GROUND_BOND_HEIGHT_FRACTION * seg.length();
            let a_near = seg.a.z.abs() <= limit && seg.a.z != 0.0;
            let b_near = seg.b.z.abs() <= limit && seg.b.z != 0.0;
            if a_near || b_near {
                out.push(Finding {
                    severity: Severity::Warning,
                    kind: "unintended-ground-bond",
                    segment: Some(seg.index),
                    message: format!(
                        "end height is within {GROUND_BOND_HEIGHT_FRACTION:.0e} of the segment \
                         length from Z=0, so GE 1 will bond it to ground; use GE -1 if that is \
                         not intended"
                    ),
                });
            }
        }
    }

    // Steps across a junction between wires: "A large radius change between connected segments
    // may decrease accuracy", and likewise a large change of length; and a short segment against
    // a fat one. Along one wire (one card) the segment rules already cover it.
    let card_of = |k: usize| (geom.origins.get(k).copied().unwrap_or(0), geom.segments[k].tag);
    for (i, conn) in geom.connectivity.iter().enumerate() {
        let here = &geom.segments[i];
        if !shown(here) {
            continue;
        }
        for junction in [&conn.end1, &conn.end2] {
            let Junction::Joined(ends) = junction else { continue };
            for end in ends {
                let Some(other) = geom.segment(end.segment) else { continue };
                if other.index <= here.index || !shown(other) {
                    continue; // report each pair once
                }
                if card_of(i) == card_of(other.index - 1) {
                    continue;
                }
                let (lo, hi) = if here.radius < other.radius { (here.radius, other.radius) } else { (other.radius, here.radius) };
                if lo > 0.0 && hi / lo > JUNCTION_RADIUS_WARN {
                    out.push(Finding {
                        severity: if hi / lo > JUNCTION_RADIUS_ERROR { Severity::Error } else { Severity::Warning },
                        kind: "radius-jump",
                        segment: Some(here.index),
                        message: format!(
                            "radius changes by {:.1}x across the junction with segment {}; \
                             large steps between connected segments reduce accuracy",
                            hi / lo,
                            other.index
                        ),
                    });
                }
                let (la, lb) = (here.length(), other.length());
                let (short, long) = if la < lb { (la, lb) } else { (lb, la) };
                if short > 0.0 && long / short > JUNCTION_LENGTH_RATIO {
                    out.push(Finding {
                        severity: Severity::Warning,
                        kind: "length-jump",
                        segment: Some(here.index),
                        message: format!(
                            "segment length changes by {:.1}x across the junction with segment {}; \
                             taper the segmentation instead",
                            long / short,
                            other.index
                        ),
                    });
                }
                if hi > 0.0 && short / hi < JUNCTION_LEN_RAD_WARN {
                    let r = short / hi;
                    out.push(Finding {
                        severity: if r < JUNCTION_LEN_RAD_ERROR { Severity::Error } else { Severity::Warning },
                        kind: "junction-length-radius",
                        segment: Some(here.index),
                        message: format!(
                            "at the junction with segment {}, the shorter segment is only {r:.2} \
                             times the fatter one's radius; their volumes overlap",
                            other.index
                        ),
                    });
                }
            }
        }
    }

    // Patches no bigger than 0.2 λ square.
    if let Some(lambda) = lambda {
        for p in &geom.patches {
            let a = p.area / (lambda * lambda);
            if a > MAX_PATCH_AREA_WAVELENGTHS2 {
                out.push(Finding {
                    severity: Severity::Error,
                    kind: "patch-too-large",
                    segment: None,
                    message: format!(
                        "patch {} covers {a:.3} square wavelengths; patches should be no larger than \
                         0.2 λ × 0.2 λ (0.04)",
                        p.index
                    ),
                });
            }
        }
    }

    // "No voltage sources (EX 0) or loads at wire open ends": the current at an open end is
    // forced to zero, so a source or load there sees almost nothing.
    for (list, what) in [(&ctx.sources, "source"), (&ctx.loads, "load")] {
        for &s in list.iter() {
            let (Some(seg), Some(conn)) = (geom.segment(s), geom.connectivity.get(s.wrapping_sub(1))) else { continue };
            if !shown(seg) || ctx.ports.contains(&s) {
                continue;
            }
            if matches!(conn.end1, Junction::Free) || matches!(conn.end2, Junction::Free) {
                out.push(Finding {
                    severity: Severity::Warning,
                    kind: "open-end-feed",
                    segment: Some(s),
                    message: format!(
                        "this {what} is on a segment at an open wire end, where the current is \
                         forced to zero; move it inward"
                    ),
                });
            }
        }
    }

    out.extend(geometry::check(geom, ctx, &shown));

    // "Segments (or patches) may not overlap since the division of current between two
    // overlapping segments is indeterminate. Overlapping segments may result in a singular
    // matrix equation." Two straight segments joined to each other at *both* ends share both
    // end points, so they are the same segment twice — typically a wire entered twice. NEC-2 does
    // not reject this, and its matrix fill (`SBF`) then disagrees with its own basis functions
    // (`TBF`); see `nec_kernel::pieces_by_segment`.
    for (i, conn) in geom.connectivity.iter().enumerate() {
        let here = &geom.segments[i];
        let joined = |j: &crate::Junction| match j {
            crate::Junction::Joined(ends) => ends.iter().map(|e| e.segment).collect(),
            _ => Vec::new(),
        };
        let at_end2 = joined(&conn.end2);
        for other in joined(&conn.end1) {
            if other > here.index && at_end2.contains(&other) {
                out.push(Finding {
                    severity: Severity::Error,
                    kind: "coincident-segments",
                    segment: Some(here.index),
                    message: format!(
                        "segment {} shares both end points with segment {other}, so the two \
                         overlap; NEC cannot divide current between overlapping segments",
                        here.index
                    ),
                });
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

    /// NEC Studio explains every check in words (app/src/lib/findings.ts, typed over the
    /// `FindingKind` union in app/src/lib/types.ts). This keeps that union in step with the kinds
    /// the linter emits, so a new check cannot reach the app unexplained.
    #[test]
    fn the_app_knows_every_kind_of_finding() {
        let types = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/src/lib/types.ts");
        let Ok(ts) = std::fs::read_to_string(&types) else {
            return; // built outside the repository
        };
        let union = ts
            .split("export type FindingKind =")
            .nth(1)
            .and_then(|s| s.split(';').next())
            .expect("types.ts declares FindingKind");
        let app: std::collections::BTreeSet<&str> = union.split('"').skip(1).step_by(2).collect();
        let marker = concat!("kind: ", "\"");
        let lint: std::collections::BTreeSet<&str> = [include_str!("mod.rs"), include_str!("geometry.rs")]
            .iter()
            .flat_map(|src| src.split(marker).skip(1))
            .filter_map(|rest| rest.split('"').next())
            .filter(|k| !k.is_empty() && k.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'))
            .collect();
        assert_eq!(app, lint, "FindingKind in app/src/lib/types.ts differs from the linter's kinds");
    }

    fn build(src: &str) -> Geometry {
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        Geometry::build(&deck).expect("builds")
    }

    fn kinds(f: &[Finding]) -> Vec<&str> {
        let mut v: Vec<&str> = f.iter().map(|x| x.kind).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    #[test]
    fn example1_is_clean_at_its_design_frequency() {
        // models/Example1.nec at 300 MHz: nine 0.0537 m segments, radius 1e-4.
        // 0.0537 m is 0.0538 wavelengths and length/radius is 537, so nothing should fire.
        let g = build("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n");
        let f = lint(&g, Some(Frequency::from_mhz(300.0).expect("valid")));
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn long_segments_are_flagged() {
        // One segment spanning a full half wavelength at 300 MHz is ~0.48 wavelengths.
        let g = build("GW 1 1 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n");
        let f = lint(&g, Some(Frequency::from_mhz(300.0).expect("valid")));
        assert!(kinds(&f).contains(&"segment-too-long"), "{f:?}");
    }

    #[test]
    fn fat_wire_trips_the_thin_wire_ratio() {
        // A 1 m segment of 0.4 m radius: ratio 2.5, below the "1% accuracy" threshold of 8
        // but above the floor of 2.
        let g = build("GW 1 1 0 0 0 1 0 0 0.4\nGE 0\nEN\n");
        let f = lint(&g, None);
        assert!(kinds(&f).contains(&"thin-wire-ratio-low"), "{f:?}");

        // Radius equal to length: ratio 1, below the usable floor.
        let g = build("GW 1 1 0 0 0 1 0 0 1.0\nGE 0\nEN\n");
        let f = lint(&g, None);
        assert!(kinds(&f).contains(&"thin-wire-ratio-unusable"), "{f:?}");
        assert_eq!(f.iter().map(|x| x.severity).max(), Some(Severity::Error));
    }

    #[test]
    fn wavelength_rules_are_skipped_without_a_frequency() {
        let g = build("GW 1 1 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n");
        let f = lint(&g, None);
        assert!(!kinds(&f).contains(&"segment-too-long"));
    }

    #[test]
    fn the_manuals_ground_bonding_disaster_is_warned_about() {
        // A horizontal wire 1 mm above ground with 1 m segments: 1e-3 of the segment length,
        // so GE 1 shorts it to ground. The manual calls this a disaster.
        let g = build("GW 1 4 0 0 0.001 4 0 0.001 1e-3\nGE 1\nEN\n");
        let f = lint(&g, None);
        assert!(kinds(&f).contains(&"unintended-ground-bond"), "{f:?}");

        // The same wire under GE -1 is deliberate and must not warn.
        let g = build("GW 1 4 0 0 0.001 4 0 0.001 1e-3\nGE -1\nEN\n");
        let f = lint(&g, None);
        assert!(!kinds(&f).contains(&"unintended-ground-bond"), "{f:?}");
    }

    #[test]
    fn a_wire_entered_twice_is_an_error() {
        let g = build(
            "GW 1 3 0 -.1 0 0 .1 0 1e-3\n\
             GW 2 3 0 -.1 0 0 .1 0 1e-3\n\
             GE 0\nEN\n",
        );
        let f = lint(&g, None);
        let hits: Vec<_> = f.iter().filter(|x| x.kind == "coincident-segments").collect();
        // Segment k of the first wire coincides with segment k+3 of the second, once per pair.
        assert_eq!(hits.len(), 3, "{f:?}");
        assert!(hits.iter().all(|x| x.severity == Severity::Error));

        // The same wire reversed is still the same segments.
        let g = build(
            "GW 1 3 0 -.1 0 0 .1 0 1e-3\n\
             GW 2 3 0 .1 0 0 -.1 0 1e-3\n\
             GE 0\nEN\n",
        );
        assert_eq!(lint(&g, None).iter().filter(|x| x.kind == "coincident-segments").count(), 3);

        // A crossing at a shared end point is not an overlap.
        let g = build(
            "GW 1 2 0 -.1 0 0 .1 0 1e-3\n\
             GW 2 2 -.1 0 0 .1 0 0 1e-3\n\
             GE 0\nEN\n",
        );
        assert!(!kinds(&lint(&g, None)).contains(&"coincident-segments"));
    }

    #[test]
    fn radius_jump_across_a_junction_is_noted() {
        let g = build(
            "GW 1 1 0 0 0 1 0 0 1e-3\n\
             GW 2 1 1 0 0 2 0 0 1e-1\n\
             GE 0\nEN\n",
        );
        let f = lint(&g, None);
        assert!(kinds(&f).contains(&"radius-jump"), "{f:?}");
        // Reported once, not once per end.
        assert_eq!(f.iter().filter(|x| x.kind == "radius-jump").count(), 1);
    }

    fn at(mhz: f64) -> LintContext {
        LintContext { frequency: Some(Frequency::from_mhz(mhz).expect("f")), ..LintContext::default() }
    }

    fn has(src: &str, ctx: &LintContext, kind: &str) -> bool {
        lint_with(&build(src), ctx).iter().any(|f| f.kind == kind)
    }

    // A clean reference: a 21-segment half-wave dipole at 300 MHz (λ ≈ 1 m).
    const DIPOLE: &str = "CE\nGW 1 21 0 -.24 0 0 .24 0 .001\nGE 0\nEN\n";

    #[test]
    fn the_reference_dipole_is_clean_of_the_new_rules() {
        let mut ctx = at(299.8);
        ctx.sources = vec![11];
        let f = lint_with(&build(DIPOLE), &ctx);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn a_very_long_segment_is_an_error() {
        let f = lint_with(&build("CE\nGW 1 2 0 -.3 0 0 .3 0 .001\nGE 0\nEN\n"), &at(299.8));
        assert!(f.iter().any(|x| x.kind == "segment-too-long" && x.severity == Severity::Error), "{f:?}");
    }

    #[test]
    fn a_fat_wire_against_the_wavelength() {
        let fat = "CE\nGW 1 21 0 -.24 0 0 .24 0 .02\nGE 0\nEN\n";
        let f = lint_with(&build(fat), &at(299.8));
        assert!(f.iter().any(|x| x.kind == "radius-too-large" && x.severity == Severity::Warning), "{f:?}");
        let fatter = "CE\nGW 1 21 0 -.24 0 0 .24 0 .04\nGE 0\nEN\n";
        let f = lint_with(&build(fatter), &at(299.8));
        assert!(f.iter().any(|x| x.kind == "radius-too-large" && x.severity == Severity::Error), "{f:?}");
    }

    #[test]
    fn the_extended_kernel_relaxes_length_over_radius() {
        // 0.0229 m segments, radius 0.005: ratio 4.6 — low for the thin-wire kernel, fine with EK.
        let src = "CE\nGW 1 21 0 -.24 0 0 .24 0 .005\nGE 0\nEN\n";
        assert!(has(src, &LintContext::default(), "thin-wire-ratio-low"));
        let ek = LintContext { extended_kernel: true, ..LintContext::default() };
        assert!(!has(src, &ek, "thin-wire-ratio-low"));
    }

    #[test]
    fn junction_steps_in_length_and_radius() {
        // A 1 m wire of 2 segments joined to a 0.05 m wire of 5: lengths 0.5 and 0.01.
        let len = "CE\nGW 1 2 0 0 0 0 0 1 .001\nGW 2 5 0 0 1 0 0 1.05 .001\nGE 0\nEN\n";
        assert!(has(len, &LintContext::default(), "length-jump"));
        // Radius 0.001 against 0.015: a 15x step is an error.
        let rad = "CE\nGW 1 5 0 0 0 0 0 1 .001\nGW 2 5 0 0 1 0 0 2 .015\nGE 0\nEN\n";
        let f = lint_with(&build(rad), &LintContext::default());
        assert!(f.iter().any(|x| x.kind == "radius-jump" && x.severity == Severity::Error), "{f:?}");
    }

    #[test]
    fn a_short_segment_against_a_fat_neighbour() {
        // 0.01 m segments meeting a wire of radius 0.004: 2.5 radii — a warning.
        let src = "CE\nGW 1 10 0 0 0 0 0 .1 .0005\nGW 2 4 0 0 .1 0 0 .5 .004\nGE 0\nEN\n";
        let f = lint_with(&build(src), &LintContext::default());
        assert!(f.iter().any(|x| x.kind == "junction-length-radius"), "{f:?}");
    }

    #[test]
    fn a_source_on_an_open_end() {
        let ctx = LintContext { sources: vec![1], ..LintContext::default() };
        assert!(has(DIPOLE, &ctx, "open-end-feed"));
        let ctx = LintContext { sources: vec![11], ..LintContext::default() };
        assert!(!has(DIPOLE, &ctx, "open-end-feed"));
    }

    #[test]
    fn a_big_patch() {
        let src = "CE\nSP 0 0 0 0 0 0 0 .1\nGE 0\nEN\n";
        assert!(has(src, &at(299.8), "patch-too-large"));
        assert!(!has(src, &at(30.0), "patch-too-large"));
    }

    #[test]
    fn wires_that_cross_without_joining() {
        let src = "CE\nGW 1 11 0 -.24 0 0 .24 0 .001\nGW 2 11 -.24 0 0 .24 0 0 .001\nGE 0\nEN\n";
        assert!(has(src, &LintContext::default(), "crossing-wires"));
    }

    #[test]
    fn wires_too_close_and_laid_over_each_other() {
        // Parallel, 1.5 mm apart with 1 mm radii: the volumes intersect.
        let near = "CE\nGW 1 11 0 -.24 0 0 .24 0 .001\nGW 2 11 .0015 -.24 0 .0015 .24 0 .001\nGE 0\nEN\n";
        assert!(has(near, &LintContext::default(), "overlaid-wires") || has(near, &LintContext::default(), "intersecting-wires"));
        // Skew wires passing 1.5 mm apart.
        let skew = "CE\nGW 1 11 0 -.24 0 0 .24 0 .001\nGW 2 11 -.24 0 .0015 .24 0 .0015 .001\nGE 0\nEN\n";
        assert!(has(skew, &LintContext::default(), "intersecting-wires"));
    }

    #[test]
    fn a_sharp_fat_junction() {
        // Two fat wires folded back at 10° from a common end.
        let src = "CE\nGW 1 5 0 0 0 0 0 .1 .005\nGW 2 5 0 0 0 .0174 0 .0985 .005\nGE 0\nEN\n";
        assert!(has(src, &LintContext::default(), "sharp-junction"));
        // A right angle with thin wire is fine.
        let ok = "CE\nGW 1 5 0 0 0 0 0 .1 .0005\nGW 2 5 0 0 0 .1 0 0 .0005\nGE 0\nEN\n";
        assert!(!has(ok, &LintContext::default(), "sharp-junction"));
    }

    #[test]
    fn close_parallel_wires_segmented_differently() {
        // Two half-wave wires 2 cm apart (λ/50), 21 and 9 segments.
        let src = "CE\nGW 1 21 0 -.24 0 0 .24 0 .001\nGW 2 9 .02 -.24 0 .02 .24 0 .001\nGE 0\nEN\n";
        assert!(has(src, &at(299.8), "unequal-parallel-segmentation"));
        let same = src.replace("GW 2 9", "GW 2 21");
        assert!(!has(&same, &at(299.8), "unequal-parallel-segmentation"));
    }

    #[test]
    fn heights_over_the_ground_models() {
        // A horizontal dipole 5 cm up at 300 MHz (0.05 λ).
        let low = "CE\nGW 1 21 0 -.24 .05 0 .24 .05 .001\nGE 0\nEN\n";
        let refl = LintContext { ground: LintGround::Reflection, ..at(299.8) };
        assert!(has(low, &refl, "low-over-reflection-ground"));
        let high = low.replace(".05 0 .24 .05", ".3 0 .24 .3").replace("-.24 .05", "-.24 .3");
        assert!(!has(&high, &refl, "low-over-reflection-ground"));
        // 2 mm up with a 1 mm radius over Sommerfeld ground: within 3 radii.
        let skim = "CE\nGW 1 21 0 -.24 .002 0 .24 .002 .001\nGE 0\nEN\n";
        let som = LintContext { ground: LintGround::Sommerfeld, ..at(299.8) };
        assert!(has(skim, &som, "too-close-to-ground"));
        // Below ground.
        let buried = "CE\nGW 1 21 0 -.24 -.1 0 .24 -.1 .001\nGE 0\nEN\n";
        assert!(has(buried, &som, "below-ground"));
    }

    #[test]
    fn dummy_wires_are_not_checked() {
        let src = "CE\nGW 1 21 0 -.24 0 0 .24 0 .001\nGW 9801 1 -.0005 0 9801 .0005 0 9801 .1\nGE 0\nEN\n";
        let f = lint_with(&build(src), &at(299.8));
        assert!(f.is_empty(), "{f:?}");
    }
}
