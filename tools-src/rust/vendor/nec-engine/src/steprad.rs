//! Stepped-radius (tapered element) correction: W2DU's (David Leeson's) substitution of a
//! uniform element for one built of sections of different diameter, as 4nec2 applies it through
//! its `Convert.exe`.
//!
//! NEC-2's thin-wire model handles a sudden change of radius poorly, so the resonance of a Yagi
//! element made of telescoping tubes comes out wrong. Leeson's method replaces each such element
//! with one of a single radius and a corrected length that has the same self-impedance, from a
//! closed-form model of the tapered dipole. The arithmetic here is `Convert.exe`'s, read from
//! its decompilation (`FUN_00401120`, `C:\4nec2-decompiled\helpers\convert`), down to its
//! constants (21.13, 42.545, 56.93, 76.39, 4253, 68673, 14.67) and its π of 3.14159, and is
//! checked against `Convert.exe`'s own output (`cargo xtask steprad`).
//!
//! An element qualifies when, like `Convert.exe` requires:
//! * it is a straight run of segments between two free ends (a dipole, whose radii and lengths
//!   are symmetric about its centre) or between a ground connection and a free end (a monopole,
//!   fat end down);
//! * its radius changes somewhere along it;
//! * any source, line or network on it is at its centre (the base, for a monopole) — so a fed
//!   dipole needs an odd number of segments;
//! * no load other than wire conductivity (`LD 5`) is on it;
//! * the frequency is within 15 % of its quarter-wave resonance, where the method holds.
//!
//! The element's wire cards keep their segment counts, so nothing pointing at its segments moves;
//! their end points are scaled about the centre (the base) and all get the equivalent radius.
//! Unlike `Convert.exe`, which divides by the element's direction cosines and so only works for
//! elements along an axis, the scaling here is along the element's own axis.

use nec_cards::{Deck, Item};
use nec_core::geom::Point3;
use nec_geom::{Geometry, Junction, WireEnd};
use nec_solve::Program;

/// `Convert.exe`'s π: five decimals, kept so the results match its own.
#[allow(clippy::approx_constant)]
const PI: f64 = 3.14159;
/// `Convert.exe`'s speed of light, metres per second.
const C: f64 = 299_792_500.0;

/// A wire card's new end points and radius, in the card's own units (before `GS`).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CardChange {
    /// Deck line of the `GW` card.
    pub line: usize,
    /// `x1 y1 z1 x2 y2 z2 radius`.
    pub values: [f64; 7],
}

/// What the correction did.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct StepRadReport {
    /// One line per element considered: converted, or why not.
    pub log: Vec<String>,
}

/// One straight run of segments: 0-based indices in order, and how each end finishes.
struct Chain {
    segs: Vec<usize>,
    grounded_start: bool,
    grounded_end: bool,
}

/// The segment joined to segment `k`'s end `end`, when exactly one is, and which of its ends.
fn next(geom: &Geometry, k: usize, end: WireEnd) -> Option<(usize, WireEnd)> {
    let c = &geom.connectivity[k];
    let j = match end {
        WireEnd::A => &c.end1,
        WireEnd::B => &c.end2,
    };
    match j {
        Junction::Joined(v) if v.len() == 1 => Some((v[0].segment - 1, v[0].end)),
        _ => None,
    }
}

fn junction(geom: &Geometry, k: usize, end: WireEnd) -> &Junction {
    let c = &geom.connectivity[k];
    match end {
        WireEnd::A => &c.end1,
        WireEnd::B => &c.end2,
    }
}

fn other(end: WireEnd) -> WireEnd {
    match end {
        WireEnd::A => WireEnd::B,
        WireEnd::B => WireEnd::A,
    }
}

