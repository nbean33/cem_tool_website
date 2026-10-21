//! Emitting card decks.
//!
//! Two output forms, for two different jobs:
//!
//! * [`to_authoring_text`] rebuilds the deck as written, symbols and comments intact. Used by
//!   the round-trip property test and, later, by the GUI when saving a project.
//! * [`to_engine_input`] emits the purely numeric deck the reference engine consumes. This is
//!   what makes differential testing of parametric models possible at all, since the engine
//!   cannot read `SY` cards.
//!
//! The engine form was reverse-engineered by comparing `models\Example1.nec` (authoring) with
//! `out\EXAMPLE1.inp` (what 4nec2 actually handed the engine). The transformation is:
//! strip trailing `'` comments from data cards, keep `CM`/`CE` text verbatim, drop blank and
//! comment-only lines and all `SY` cards, insert an execute card if none is present, and use
//! CRLF endings.

use crate::deck::{Deck, FieldKind, Item};
use crate::error::Result;

/// Cards that cause NEC to run a calculation. If a deck contains none of these, the engine
/// reads the geometry and exits without producing results, so one must be supplied.
const EXECUTE_CARDS: [&str; 4] = ["XQ", "RP", "NE", "NH"];

/// Rebuild the deck in its authoring form, preserving symbols, comments and blank lines.
///
/// Field separators are normalized to single spaces, so this is a *semantic* round trip
/// rather than a byte-identical one: re-parsing the result yields an equal deck.
#[must_use]
pub fn to_authoring_text(deck: &Deck) -> String {
    let mut out = String::new();
    for item in &deck.items {
        match item {
            Item::Blank => {}
            Item::LineComment(text) => {
                out.push('\'');
                out.push_str(text);
            }
            Item::CommentCard { mnemonic, text } => {
                out.push_str(mnemonic);
                out.push_str(text);
            }
            Item::Sy { raw, comment, .. } => {
                out.push_str("SY ");
                out.push_str(raw);
                if let Some(c) = comment {
                    out.push('\'');
                    out.push_str(c);
                }
            }
            Item::Card(card) => {
                out.push_str(&card.mnemonic);
                for f in &card.fields {
                    out.push(' ');
                    out.push_str(&f.text);
                }
                if let Some(c) = &card.comment {
                    out.push('\'');
                    out.push_str(c);
                }
            }
        }
        out.push('\n');
    }
    out
}

/// Emit the numeric deck for the reference engine.
///
/// Requires that [`Deck::resolve`](crate::deck::Deck::resolve) has already run; a still-symbolic
/// field is reported as an error rather than silently dropped. 4nec2's extension cards (`LD 6`,
/// `LD 7`, `EX 6`) are lowered to standard NEC-2 first, as 4nec2 does; see [`crate::lower`].
///
/// A deck of several runs (`NX`) is emitted run by run, each with its own comment block, joined
/// by `NX`. A run that writes a numerical Green's function (`WG`) gets no execute card: `WG` itself
/// has NEC fill and factor the matrix.
pub fn to_engine_input(deck: &Deck) -> Result<String> {
    let lowered = crate::lower::lower_4nec2(deck)?;
    let runs = lowered.runs();
    let mut lines: Vec<String> = Vec::new();
    for (i, run) in runs.iter().enumerate() {
        if i > 0 {
            lines.push("NX".to_string());
        }
        lines.extend(run_lines(run, i + 1 == runs.len())?);
    }
    Ok(join_crlf(&lines))
}

fn run_lines(deck: &Deck, last: bool) -> Result<Vec<String>> {
    let mut lines = comment_block(deck);
    let mut has_execute = deck.cards("WG").next().is_some();
    let mut saw_en = false;

    for item in &deck.items {
        match item {
            // Not meaningful to the engine; comment cards were hoisted into `comment_block`.
            Item::Blank | Item::LineComment(_) | Item::Sy { .. } | Item::CommentCard { .. } => {}

            Item::Card(card) => {
                if card.mnemonic == "EN" {
                    saw_en = true;
                    continue; // re-emitted last, after any inserted execute card
                }
                if EXECUTE_CARDS.contains(&card.mnemonic.as_str()) {
                    has_execute = true;
                }

                lines.push(format_card(card)?);
            }
        }
    }

    if !has_execute {
        lines.push("XQ".to_string());
    }
    if saw_en && last {
        lines.push("EN".to_string());
    }
    Ok(lines)
}

