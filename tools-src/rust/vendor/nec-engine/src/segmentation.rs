//! Changing how many segments wires have, and keeping everything that points at a segment
//! pointing at the same place along its wire.
//!
//! Sources (`EX`), loads (`LD`), lines and networks (`TL`, `NT`) name a segment by number, so
//! re-segmenting a wire moves them unless they are renumbered. [`resegment`] rewrites the
//! segment count of the chosen wire cards (`GW`, `GA`, `GH`) and renumbers every such
//! reference to the segment that now holds the same point: segment `k` of `N`, counting
//! from one, sits at fraction `(k − ½)/N` along its wire, and becomes segment `CInt(f·N′ + ½)` of
//! `N′` — the rounding 4nec2 uses for percentage positions (Visual Basic's `CInt`, half to
//! even). Fields written as percentages are left alone: they resolve against the new count
//! by themselves.
//!
//! The convergence test refines every wire this way; auto-segmentation does it to the wires
//! that are too coarse for the frequency.

use crate::{EngineError, Result};
use nec_cards::edit::{CardLine, cards, replace_card};
use nec_cards::{FieldKind, Item, SymbolTable, parse_deck};
use nec_geom::Geometry;
use std::collections::HashMap;

/// What happened to one wire card.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Change {
    /// Deck line of the card.
    pub line: usize,
    /// Wire tag.
    pub tag: i32,
    /// Segments before and after.
    pub from: usize,
    /// See `from`.
    pub to: usize,
}

/// A run of segments one wire card (or one copy of it) produced: consecutive segments with the
/// same originating card line and tag, each starting where the last ended.
#[derive(Debug, Clone, Copy)]
struct Run {
    origin: usize,
    tag: i32,
    start: usize,
    len: usize,
}

fn runs(geom: &Geometry) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::new();
    for (i, s) in geom.segments.iter().enumerate() {
        let origin = geom.origins.get(i).copied().unwrap_or(0);
        let continues = out.last().is_some_and(|r| {
            let prev = &geom.segments[r.start + r.len - 1];
            r.origin == origin && r.tag == s.tag && prev.b.distance(s.a) <= 1e-9 * (1.0 + prev.length())
        });
        if continues {
            if let Some(r) = out.last_mut() {
                r.len += 1;
            }
        } else {
            out.push(Run { origin, tag: s.tag, start: i, len: 1 });
        }
    }
    out
}

/// Visual Basic's `CInt`: round half to even.
fn cint(x: f64) -> i64 {
    #[allow(clippy::cast_possible_truncation)]
    let r = x.round_ties_even() as i64;
    r
}

fn build(text: &str, overrides: &SymbolTable) -> Result<Geometry> {
    let mut deck = parse_deck(text).map_err(|e| EngineError::Deck(e.to_string()))?;
    deck.resolve_with(overrides).map_err(|e| EngineError::Deck(e.to_string()))?;
    Geometry::build(&deck).map_err(|e| EngineError::Geometry(e.to_string()))
}

/// The segment-count field of a wire card, if it has one: `GW`, `GA` and `GH` hold it second.
fn count_field(c: &CardLine) -> Option<usize> {
    matches!(c.mnemonic.as_str(), "GW" | "GA" | "GH").then_some(1)
}

/// Whether a wire card's segment count is pinned against automatic changes: written with a
/// leading `+` (4nec2's rule).
#[must_use]
pub fn pinned(c: &CardLine) -> bool {
    count_field(c).and_then(|i| c.fields.get(i)).is_some_and(|f| f.trim_start().starts_with('+'))
}

/// The wire cards of `text` with their deck lines, tags, current segment counts and lengths in
/// metres (the length of what the card built, including any `GS` scaling).
///
/// # Errors
/// The deck does not parse or build.
pub fn wires(text: &str) -> Result<Vec<(CardLine, f64)>> {
    let geom = build(text, &SymbolTable::new())?;
    let lines = cards(text, &["GW", "GA", "GH"]).map_err(|e| EngineError::Deck(e.to_string()))?;
    let mut length: HashMap<usize, f64> = HashMap::new();
    for (i, s) in geom.segments.iter().enumerate() {
        let origin = geom.origins.get(i).copied().unwrap_or(0);
        // Only the card's own segments: copies made by GM/GR carry the copy card's line.
        *length.entry(origin).or_default() += s.length();
    }
    Ok(lines
        .into_iter()
        .map(|c| {
            let l = length.get(&c.line).copied().unwrap_or(0.0);
            (c, l)
        })
        .collect())
}