/// Straight runs that start at a free or grounded end and end at one, with no junction of three
/// or more wires and no bend between.
fn chains(geom: &Geometry) -> Vec<Chain> {
    let mut seen = vec![false; geom.segments.len()];
    let mut out = Vec::new();
    for start in 0..geom.segments.len() {
        if seen[start] {
            continue;
        }
        for start_end in [WireEnd::A, WireEnd::B] {
            let j = junction(geom, start, start_end);
            if !matches!(j, Junction::Free | Junction::Ground) {
                continue;
            }
            let dir = geom.segments[start].direction();
            let mut segs = vec![start];
            let (mut k, mut entered) = (start, start_end);
            let mut ok = true;
            let finish;
            loop {
                let out_end = other(entered);
                match junction(geom, k, out_end) {
                    Junction::Free => {
                        finish = false;
                        break;
                    }
                    Junction::Ground => {
                        finish = true;
                        break;
                    }
                    _ => {}
                }
                let Some((n, n_end)) = next(geom, k, out_end) else {
                    ok = false;
                    finish = false;
                    break;
                };
                let straight = dir
                    .zip(geom.segments[n].direction())
                    .is_some_and(|(u, v)| u.dot(v).abs() > 1.0 - 1e-6);
                if !straight || segs.contains(&n) {
                    ok = false;
                    finish = false;
                    break;
                }
                segs.push(n);
                k = n;
                entered = n_end;
            }
            for &s in &segs {
                seen[s] = true;
            }
            if ok {
                out.push(Chain { segs, grounded_start: matches!(j, Junction::Ground), grounded_end: finish });
            }
            break;
        }
    }
    out
}

/// Leeson's equivalent element for a half element of sections `(length, radius)` from the centre
/// (base) outward, the first counted at `w0` of its length: `(length factor, equivalent radius,
/// half-length)`. The new half-length is the old divided by the factor.
fn leeson(sections: &[(f64, f64)], w0: f64) -> (f64, f64, f64) {
    let weighted = |i: usize, l: f64| if i == 0 { l * w0 } else { l };
    let h: f64 = sections.iter().enumerate().map(|(i, &(l, _))| weighted(i, l)).sum();
    let mean_ln: f64 = sections.iter().enumerate().map(|(i, &(l, a))| weighted(i, l) * a.ln()).sum::<f64>() / h;
    let (mut s1, mut s2, mut s3, mut s4) = (0.0, 0.0, 0.0, 0.0);
    let mut z0 = 0.0;
    for (i, &(l, a)) in sections.iter().enumerate() {
        let z1 = z0 + weighted(i, l);
        let (t1, t0) = (PI * z1 / h, PI * z0 / h);
        let d = a.ln() - mean_ln;
        s1 += 60.0 * d * (t1.sin() - t0.sin());
        s2 += 120.0 * d * (t1.cos() * z1 - t0.cos() * z0) / h;
        s3 += -60.0 * d * (t1.cos() - t0.cos());
        s4 += 120.0 * d * (t1.sin() * z1 - t0.sin() * z0) / h;
        z0 = z1;
    }
    let omega = 120.0 * ((2.0 * h).ln() - mean_ln - 1.0);
    let d = 1.0 - (s3 - 21.13) / omega;
    let p = (42.545 - s1) * d;
    let q = (omega - 2.0 * (s3 - 21.13)) - (s2 + 56.93) * d - (s4 + 76.39) * (p / omega);
    let r = (p - (21.13 / omega + 1.0) * 42.545) / (q * PI);
    let factor = 1.0 - 2.0 * r;
    let e = -((4253.0 / omega + factor * q + 68673.0 / (omega * omega) + 14.67) / 120.0 + 1.0);
    let radius = 2.0 * (h / factor) * e.exp();
    (factor, radius, h)
}