/// Emit only the geometry section — comment cards and every card up to and including `GE` —
/// followed by `EN`.
///
/// Given this, the engine prints its `SEGMENTATION DATA` table and exits without reading a
/// frequency or solving anything. That matters for validating geometry: with the full deck the
/// engine runs a complete O(N³) solve per model, and one corpus model (`HFbeams\Bowtie.nec`)
/// needs the 8000-segment build and minutes of CPU just to print a table that appears before
/// any of that work begins. Verified by running the engine on a geometry-only deck.
pub fn to_geometry_input(deck: &Deck) -> Result<String> {
    let mut lines = comment_block(deck);
    for item in &deck.items {
        match item {
            Item::Blank | Item::LineComment(_) | Item::Sy { .. } | Item::CommentCard { .. } => {}
            Item::Card(card) => {
                lines.push(format_card(card)?);
                if card.mnemonic == "GE" {
                    break;
                }
            }
        }
    }
    lines.push("EN".to_string());
    Ok(join_crlf(&lines))
}

/// Every comment card in the deck, hoisted to the top, ending in exactly one `CE`.
///
/// NEC reads comment cards only until the first `CE`; whatever follows is parsed as data. Decks
/// in the wild do not respect that — `HFbeams\5_el_wbeam_PA2AWU.nec` has 27 `CM`, a `CE`, three
/// more `CM`, and a second `CE`, and the engine rejects the fourth-from-last `CM` with
/// `CARD ERROR - TOO MANY FIELDS IN RECORD`. So all comment cards are gathered in order, every
/// `CE` but the last becomes a `CM`, and the block always ends in `CE`.
///
/// The block is never empty. With no comment cards at all, the engine silently consumed the
/// first `GW` as if it were a comment and produced an empty segmentation table — verified by
/// running it on a deck with and without a bare `CE`.
fn comment_block(deck: &Deck) -> Vec<String> {
    let texts: Vec<&str> = deck
        .items
        .iter()
        .filter_map(|it| match it {
            Item::CommentCard { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();

    if texts.is_empty() {
        return vec!["CE".to_string()];
    }
    let last = texts.len() - 1;
    texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let mnemonic = if i == last { "CE" } else { "CM" };
            format!("{mnemonic}{text}")
        })
        .collect()
}

/// One card as a numeric engine-input line.
fn format_card(card: &crate::deck::Card) -> Result<String> {
    let mut s = card.mnemonic.clone();
    for (i, f) in card.fields.iter().enumerate() {
        s.push(' ');
        if f.kind == FieldKind::Text {
            // e.g. the NGF filename on `GF 0 RADIAL8.NGF`.
            s.push_str(&f.text);
        } else {
            let v = f.require(&card.mnemonic, card.line)?;
            s.push_str(&fmt_field(v, i, &card.mnemonic, None));
        }
    }
    if s.len() <= CARD_COLUMNS {
        return Ok(s);
    }

    // Too wide for the engine's card image: shed precision until it fits.
    for sig in (MIN_SIGNIFICANT..=MAX_SIGNIFICANT).rev() {
        let mut t = card.mnemonic.clone();
        for (i, f) in card.fields.iter().enumerate() {
            t.push(' ');
            if f.kind == FieldKind::Text {
                t.push_str(&f.text);
            } else {
                let v = f.require(&card.mnemonic, card.line)?;
                t.push_str(&fmt_field(v, i, &card.mnemonic, Some(sig)));
            }
        }
        if t.len() <= CARD_COLUMNS {
            return Ok(t);
        }
    }
    Err(crate::error::CardError::CardTooWide {
        line: card.line,
        mnemonic: card.mnemonic.clone(),
    })
}

/// The engine reads 80-column card images and silently discards anything past column 80.
///
/// Punched-card heritage, measured directly: a `GW` card whose radius field ends in column 80
/// is read correctly; the same card shifted one column right reads the radius as zero, and
/// the engine then reports `GEOMETRY DATA CARD ERROR`. Symbol resolution produces full f64
/// precision (`58.82608298685654`), which is how 18 corpus models overflowed before this limit
/// was enforced.
pub const CARD_COLUMNS: usize = 80;

/// Widest number the engine can read in one field.
///
/// `PARSIT` copies each free-format field into `CHARACTER BUFFER*20` before reading it, so a
/// longer number is cut short and rejected (`CARD ERROR - INVALID NUMBER AT REAL POSITION`). A
/// real with no decimal point gets one appended first, which leaves 19 characters. Measured:
/// `1.5308084989341915e-15` (22 characters, from `cos(90)` in `HFverticalGndScreen.nec`) stops
/// the engine.
pub const FIELD_CHARS: usize = 20;

/// Most significant digits ever emitted: the most an f64 can meaningfully carry.
const MAX_SIGNIFICANT: usize = 15;

/// Fewest significant digits used to squeeze a card into 80 columns. Seven digits is about
/// 1e-7 relative, far below anything NEC's single-segment numerics resolve.
const MIN_SIGNIFICANT: usize = 7;

fn join_crlf(lines: &[String]) -> String {
    let mut out = lines.join("\r\n");
    out.push_str("\r\n");
    out
}

/// [`fmt_num`], shedding precision if the number is wider than the engine reads ([`FIELD_CHARS`]).
fn fmt_field(v: f64, index: usize, mnemonic: &str, sig: Option<usize>) -> String {
    let fits = |s: &str| s.len() <= FIELD_CHARS && (s.contains('.') || s.len() < FIELD_CHARS);
    let s = fmt_num(v, index, mnemonic, sig);
    if fits(&s) {
        return s;
    }
    let top = sig.unwrap_or(MAX_SIGNIFICANT).min(MAX_SIGNIFICANT);
    let mut last = s;
    for sig in (MIN_SIGNIFICANT..=top).rev() {
        last = fmt_num(v, index, mnemonic, Some(sig));
        if fits(&last) {
            break;
        }
    }
    last
}

/// Format a field value for the engine.
///
/// The leading fields of NEC cards are integer codes and tags; emitting those as `0` rather
/// than `0.0000` keeps the deck looking like the hand-written ones the engine was built for.
///
/// With `sig == None` the value is printed at full round-trip precision. With
/// `Some(n)` it is rounded to `n` significant digits and printed in whichever of fixed or
/// exponent notation is shorter — used only when a card would otherwise overflow 80 columns.
fn fmt_num(v: f64, index: usize, mnemonic: &str, sig: Option<usize>) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    if !v.is_finite() {
        // Let it through as text rather than silently substituting; the engine will complain
        // loudly, which is more useful than a plausible wrong number.
        return format!("{v}");
    }

    // Integer-coded leading fields are always emitted as integers, truncated toward zero.
    //
    // The engine rejects a fraction there outright (`CARD ERROR - INVALID NUMBER AT INTEGER
    // POSITION 2`), and symbol resolution produces fractions: `Objects\TL-test.nec` computes its
    // segment count as 94.9178…. Truncation matches `Card::int`, which the geometry builder
    // uses, and the `SY` `int()` function. Whether 4nec2 truncates or rounds here is unverified
    // (see KNOWN-UNKNOWNS.md); the two differ for this one model, 94 versus 95 segments.
    let int_fields = integer_field_count(mnemonic);
    if index < int_fields {
        #[allow(clippy::cast_possible_truncation)]
        return format!("{}", v.trunc() as i64);
    }

    let Some(sig) = sig else {
        // Outside this range decimal notation becomes unreadable (or enormous); the corpus
        // itself writes such values in exponent form, e.g. `1.33173e-3`.
        let mag = v.abs();
        return if (1e-4..1e10).contains(&mag) {
            format!("{v}")
        } else {
            format!("{v:e}")
        };
    };

    // Exponent form at `sig` significant digits, with trailing mantissa zeros removed.
    let exp_form = {
        let s = format!("{:.*e}", sig.saturating_sub(1), v);
        match s.split_once('e') {
            Some((mant, exp)) => {
                let mant = if mant.contains('.') {
                    mant.trim_end_matches('0').trim_end_matches('.')
                } else {
                    mant
                };
                format!("{mant}e{exp}")
            }
            None => s,
        }
    };

    // Fixed form at the same number of significant digits.
    #[allow(clippy::cast_possible_truncation)]
    let magnitude = v.abs().log10().floor() as i64;
    #[allow(clippy::cast_possible_wrap)]
    let decimals = (sig as i64 - 1 - magnitude).max(0);
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let fixed_form = {
        let s = format!("{:.*}", decimals as usize, v);
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    };

    if fixed_form.len() <= exp_form.len() {
        fixed_form
    } else {
        exp_form
    }
}