/// Give the wire cards in `counts` (deck line → new segment count) their new counts, and
/// renumber every `EX`/`LD`/`TL`/`NT` segment reference to follow. Returns the new text and the
/// changes made.
///
/// # Errors
/// The deck does not parse or build before or after the change.
pub fn resegment(text: &str, counts: &HashMap<usize, usize>) -> Result<(String, Vec<Change>)> {
    resegment_with(text, counts, &SymbolTable::new())
}

/// [`resegment`] with some `SY` variables pinned, as [`nec_cards::Deck::resolve_with`] pins
/// them.
///
/// # Errors
/// As [`resegment`].
pub fn resegment_with(
    text: &str,
    counts: &HashMap<usize, usize>,
    overrides: &SymbolTable,
) -> Result<(String, Vec<Change>)> {
    let old_geom = build(text, overrides)?;
    let all = cards(text, &[]).map_err(|e| EngineError::Deck(e.to_string()))?;

    // Rewrite the wire cards.
    let mut out = text.to_string();
    let mut changes = Vec::new();
    for c in &all {
        let (Some(i), Some(&n)) = (count_field(c), counts.get(&c.line)) else { continue };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let from = c.values.get(i).copied().flatten().unwrap_or(0.0).abs() as usize;
        if n == from || n == 0 {
            continue;
        }
        let mut fields = c.fields.clone();
        let keep_plus = fields[i].trim_start().starts_with('+');
        fields[i] = if keep_plus { format!("+{n}") } else { n.to_string() };
        out = replace_card(&out, c.line, &c.mnemonic, &fields).ok_or_else(|| EngineError::Deck("wire card".into()))?;
        #[allow(clippy::cast_possible_truncation)]
        let tag = c.values.first().copied().flatten().unwrap_or(0.0) as i32;
        changes.push(Change { line: c.line, tag, from, to: n });
    }
    if changes.is_empty() {
        return Ok((out, changes));
    }
    let new_geom = build(&out, overrides)?;
    let (old_runs, new_runs) = (runs(&old_geom), runs(&new_geom));

    // Where an old segment (0-based) lands in the new geometry, as (tag, number within the
    // tag, absolute number).
    let relocate = |old: usize| -> Option<(i32, usize, usize)> {
        let (ri, r) = old_runs.iter().enumerate().find(|(_, r)| old >= r.start && old < r.start + r.len)?;
        // The same run in the new geometry: the same ordinal among runs from the same card.
        let ordinal = old_runs[..ri].iter().filter(|q| q.origin == r.origin).count();
        let nr = new_runs.iter().filter(|q| q.origin == r.origin).nth(ordinal)?;
        let k = old - r.start;
        #[allow(clippy::cast_precision_loss)]
        let frac = (k as f64 + 0.5) / r.len as f64;
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap, clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let k_new = (cint(frac * nr.len as f64 + 0.5).clamp(1, nr.len as i64) - 1) as usize;
        let abs = nr.start + k_new;
        let tag = new_geom.segments[abs].tag;
        let within = new_geom.segments[..=abs].iter().filter(|s| s.tag == tag).count();
        Some((tag, within, abs + 1))
    };

    // Renumber references. Field pairs (tag, segment) per card.
    let pairs: &[(&str, &[(usize, usize)])] = &[
        ("EX", &[(1, 2)]),
        ("LD", &[(1, 2), (1, 3)]),
        ("TL", &[(0, 1), (2, 3)]),
        ("NT", &[(0, 1), (2, 3)]),
    ];
    let deck = parse_deck(&out).map_err(|e| EngineError::Deck(e.to_string()))?;
    let kinds: HashMap<usize, Vec<FieldKind>> = deck
        .items
        .iter()
        .filter_map(|it| match it {
            Item::Card(c) => Some((c.line, c.fields.iter().map(|f| f.kind).collect())),
            _ => None,
        })
        .collect();
    let after = cards(&out, &[]).map_err(|e| EngineError::Deck(e.to_string()))?;
    for c in &after {
        let Some((_, fields_of)) = pairs.iter().find(|(m, _)| *m == c.mnemonic) else { continue };
        // EX types 1–4 (incident fields) and LD with no segment range name no segment.
        #[allow(clippy::cast_possible_truncation)]
        if c.mnemonic == "EX" && !matches!(c.values.first().copied().flatten().map(|v| v.round() as i64), Some(0 | 5 | 6)) {
            continue;
        }
        let mut fields = c.fields.clone();
        let mut changed = false;
        for &(ti, si) in *fields_of {
            if si >= fields.len() {
                continue;
            }
            if kinds.get(&c.line).and_then(|k| k.get(si)) == Some(&FieldKind::Percent) {
                continue;
            }
            #[allow(clippy::cast_possible_truncation)]
            let tag = c.values.get(ti).copied().flatten().unwrap_or(0.0) as i32;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let seg = c.values.get(si).copied().flatten().unwrap_or(0.0) as usize;
            // LD with a zero segment range means "the whole tag" (or everything): no number.
            if seg == 0 {
                continue;
            }
            let Some(old) = old_geom.resolve_segment(tag, seg).map(|s| s.index - 1) else { continue };
            let Some((_, within, abs)) = relocate(old) else { continue };
            let new = if tag == 0 { abs } else { within };
            if new != seg {
                fields[si] = new.to_string();
                changed = true;
            }
        }
        if changed {
            out = replace_card(&out, c.line, &c.mnemonic, &fields).ok_or_else(|| EngineError::Deck("reference card".into()))?;
        }
    }
    Ok((out, changes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECK: &str = "CE\nGW 1 11 0 -5 10 0 5 10 .001\nGW 2 +9 1 -5 10 1 5 10 .001\nGE 0\nEX 0 1 6 0 1 0\nLD 0 2 3 3 50 0 0\nTL 1 6 2 5 50 0\nFR 0 1 0 0 14 0\nEN\n";

    fn line_of(text: &str, m: &str) -> String {
        text.lines().find(|l| l.starts_with(m)).unwrap_or_default().to_string()
    }

    #[test]
    fn references_follow_their_point_along_the_wire() {
        let counts: HashMap<usize, usize> = [(2, 21), (3, 27)].into_iter().collect();
        let (out, changes) = resegment(DECK, &counts).expect("resegments");
        assert_eq!(changes.len(), 2);
        assert!(out.contains("GW 1 21 0 -5"), "{out}");
        // The + survives, so the wire stays pinned for auto-segmentation.
        assert!(out.contains("GW 2 +27 1 -5"), "{out}");
        // Centre of 11 is the centre of 21; segment 3 of 9 (fraction 5/18) is segment 8 of 27;
        // the TL's 5 of 9 (the centre) is 14 of 27.
        assert_eq!(line_of(&out, "EX"), "EX 0 1 11 0 1 0");
        assert_eq!(line_of(&out, "LD"), "LD 0 2 8 8 50 0 0");
        assert_eq!(line_of(&out, "TL"), "TL 1 11 2 14 50 0");
    }

    #[test]
    fn percent_positions_are_left_to_resolve() {
        let deck = DECK.replace("EX 0 1 6 0 1 0", "EX 0 1 50% 0 1 0");
        let counts: HashMap<usize, usize> = [(2, 21)].into_iter().collect();
        let (out, _) = resegment(&deck, &counts).expect("resegments");
        assert_eq!(line_of(&out, "EX"), "EX 0 1 50% 0 1 0");
    }

    #[test]
    fn wires_report_lengths_and_pins() {
        let w = wires(DECK).expect("wires");
        assert_eq!(w.len(), 2);
        assert!((w[0].1 - 10.0).abs() < 1e-9);
        assert!(!pinned(&w[0].0));
        assert!(pinned(&w[1].0));
    }
}
