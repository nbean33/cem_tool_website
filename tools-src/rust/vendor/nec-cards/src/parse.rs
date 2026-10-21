//! Reading a card deck from text.
//!
//! The format tolerances here were derived from the 467-model corpus in `C:\4nec2\models`,
//! not from the specification alone, because real decks use more freedom than the spec
//! suggests:
//!
//! * fields separated by spaces, tabs **or** commas, in any mixture;
//! * trailing comments introduced by `'` on data cards;
//! * whole-line comments introduced by `'`, and blank lines, anywhere;
//! * lowercase mnemonics (`sy Did=...` appears alongside `SY`);
//! * numeric forms like `-.2418` and `1.33173e-3`;
//! * `CM`/`CE` carrying free text, where a `'` is part of the comment, not a delimiter.

use crate::deck::{Card, Deck, Field, FieldKind, Item};
use crate::error::{CardError, Result};

/// Parse a deck from source text. Line endings may be CRLF or LF.
///
/// A DOS end-of-file marker (`0x1A`, Ctrl-Z) truncates the input, as MS-DOS tools expected.
/// One corpus file (`VHFbeams\BIQDFEED.NEC`) ends with a bare `0x1A` line, which is not
/// Unicode whitespace and so would otherwise be parsed as a card with an empty mnemonic.
pub fn parse_deck(src: &str) -> Result<Deck> {
    let src = match src.find('\u{1a}') {
        Some(i) => &src[..i],
        None => src,
    };

    let mut items = Vec::new();

    for (idx, raw_line) in src.split('\n').enumerate() {
        let line_no = idx + 1;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // A trailing newline produces one final empty piece; don't record a phantom line.
            if idx + 1 == src.split('\n').count() && src.ends_with('\n') {
                continue;
            }
            items.push(Item::Blank);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('\'') {
            items.push(Item::LineComment(rest.to_string()));
            continue;
        }

        // Comment cards are recognized by their first two characters, before the general
        // mnemonic rule, because their text may abut the mnemonic with no separator:
        // the corpus contains `CMBISTATIC`, `CECYLINDER` and `CMPP`.
        if trimmed.len() >= 2 {
            let head = trimmed[..2].to_ascii_uppercase();
            if head == "CM" || head == "CE" {
                items.push(Item::CommentCard {
                    mnemonic: head,
                    text: trimmed[2..].to_string(),
                });
                continue;
            }
        }

        let (mnemonic, rest) = split_mnemonic(trimmed, line_no)?;

        match mnemonic.as_str() {
            // Comment cards carry free text; a quote inside is content, not a delimiter.
            "CM" | "CE" => items.push(Item::CommentCard {
                mnemonic,
                text: rest.to_string(),
            }),

            // 4nec2 symbolic variables.
            "SY" => {
                let (payload, comment) = split_trailing_comment(rest);
                items.push(Item::Sy {
                    raw: payload.trim().to_string(),
                    comment,
                    line: line_no,
                });
            }

            _ => {
                let (payload, comment) = split_trailing_comment(rest);
                let fields = tokenize_fields(payload, &mnemonic);
                items.push(Item::Card(Card {
                    mnemonic,
                    fields,
                    comment,
                    line: line_no,
                }));
            }
        }
    }

    Ok(Deck {
        items,
        symbols: crate::expr::SymbolTable::new(),
    })
}

/// Take the leading run of ASCII letters as the mnemonic, uppercased.
///
/// Stopping at the first non-letter also handles column-packed decks such as `GW1 9 ...`,
/// where the tag digit abuts the mnemonic.
fn split_mnemonic(line: &str, line_no: usize) -> Result<(String, &str)> {
    let end = line
        .char_indices()
        .find(|(_, c)| !c.is_ascii_alphabetic())
        .map_or(line.len(), |(i, _)| i);

    let mnemonic = &line[..end];
    if mnemonic.len() != 2 {
        return Err(CardError::BadMnemonic {
            line: line_no,
            mnemonic: mnemonic.to_string(),
        });
    }
    Ok((mnemonic.to_ascii_uppercase(), &line[end..]))
}

/// Split a data-card payload from its trailing comment.
///
/// Two delimiters occur in the corpus: `'` (4nec2's own convention) and `!` (Fortran style,
/// used by hand-written decks such as `Objects\747plane.nec`). Whichever appears first wins.
///
/// `#` is deliberately *not* a comment delimiter: `#12` in a radius field is AWG wire-gauge
/// data, not an annotation.
fn split_trailing_comment(rest: &str) -> (&str, Option<String>) {
    let cut = rest
        .char_indices()
        .find(|(_, c)| *c == '\'' || *c == '!')
        .map(|(i, _)| i);
    match cut {
        Some(i) => (&rest[..i], Some(rest[i + 1..].to_string())),
        None => (rest, None),
    }
}