/// How many leading fields of a card are integer codes rather than reals.
///
/// Only cards whose layout matters for emission are listed; anything unlisted is treated as
/// all-real, which still round-trips correctly because integral f64 values print without a
/// decimal point.
fn integer_field_count(mnemonic: &str) -> usize {
    match mnemonic {
        "GW" => 2,  // tag, segment count
        "GE" => 1,  // ground flag
        "GS" => 2,  // I1, I2 (then the scale factor)
        "GM" => 2,  // tag increment, repeat count
        "GX" => 2,  // tag increment, symmetry flags
        "GR" => 2,  // tag increment, repeat count
        "EX" => 4,  // type, tag, segment, flags
        "FR" => 4,  // type, count, unused, unused
        "GN" => 4,  // type, radial count, unused, unused
        "LD" => 4,  // type, tag, start segment, end segment
        "RP" => 4,  // mode, theta count, phi count, XNDA digits
        "NE" | "NH" => 4,
        "TL" | "NT" => 4,
        "PT" | "PQ" => 4,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_deck;

    const EXAMPLE1: &str = "CM Example 1 :\tDipole in free space \t' Comment cards\r\n\
                            CM See GetStarted.txt\r\n\
                            CE \t\t\t\t\t\t\t' End of comment\r\n\
                            '\r\n\
                            GW 1 9 0 -.2418 0 0 .2418 0 .0001\t\t' Wire 1\r\n\
                            GE 0\t\t\t\t\t\t\t' End of geometry\r\n\
                            '\r\n\
                            EX 0 1 5 0 1 0\t\t\t\t\t' Voltage source\r\n\
                            FR 0 1 0 0 300 0\t\t\t\t\t' 300 Mc\r\n\
                            EN\r\n";

    #[test]
    fn authoring_form_round_trips_semantically() {
        let first = parse_deck(EXAMPLE1).expect("parses");
        let text = to_authoring_text(&first);
        let second = parse_deck(&text).expect("reparses");
        assert_eq!(
            first.items, second.items,
            "re-parsing the emitted authoring form must yield the same deck"
        );
    }

    /// Compare every numeric field of two decks, so formatting choices (`.0001` vs `0.0001`)
    /// don't make tests brittle. The engine reads free-form reals, so only values matter.
    fn assert_same_numbers(a: &Deck, b: &Deck) {
        let nums = |d: &Deck| -> Vec<(String, Vec<f64>)> {
            d.items
                .iter()
                .filter_map(|it| match it {
                    Item::Card(c) => Some((
                        c.mnemonic.clone(),
                        c.fields.iter().map(|f| f.value.expect("resolved")).collect(),
                    )),
                    _ => None,
                })
                .collect()
        };
        let (va, vb) = (nums(a), nums(b));
        for (mnemonic, want) in &va {
            let got = vb
                .iter()
                .find(|(m, _)| m == mnemonic)
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("{mnemonic} card missing after round trip"));
            assert_eq!(want.len(), got.len(), "{mnemonic} field count changed");
            for (i, (w, g)) in want.iter().zip(got).enumerate() {
                assert!(
                    (w - g).abs() <= w.abs() * 1e-15,
                    "{mnemonic} field {i}: {w} became {g}"
                );
            }
        }
    }

    #[test]
    fn engine_input_applies_the_transformation_4nec2_applies() {
        // Checked against the real pair: models\Example1.nec -> out\EXAMPLE1.inp.
        let mut deck = parse_deck(EXAMPLE1).expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_engine_input(&deck).expect("emits");
        let lines: Vec<&str> = inp.trim_end().split("\r\n").collect();

        // Comment cards pass through verbatim, quote and tabs included.
        assert_eq!(lines[0], "CM Example 1 :\tDipole in free space \t' Comment cards");
        assert_eq!(lines[1], "CM See GetStarted.txt");
        assert_eq!(lines[2], "CE \t\t\t\t\t\t\t' End of comment");

        // Data cards keep their order and lose their trailing comments.
        let mnemonics: Vec<&str> = lines[3..]
            .iter()
            .map(|l| l.split_whitespace().next().unwrap_or(""))
            .collect();
        assert_eq!(mnemonics, vec!["GW", "GE", "EX", "FR", "XQ", "EN"]);
        assert!(
            lines[3..].iter().all(|l| !l.contains('\'')),
            "data cards must not carry comments: {lines:?}"
        );
        assert_eq!(lines.len(), 9, "blank and comment-only lines are dropped");

        // And the numbers survive exactly.
        let mut back = parse_deck(&inp).expect("engine input re-parses");
        back.resolve().expect("resolves");
        assert_same_numbers(&deck, &back);
    }

    #[test]
    fn existing_execute_card_is_not_duplicated() {
        let mut deck = parse_deck("GE 0\nRP 0 1 1 1000 0 0 0 0\nEN\n").expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_engine_input(&deck).expect("emits");
        assert_eq!(inp.matches("XQ").count(), 0);
        assert!(inp.contains("RP 0 1 1 1000 0 0 0 0"));
    }

    #[test]
    fn en_is_emitted_last_even_after_inserting_execute() {
        let mut deck = parse_deck("GE 0\nEN\n").expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_engine_input(&deck).expect("emits");
        let lines: Vec<&str> = inp.trim_end().split("\r\n").collect();
        // The leading CE is required: the engine treats the first card as a comment otherwise.
        assert_eq!(lines, vec!["CE", "GE 0", "XQ", "EN"]);
    }

    #[test]
    fn symbolic_deck_emits_numeric_engine_input() {
        let src = "SY Dril=0.25, Hgh=0, Wr=1e-3\n\
                   GW 1 11 0 -Dril Hgh 0 Dril Hgh Wr\n\
                   GE 0\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_engine_input(&deck).expect("emits");
        assert!(!inp.contains("SY"), "SY cards must not reach the engine");

        // The emitted deck must be fully numeric and carry the resolved values.
        let mut back = parse_deck(&inp).expect("engine input re-parses");
        assert!(
            !back.has_symbolic_fields(),
            "engine input still contains symbols: {inp}"
        );
        back.resolve().expect("resolves");
        let gw = back.cards("GW").next().expect("gw");
        assert!((gw.real(3).expect("y1") + 0.25).abs() < 1e-15);
        assert!((gw.real(6).expect("y2") - 0.25).abs() < 1e-15);
        assert!((gw.real(8).expect("radius") - 1e-3).abs() < 1e-18);
    }

    #[test]
    fn geometry_input_stops_at_ge_and_never_executes() {
        let mut deck = parse_deck(EXAMPLE1).expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_geometry_input(&deck).expect("emits");
        let mnemonics: Vec<&str> = inp
            .trim_end()
            .split("\r\n")
            .map(|l| l.split_whitespace().next().unwrap_or(""))
            .collect();
        assert_eq!(mnemonics, vec!["CM", "CM", "CE", "GW", "GE", "EN"]);
        // No frequency, no excitation, no execute card: nothing for the engine to solve.
        for absent in ["FR", "EX", "XQ"] {
            assert!(!mnemonics.contains(&absent), "{absent} must not be emitted");
        }
    }

    #[test]
    fn comment_cards_are_hoisted_into_one_block_ending_in_ce() {
        // The shape of HFbeams\5_el_wbeam_PA2AWU.nec: CM, CE, CM, CE, then geometry, and a
        // stray CM after the geometry for good measure.
        let src = "CM one\nCE two\nCM three\nCE four\nGW 1 1 0 0 0 1 0 0 1e-3\nCM five\nGE 0\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        for inp in [
            to_engine_input(&deck).expect("emits"),
            to_geometry_input(&deck).expect("emits"),
        ] {
            let lines: Vec<&str> = inp.trim_end().split("\r\n").collect();
            assert_eq!(&lines[..5], &["CM one", "CM two", "CM three", "CM four", "CE five"]);
            assert_eq!(lines[5].split_whitespace().next(), Some("GW"));
            assert_eq!(inp.matches("\r\nCE").count() + usize::from(inp.starts_with("CE")), 1);
        }
    }

    #[test]
    fn a_deck_without_comments_still_gets_a_ce() {
        // Without one, the engine consumed the first GW as a comment.
        let mut deck = parse_deck("GW 1 1 0 0 0 1 0 0 1e-3\nGE 0\nEN\n").expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_geometry_input(&deck).expect("emits");
        assert!(inp.starts_with("CE\r\nGW "), "{inp}");
    }

    #[test]
    fn fractional_integer_fields_are_truncated() {
        // Objects\TL-test.nec resolves its segment count to 94.9178...; the engine needs 94.
        let src = "SY N=94.9178240038814\nGW 1 N 0 0 0 0 0 21 9.7e-4\nGE 1\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_geometry_input(&deck).expect("emits");
        assert!(inp.contains("\r\nGW 1 94 0 0 0 0 0 21 "), "{inp}");
    }

    #[test]
    fn resolved_cards_are_squeezed_into_80_columns() {
        // Aperiodic\Slopvee.nec after symbol resolution: 104 columns at full precision, so the
        // engine dropped the radius and reported GEOMETRY DATA CARD ERROR.
        let src = "SY X=58.82608298685654, Y=-105.80962435538991\n\
                   GW 11 3 X Y 0 X Y 1.829 0.001\nGE 1\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_geometry_input(&deck).expect("emits");
        for line in inp.lines() {
            assert!(line.len() <= CARD_COLUMNS, "{} columns: {line}", line.len());
        }

        // Precision is shed only as far as needed, and every field survives.
        let mut back = parse_deck(&inp).expect("re-parses");
        back.resolve().expect("resolves");
        let gw = back.cards("GW").next().expect("gw");
        assert_eq!(gw.fields.len(), 9);
        assert!((gw.real(2).expect("x1") - 58.826_082_986_856_54).abs() < 1e-5);
        assert!((gw.real(8).expect("radius") - 0.001).abs() < 1e-15, "radius must survive");
    }

    #[test]
    fn no_number_is_wider_than_the_engine_reads() {
        // HFvertical\GndScreen.nec: radl*cos(4*ra) with ra = 22.5 degrees is 25*cos(90°),
        // 1.5308084989341915e-15 at full precision — 22 characters, which the engine rejects.
        let mut deck = parse_deck("SY ra=360/16
GW 14 25 0 0 0.0018 25*cos(4*ra) 25*sin(4*ra) 0.0018 6e-4
GE 0
EN
")
            .expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_geometry_input(&deck).expect("emits");
        let gw = inp.lines().find(|l| l.starts_with("GW")).expect("gw");
        for f in gw.split_whitespace().skip(1) {
            assert!(f.len() <= FIELD_CHARS, "{f:?} in {gw}");
            assert!(f.contains('.') || f.len() < FIELD_CHARS, "{f:?} leaves no room for a point");
        }
        let mut back = parse_deck(&inp).expect("re-parses");
        back.resolve().expect("resolves");
        let x2 = back.cards("GW").next().expect("gw").real(5).expect("x2");
        assert!(x2.abs() < 1e-14, "{x2}");
    }

    #[test]
    fn short_cards_keep_full_precision() {
        let mut deck = parse_deck("SY A=0.1234567890123\nGW 1 1 0 0 0 A 0 0 1e-3\nGE 0\nEN\n")
            .expect("parses");
        deck.resolve().expect("resolves");
        let inp = to_geometry_input(&deck).expect("emits");
        assert!(inp.contains("0.1234567890123"), "{inp}");
    }

    #[test]
    fn reduced_precision_picks_the_shorter_notation() {
        assert_eq!(fmt_num(58.826_082_986_856_54, 5, "GW", Some(7)), "58.82608");
        assert_eq!(fmt_num(-105.809_624_355_389_91, 5, "GW", Some(7)), "-105.8096");
        // Fixed and exponent forms are both 11 characters here; ties go to fixed.
        assert_eq!(fmt_num(1.026_262_69e-3, 8, "GW", Some(7)), "0.001026263");
        // Exponent form wins once it is genuinely shorter.
        assert_eq!(fmt_num(1.026_262_69e-9, 8, "GW", Some(7)), "1.026263e-9");
        assert_eq!(fmt_num(0.001, 8, "GW", Some(7)), "1e-3"); // 4 chars beats 5
        // Integer fields are untouched by the precision budget.
        assert_eq!(fmt_num(11.0, 0, "GW", Some(7)), "11");
    }

    #[test]
    fn unresolved_field_is_an_error_not_a_silent_drop() {
        let deck = parse_deck("GW 1 11 0 -Dril 0 0 1 0 1e-3\nEN\n").expect("parses");
        // deliberately not resolved
        assert!(to_engine_input(&deck).is_err());
    }
}