/// Compute the corrections for every qualifying element of `geom` at `mhz`.
#[must_use]
pub fn corrections(deck: &Deck, geom: &Geometry, prog: &Program, mhz: f64) -> (Vec<CardChange>, StepRadReport) {
    let mut report = StepRadReport::default();
    let mut changes = Vec::new();
    // 4nec2: "Step-radius correction not allowed for structure (LD 5 0) loading", and the method
    // has no answer for an incident field.
    for it in &deck.items {
        let Item::Card(c) = it else { continue };
        if c.mnemonic == "LD" && c.int(0).ok() == Some(5) && c.int(1).ok() == Some(0) {
            report.log.push("Step-radius correction is not allowed with structure-wide LD 5 loading.".into());
            return (changes, report);
        }
    }
    if !prog.incident_fields().is_empty() {
        report.log.push("Step-radius correction is not applied with an incident-field excitation.".into());
        return (changes, report);
    }
    let gw: Vec<&nec_cards::Card> = deck
        .items
        .iter()
        .filter_map(|it| match it {
            Item::Card(c) if c.mnemonic == "GW" => Some(c),
            _ => None,
        })
        .collect();

    for chain in chains(geom) {
        let segs = &chain.segs;
        let n = segs.len();
        let radii: Vec<f64> = segs.iter().map(|&k| geom.segments[k].radius).collect();
        let lens: Vec<f64> = segs.iter().map(|&k| geom.segments[k].length()).collect();
        if radii.iter().all(|&r| (r - radii[0]).abs() <= 1e-9 * radii[0].max(1e-30)) {
            continue; // not stepped
        }
        let tag = geom.segments[segs[0]].tag;
        let name = format!("wire {tag} (segments {}–{})", geom.segments[segs[0]].index, geom.segments[segs[n - 1]].index);
        let rel = |a: f64, b: f64| (a - b).abs() <= 1e-6 * a.abs().max(b.abs()).max(1e-30);
        let dipole = (radii[0] - radii[n - 1]).abs() <= 1e-9;
        // Order from the centre (base) outward, and find the centre point.
        let (order, w0, centre, centre_seg): (Vec<usize>, f64, Point3, usize);
        if dipole {
            if chain.grounded_start || chain.grounded_end {
                report.log.push(format!("{name}: equal end radii but grounded; not converted"));
                continue;
            }
            if (0..n).any(|i| !rel(radii[i], radii[n - 1 - i])) {
                report.log.push(format!("{name}: radii not symmetric about the centre; not converted"));
                continue;
            }
            if (0..n).any(|i| !rel(lens[i], lens[n - 1 - i])) {
                report.log.push(format!("{name}: segment lengths not symmetric about the centre; not converted"));
                continue;
            }
            let mid = n / 2;
            order = (mid..n).collect();
            if n % 2 == 1 {
                w0 = 0.5;
                let s = &geom.segments[segs[mid]];
                centre = s.center();
            } else {
                w0 = 1.0;
                // The joint between the two halves.
                let (a, b) = (&geom.segments[segs[mid - 1]], &geom.segments[segs[mid]]);
                centre = [a.a, a.b]
                    .into_iter()
                    .flat_map(|p| [b.a, b.b].into_iter().map(move |q| (p, q)))
                    .min_by(|x, y| x.0.distance(x.1).total_cmp(&y.0.distance(y.1)))
                    .map_or(b.a, |(p, _)| p);
            }
            centre_seg = segs[mid];
        } else {
            // A monopole: from its fat end, which must be on the ground.
            let fat_first = radii[0] >= radii[n - 1];
            let grounded = if fat_first { chain.grounded_start } else { chain.grounded_end };
            if !grounded {
                report.log.push(format!("{name}: unequal end radii but no end on the ground plane; not converted"));
                continue;
            }
            order = if fat_first { (0..n).collect() } else { (0..n).rev().collect() };
            w0 = 1.0;
            let base = &geom.segments[segs[order[0]]];
            centre = if base.a.z.abs() <= base.b.z.abs() { base.a } else { base.b };
            centre_seg = segs[order[0]];
        }
        let on_chain = |seg: usize| segs.contains(&seg);
        // Sources, lines and networks only at the centre (base); no loads but conductivity.
        let feeds_ok = prog.sources.iter().filter(|s| on_chain(s.segment)).all(|s| s.segment == centre_seg && (!dipole || n % 2 == 1))
            && prog
                .networks
                .iter()
                .flat_map(|nw| [nw.port1, nw.port2])
                .filter(|&p| on_chain(p))
                .all(|p| p == centre_seg && (!dipole || n % 2 == 1));
        if !feeds_ok {
            report.log.push(format!("{name}: a source, line or network is not at the element's centre; not converted"));
            continue;
        }
        if prog
            .loads
            .iter()
            .any(|l| on_chain(l.segment) && !matches!(l.kind, nec_kernel::load::LoadKind::Conductivity { .. }))
        {
            report.log.push(format!("{name}: carries a load; not converted"));
            continue;
        }
        let sections: Vec<(f64, f64)> = order.iter().map(|&i| (lens[i], radii[i])).collect();
        let (factor, radius, h) = leeson(&sections, w0);
        let resonance = C / (4.0 * h);
        let ratio = mhz * 1e6 / resonance;
        if !(0.85..=1.15).contains(&ratio) {
            report.log.push(format!(
                "{name}: {mhz} MHz is {:.0}% of the element's quarter-wave frequency {:.4} MHz, outside the method's ±15%; not converted",
                ratio * 100.0,
                resonance / 1e6
            ));
            continue;
        }
        // Rewrite the wire cards that built the element, which must lie wholly on it, through the
        // card-to-world scale (a GS card; a later GM move cannot be undone onto the card).
        let lines: Vec<usize> = segs.iter().map(|&k| geom.origins.get(k).copied().unwrap_or(0)).collect();
        let mut card_lines: Vec<usize> = lines.clone();
        card_lines.dedup();
        let mut element_changes = Vec::new();
        let mut usable = true;
        for &line in &card_lines {
            let Some(card) = gw.iter().find(|c| c.line == line) else {
                usable = false;
                break;
            };
            let v: Vec<f64> = (2..9).map(|i| card.real(i).unwrap_or(0.0)).collect();
            let own: Vec<usize> = segs.iter().copied().filter(|&k| geom.origins.get(k) == Some(&line)).collect();
            let total_own = geom.origins.iter().filter(|&&o| o == line).count();
            if own.len() != total_own {
                usable = false;
                break;
            }
            let (pa, pb) = (Point3::new(v[0], v[1], v[2]), Point3::new(v[3], v[4], v[5]));
            // The card is straight (it lies on the element), so its world length over its card
            // length is the GS scale; and its first end must then land on one of its ends.
            let card_len = pa.distance(pb);
            let world_len: f64 = own.iter().map(|&k| geom.segments[k].length()).sum();
            let s = if card_len > 0.0 { world_len / card_len } else { 1.0 };
            let scaled = Point3::new(pa.x * s, pa.y * s, pa.z * s);
            let (first, last) = (&geom.segments[own[0]], &geom.segments[own[own.len() - 1]]);
            let nearest = [first.a, first.b, last.a, last.b]
                .iter()
                .map(|p| p.distance(scaled))
                .fold(f64::INFINITY, f64::min);
            if nearest > 1e-6 * (1.0 + scaled.norm()) {
                usable = false;
                break;
            }
            let move_point = |p: Point3| {
                let w = Point3::new(p.x * s, p.y * s, p.z * s);
                let d = centre.to(w);
                Point3::new((centre.x + d.x / factor) / s, (centre.y + d.y / factor) / s, (centre.z + d.z / factor) / s)
            };
            let (na, nb) = (move_point(pa), move_point(pb));
            element_changes.push(CardChange { line, values: [na.x, na.y, na.z, nb.x, nb.y, nb.z, radius / s] });
        }
        if !usable {
            report.log.push(format!("{name}: its wire cards also build other wires, or are moved by a later card; not converted"));
            continue;
        }
        report.log.push(format!(
            "{name}: {} half-length {:.4} m → {:.4} m, uniform radius {:.5} m (stepped {:.5}–{:.5} m)",
            if dipole { "dipole" } else { "monopole" },
            h,
            h / factor,
            radius,
            radii.iter().copied().fold(f64::INFINITY, f64::min),
            radii.iter().copied().fold(0.0, f64::max)
        ));
        changes.extend(element_changes);
    }
    (changes, report)
}