/// The most fields a card can meaningfully take.
///
/// Used only to decide whether a space-separated unit name is a suffix or a value (see
/// [`tokenize_fields`]), so unknown cards fall back to NEC's format maximum: nine fields for
/// geometry cards (I1, I2, F1–F7) and ten for program-control cards (I1–I4, F1–F6). Erring
/// high only means fewer merges.
fn max_fields(mnemonic: &str) -> usize {
    match mnemonic {
        "GE" => 1,
        "GR" | "GX" => 2,
        "GS" => 3,
        "FR" => 6,
        "LD" => 7,
        "GW" | "GM" => 9,
        m if m.starts_with('G') || m.starts_with('S') => 9,
        _ => 10,
    }
}

/// Split a payload into fields, deciding for each bare unit name whether it is a suffix on the
/// previous number or a value in its own right.
///
/// The corpus uses both readings, separated only by spaces:
///
/// ```text
/// GS 0 0 ft                       ' ft IS the scale factor     (HFvertical\3vertical.nec)
/// LD 1 4 1 0 0 60.7 uh 0          ' uh is a suffix on 60.7     (Objects\Vehicle.nec)
/// GW 1 11 0 -68 ft X Y -68 ft Z #12   ' ft is a suffix (tab-delimited in the source)
/// ```
///
/// What disambiguates them is the card's field layout. `GS` takes three fields and
/// `GS 0 0 ft` already has three, so `ft` must be the value; `LD` takes seven and splitting
/// `60.7 uh` would give eight. So a unit preceded by spaces is merged as a suffix only when the
/// card would otherwise have more fields than it can take. A unit preceded by a tab or comma is
/// always its own field — 50 corpus lines read `GS<tab>0<tab>0<tab>ft`.
///
/// An earlier version always merged, which turned `GS 0 0 ft` into a scale factor of zero; the
/// reference engine then collapsed the structure onto the origin and reported a segment
/// connection error.
///
/// Residual ambiguity: a card that omits trailing fields *and* uses a space-separated suffix,
/// such as `GW 1 9 0 -68 ft 0 0 0` with no radius, has few enough tokens that the suffix is
/// read as a value. No corpus model does this.
fn tokenize_fields(payload: &str, mnemonic: &str) -> Vec<Field> {
    // Tokens, each remembering whether a tab or comma preceded it.
    let mut toks: Vec<(String, bool)> = Vec::new();
    let mut hard_sep = true; // the mnemonic boundary counts as a hard separator
    let mut cur = String::new();
    for c in payload.chars() {
        if c.is_whitespace() || c == ',' {
            if !cur.is_empty() {
                toks.push((std::mem::take(&mut cur), hard_sep));
                hard_sep = false;
            }
            if c == '\t' || c == ',' {
                hard_sep = true;
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        toks.push((cur, hard_sep));
    }

    let merge_suffixes = toks.len() > max_fields(mnemonic);

    let mut merged: Vec<String> = Vec::with_capacity(toks.len());
    for (tok, after_hard_sep) in toks {
        let is_suffix = merge_suffixes
            && !after_hard_sep
            && !merged.is_empty()
            && crate::expr::is_unit_name(&tok);
        if is_suffix {
            let last = merged.last_mut().expect("non-empty");
            last.push(' ');
            last.push_str(&tok);
        } else {
            merged.push(tok);
        }
    }

    merged
        .iter()
        .enumerate()
        .map(|(i, s)| {
            // A WG or GF field that does not start with 0 or 1 is the NGF file's name, wherever
            // it stands (`PARSIT`).
            let ngf_name = matches!(mnemonic, "WG" | "GF") && !s.starts_with(['0', '1']);
            let kind = if ngf_name || crate::deck::is_text_field(mnemonic, i) {
                FieldKind::Text
            } else {
                FieldKind::Numeric
            };
            Field::from_text_kind(s, kind)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_example1_shape() {
        // models/Example1.nec, with its tabs and trailing comments.
        let src = "CM Example 1 :\tDipole in free space \t' Comment cards\r\n\
                   CE \t\t\t\t\t\t\t' End of comment\r\n\
                   '\r\n\
                   GW 1 9 0 -.2418 0 0 .2418 0 .0001\t\t' Wire 1, 9 segments\r\n\
                   GE 0\t\t\t\t\t\t\t' End of geometry\r\n\
                   EX 0 1 5 0 1 0\t\t\t\t\t' Voltage source\r\n\
                   FR 0 1 0 0 300 0\t\t\t\t\t' 300 Mc\r\n\
                   EN\r\n";
        let deck = parse_deck(src).expect("parses");

        // CM/CE keep their text verbatim, quote included.
        match &deck.items[0] {
            Item::CommentCard { mnemonic, text } => {
                assert_eq!(mnemonic, "CM");
                assert!(text.contains("Dipole in free space"));
                assert!(text.contains('\''), "CM text keeps its quote as content");
            }
            other => panic!("expected CM card, got {other:?}"),
        }

        assert_eq!(deck.items[2], Item::LineComment(String::new()));

        let gw = deck.cards("GW").next().expect("a GW card");
        assert_eq!(gw.fields.len(), 9);
        assert_eq!(gw.int(0).expect("tag"), 1);
        assert_eq!(gw.int(1).expect("segs"), 9);
        assert!((gw.real(3).expect("y1") + 0.2418).abs() < 1e-12);
        assert!((gw.real(8).expect("radius") - 0.0001).abs() < 1e-15);
        assert_eq!(gw.comment.as_deref(), Some(" Wire 1, 9 segments"));

        assert!(!deck.uses_sy());
        assert!(!deck.has_symbolic_fields());
    }

    #[test]
    fn parses_tab_and_comma_separated_fields() {
        let deck = parse_deck("GW\t1\t11\t0\t-1\t0\t0\t1\t0\t1e-3\n").expect("parses");
        assert_eq!(deck.cards("GW").next().expect("gw").fields.len(), 9);

        let deck = parse_deck("FR 0,1,0,0,14.05,0\n").expect("parses");
        assert_eq!(deck.cards("FR").next().expect("fr").fields.len(), 6);
    }

    #[test]
    fn accepts_lowercase_mnemonics() {
        let deck = parse_deck("sy Did=1.5\n").expect("parses");
        assert!(matches!(&deck.items[0], Item::Sy { .. }));
    }

    #[test]
    fn resolves_symbolic_geometry_from_corpus_pattern() {
        // Equations/3elYagiGain.nec in miniature.
        let src = "SY Fr=14.05\n\
                   SY Dril=0.25, Hgh=0, Wr=1e-3\n\
                   GW\t1\t11\t0\t-Dril\tHgh\t0\tDril\tHgh\tWr\t'Driver\n\
                   FR 0 1 0 0 Fr 0\n\
                   EN\n";
        let mut deck = parse_deck(src).expect("parses");
        assert!(deck.uses_sy());
        assert!(deck.has_symbolic_fields());

        deck.resolve().expect("resolves");

        let gw = deck.cards("GW").next().expect("gw");
        assert!((gw.real(3).expect("y1") + 0.25).abs() < 1e-15);
        assert!((gw.real(6).expect("y2") - 0.25).abs() < 1e-15);
        assert!((gw.real(8).expect("radius") - 1e-3).abs() < 1e-18);

        let fr = deck.cards("FR").next().expect("fr");
        assert!((fr.real(4).expect("freq") - 14.05).abs() < 1e-12);

        assert_eq!(deck.symbols.get("fr"), Some(14.05));
    }

    #[test]
    fn int_function_in_segment_count_field() {
        // Corpus: GW 1 int(hgh) 0 0 0 0 0 hgh 1e-3
        let src = "SY hgh=7.9\nGW 1 int(hgh) 0 0 0 0 0 hgh 1e-3\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let gw = deck.cards("GW").next().expect("gw");
        assert_eq!(gw.int(1).expect("segment count"), 7);
    }

    #[test]
    fn sy_cards_are_sequential_so_later_may_use_earlier() {
        let src = "SY D=1000\nSY D1=LOG(D)/log(10)\nSY D2=D1^2\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        assert!((deck.symbols.get("d1").expect("d1") - 3.0).abs() < 1e-12);
        assert!((deck.symbols.get("d2").expect("d2") - 9.0).abs() < 1e-12);
    }

    #[test]
    fn card_with_no_fields() {
        let deck = parse_deck("EN\n").expect("parses");
        assert_eq!(deck.cards("EN").next().expect("en").fields.len(), 0);
    }

    #[test]
    fn rejects_a_line_that_is_not_a_card() {
        assert!(matches!(
            parse_deck("12345 oops\n"),
            Err(CardError::BadMnemonic { line: 1, .. })
        ));
    }

    #[test]
    fn dos_eof_marker_truncates_input() {
        // VHFbeams\BIQDFEED.NEC ends with a bare 0x1A line.
        let deck = parse_deck("GE 0\nEN\n\u{1a}\n").expect("parses");
        assert_eq!(deck.items.len(), 2);
    }

    #[test]
    fn comment_text_may_abut_the_mnemonic() {
        // SPatch\Nec-Ex9.nec has `CMBISTATIC ...`; VHFbeams\i5tdjii.nec has `CMPP`.
        let deck = parse_deck("CMBISTATIC SCATTERING\nCECYLINDER\nEN\n").expect("parses");
        assert_eq!(
            deck.items[0],
            Item::CommentCard {
                mnemonic: "CM".to_string(),
                text: "BISTATIC SCATTERING".to_string()
            }
        );
        assert_eq!(
            deck.items[1],
            Item::CommentCard {
                mnemonic: "CE".to_string(),
                text: "CYLINDER".to_string()
            }
        );
    }

    #[test]
    fn bang_is_also_a_comment_delimiter() {
        // Objects\747plane.nec uses Fortran-style `!` comments.
        let deck =
            parse_deck("GW 1 5 1102.48 -128.50 118.35 1102.00 -128.50 0.00 81.0E-3 ! Drive\n")
                .expect("parses");
        let gw = deck.cards("GW").next().expect("gw");
        assert_eq!(gw.fields.len(), 9);
        assert!((gw.real(8).expect("radius") - 81.0e-3).abs() < 1e-12);
        assert_eq!(gw.comment.as_deref(), Some(" Drive"));
    }

    #[test]
    fn unit_suffix_fields_merge_into_the_preceding_field() {
        // Corpus: GW 1 11 0 -68 ft X Y -68 ft Z #12
        let src = "SY X=1, Y=2, Z=3\nGW 1 11 0 -68 ft X Y -68 ft Z #12\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        let gw = deck.cards("GW").next().expect("gw");
        assert_eq!(gw.fields.len(), 9, "fields: {:?}", gw.fields);

        deck.resolve().expect("resolves");
        let gw = deck.cards("GW").next().expect("gw");
        assert!((gw.real(3).expect("y1") + 68.0 * 0.3048).abs() < 1e-12);
        // The radius came from the AWG shorthand: #12 is 1.026263e-3 m.
        assert!((gw.real(8).expect("radius") - 1.026_262_69e-3).abs() < 1e-9);
    }

    #[test]
    fn unit_as_the_value_of_a_card_with_room_for_it() {
        // HFvertical\3vertical.nec: `ft` is the scale factor, not a suffix on the zero.
        // Merging here once produced a scale factor of 0 and a collapsed structure.
        for src in ["GS 0 0 ft\n", "GS\t0\t0\tft\n"] {
            let mut deck = parse_deck(src).expect("parses");
            deck.resolve().expect("resolves");
            let gs = deck.cards("GS").next().expect("gs");
            assert_eq!(gs.fields.len(), 3, "{src:?} -> {:?}", gs.fields);
            assert!((gs.real(2).expect("scale") - 0.3048).abs() < 1e-12, "{src:?}");
        }
    }

    #[test]
    fn unit_as_a_suffix_when_the_card_would_overflow() {
        // Objects\Vehicle.nec: tab-delimited, with `60.7 uh` space-joined inside one cell.
        let mut deck = parse_deck("LD\t1\t4\t1\t0\t0\t60.7 uh\t0\n").expect("parses");
        deck.resolve().expect("resolves");
        let ld = deck.cards("LD").next().expect("ld");
        assert_eq!(ld.fields.len(), 7, "{:?}", ld.fields);
        assert!((ld.real(5).expect("L") - 60.7e-6).abs() < 1e-15);
        assert!(ld.real(6).expect("C").abs() < 1e-30);
    }

    #[test]
    fn unit_after_a_tab_is_never_a_suffix() {
        // Even on an overflowing card, a tab-separated unit stands alone.
        let deck = parse_deck("GW\t1\t2\t0\t0\t0\t0\t0\t1\tft\t1\n").expect("parses");
        let gw = deck.cards("GW").next().expect("gw");
        assert_eq!(gw.fields.len(), 10);
        assert_eq!(gw.fields[8].text, "ft");
    }

    #[test]
    fn column_packed_mnemonic_and_tag() {
        let deck = parse_deck("GW1 9 0 0 0 0 0 1 1e-3\n").expect("parses");
        let gw = deck.cards("GW").next().expect("gw");
        assert_eq!(gw.int(0).expect("tag"), 1);
        assert_eq!(gw.int(1).expect("segs"), 9);
    }
}