/// Apply card changes to a resolved deck's `GW` cards.
pub fn apply(deck: &mut Deck, changes: &[CardChange]) {
    for it in &mut deck.items {
        let Item::Card(c) = it else { continue };
        if c.mnemonic != "GW" {
            continue;
        }
        let Some(ch) = changes.iter().find(|ch| ch.line == c.line) else { continue };
        for (k, &v) in ch.values.iter().enumerate() {
            if let Some(f) = c.fields.get_mut(2 + k) {
                f.value = Some(v);
                f.text = nec_cards::format_value(v);
                f.kind = nec_cards::FieldKind::Numeric;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_uniform_element_is_its_own_equivalent() {
        // Leeson's model reduces to the identity for a uniform element.
        let sections = vec![(0.5, 0.01); 10];
        let (factor, radius, h) = leeson(&sections, 1.0);
        assert!((h - 5.0).abs() < 1e-12);
        assert!((factor - 1.0).abs() < 1e-3, "{factor}");
        assert!((radius / 0.01 - 1.0).abs() < 5e-2, "{radius}");
    }

    /// 4nec2's own stepped-radius test deck (`models\HFsimple\StepRadAGT.nec`, two elements of
    /// eleven telescoping sections, in inches) and what `Convert.exe` makes of it (its `.cor`).
    const STEP_RAD_AGT: &str = "CE\nGW 1 8 -457 0 0 -372 0 0 0.25\nGW 2 5 -372 0 0 -306 0 0 0.375\nGW 3 5 -306 0 0 -240 0 0 0.5\nGW 4 10 -240 0 0 -120 0 0 1\nGW 5 10 -120 0 0 -12 0 0 1.125\nGW 6 3 -12 0 0 12 0 0 2.35\nGW 7 10 12 0 0 120 0 0 1.125\nGW 8 10 120 0 0 240 0 0 1\nGW 9 5 240 0 0 306 0 0 0.5\nGW 10 5 306 0 0 372 0 0 0.375\nGW 11 8 372 0 0 457 0 0 0.25\nGW 12 6 -397.5 234 0 -372 234 0 0.25\nGW 13 5 -372 234 0 -306 234 0 0.375\nGW 14 5 -306 234 0 -240 234 0 0.5\nGW 15 10 -240 234 0 -120 234 0 1\nGW 16 10 -120 234 0 -12 234 0 1.125\nGW 17 3 -12 234 0 12 234 0 2.35\nGW 18 10 12 234 0 120 234 0 1.125\nGW 19 10 120 234 0 240 234 0 1\nGW 20 5 240 234 0 306 234 0 0.5\nGW 21 5 306 234 0 372 234 0 0.375\nGW 22 6 372 234 0 397.5 234 0 0.25\nGS 0 0 0.0254\nGE 0\nLD 5 1 0 0 24938000 0\nLD 5 6 0 0 24938000 0\nLD 5 17 0 0 24938000 0\nEX 0 17 2 0 1 0\nGN -1\nFR 0 1 0 0 7.1 0\nEN\n";

    #[test]
    fn matches_convert_exe_on_a_monopole() {
        // A three-section vertical, fat end on perfect ground; Convert.exe's .cor values.
        let deck = "CE\nGW 1 5 0 0 0 0 0 2 0.02\nGW 2 5 0 0 2 0 0 4 0.015\nGW 3 4 0 0 4 0 0 5.2 0.01\nGE 1\nGN 1\nEX 0 1 1 0 1 0\nFR 0 1 0 0 14 0\nEN\n";
        let options = crate::ModelOptions { step_radius: true, ..Default::default() };
        let m = crate::Model::with_options(deck, &nec_cards::SymbolTable::new(), &crate::NgfStore::default(), &options)
            .expect("builds");
        let tops: Vec<f64> = m.steprad_changes.iter().map(|c| c.values[5]).collect();
        let expect = [1.932_823, 3.865_646, 5.025_340];
        assert_eq!(tops.len(), 3, "{:#?}", m.steprad_log);
        assert!(tops.iter().zip(expect).all(|(a, b)| (a - b).abs() < 1.5e-6), "{tops:?}");
        assert!(m.steprad_changes.iter().all(|c| (c.values[6] - 0.015_982).abs() < 1.5e-6 && c.values[0] == 0.0));
        assert_eq!(m.steprad_changes[0].values[2], 0.0, "the base stays on the ground");
    }

    #[test]
    fn matches_convert_exe_on_4nec2_s_test_deck() {
        let options = crate::ModelOptions { step_radius: true, ..Default::default() };
        let m = crate::Model::with_options(STEP_RAD_AGT, &nec_cards::SymbolTable::new(), &crate::NgfStore::default(), &options)
            .expect("builds");
        assert_eq!(m.steprad_changes.len(), 22, "{:#?}", m.steprad_log);
        let by_line = |line: usize| &m.steprad_changes.iter().find(|c| c.line == line).expect("changed").values;
        // Convert.exe prints six decimals.
        let close = |a: f64, b: f64| (a - b).abs() < 1.5e-6;
        let driven = by_line(2); // GW 1
        assert!(close(driven[0], -424.190_067) && close(driven[3], -345.292_571), "{driven:?}");
        assert!(close(driven[6], 0.685_695), "{driven:?}");
        let centre = by_line(7); // GW 6
        assert!(close(centre[0], -11.138_470) && close(centre[3], 11.138_470), "{centre:?}");
        let reflector = by_line(13); // GW 12
        assert!(close(reflector[0], -372.502_657) && close(reflector[3], -348.606_260), "{reflector:?}");
        assert!(close(reflector[1], 234.0) && close(reflector[6], 0.787_964), "{reflector:?}");
    }
}
