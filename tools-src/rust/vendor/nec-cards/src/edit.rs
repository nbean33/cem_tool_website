//! Editing a deck's text one card at a time, for an interactive editor.
//!
//! The text stays the model: an edit rewrites, inserts or removes one line and leaves every
//! other byte alone, so comments, layout and `SY` formulas elsewhere survive any number of
//! edits. Lines are numbered from 1 as [`parse_deck`] numbers them (the `line` of a
//! [`Card`](crate::Card)), which is also what [`nec_geom`'s segment origins] refer to.
//!
//! [`nec_geom`'s segment origins]: https://docs.rs/nec-geom

use crate::deck::Item;
use crate::error::Result;
use crate::parse::parse_deck;

/// One data card as an editor shows it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CardLine {
    /// Line number in the deck text, from 1.
    pub line: usize,
    /// The card's mnemonic, uppercased.
    pub mnemonic: String,
    /// Each field exactly as written: a number, or an expression over `SY` variables.
    pub fields: Vec<String>,
    /// Each field's value once the deck's variables are applied; `None` for a text field, or
    /// when the deck does not resolve.
    pub values: Vec<Option<f64>>,
    /// The trailing comment, without its delimiter.
    pub comment: Option<String>,
}

/// The data cards of `src` whose mnemonic is one of `mnemonics` (all of them when empty), in
/// order, with their fields' values.
///
/// # Errors
/// The text does not parse. A deck that parses but does not resolve (an undefined variable)
/// still lists its cards, without values.
pub fn cards(src: &str, mnemonics: &[&str]) -> Result<Vec<CardLine>> {
    let deck = parse_deck(src)?;
    let mut resolved = deck.clone();
    let ok = resolved.resolve().is_ok();
    let pick =
        |m: &str| mnemonics.is_empty() || mnemonics.iter().any(|x| x.eq_ignore_ascii_case(m));
    let mut out = Vec::new();
    for (raw, res) in deck.items.iter().zip(&resolved.items) {
        let (Item::Card(c), Item::Card(r)) = (raw, res) else {
            continue;
        };
        if !pick(&c.mnemonic) {
            continue;
        }
        out.push(CardLine {
            line: c.line,
            mnemonic: c.mnemonic.clone(),
            fields: c.fields.iter().map(|f| f.text.clone()).collect(),
            values: r
                .fields
                .iter()
                .map(|f| if ok { f.value } else { None })
                .collect(),
            comment: c.comment.clone(),
        });
    }
    Ok(out)
}

/// The line of the first `GE` card: where the geometry section ends, and so where a new
/// geometry card goes.
#[must_use]
pub fn geometry_end(src: &str) -> Option<usize> {
    let deck = parse_deck(src).ok()?;
    deck.items.iter().find_map(|it| match it {
        Item::Card(c) if c.mnemonic == "GE" => Some(c.line),
        _ => None,
    })
}

/// The text's lines and the line ending it uses (`\r\n` when its first line ends so).
fn split(src: &str) -> (Vec<&str>, &'static str) {
    let eol = if src.split('\n').next().is_some_and(|l| l.ends_with('\r')) && src.contains('\n') {
        "\r\n"
    } else {
        "\n"
    };
    let body = src.strip_suffix('\n').unwrap_or(src);
    let lines = if src.is_empty() {
        Vec::new()
    } else {
        body.split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .collect()
    };
    (lines, eol)
}

fn join(lines: &[String], eol: &str) -> String {
    let mut s = lines.join(eol);
    s.push_str(eol);
    s
}

/// A card's text: the mnemonic and fields separated by `sep`.
fn card_text(mnemonic: &str, fields: &[String], sep: &str) -> String {
    let mut s = mnemonic.to_string();
    for f in fields {
        s.push_str(sep);
        s.push_str(f.trim());
    }
    s
}

/// Rewrite the card on `line` with new fields (and mnemonic), keeping its indentation, its
/// trailing comment and its separator (tabs when it used tabs).
///
/// Returns `None` when `line` is not a line of the text.
#[must_use]
pub fn replace_card(src: &str, line: usize, mnemonic: &str, fields: &[String]) -> Option<String> {
    let (lines, eol) = split(src);
    let old = *lines.get(line.checked_sub(1)?)?;
    let indent = &old[..old.len() - old.trim_start().len()];
    let cut = old.find(['\'', '!']);
    let (payload, comment) = match cut {
        Some(i) => (&old[..i], &old[i..]),
        None => (old, ""),
    };
    let sep = if payload.trim().contains('\t') {
        "\t"
    } else {
        " "
    };
    let mut new = format!("{indent}{}", card_text(mnemonic, fields, sep));
    if !comment.is_empty() {
        // Keep the gap before the comment as it was.
        let gap = &payload[payload.trim_end().len()..];
        new.push_str(if gap.is_empty() { " " } else { gap });
        new.push_str(comment);
    }
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    out[line - 1] = new;
    Some(join(&out, eol))
}

/// Insert a card before `line` (after the last line when `line` is past the end).
#[must_use]
pub fn insert_card(src: &str, line: usize, mnemonic: &str, fields: &[String]) -> String {
    let (lines, eol) = split(src);
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    let at = line.saturating_sub(1).min(out.len());
    out.insert(at, card_text(mnemonic, fields, " "));
    join(&out, eol)
}

/// Remove `line`. Returns `None` when it is not a line of the text.
#[must_use]
pub fn delete_line(src: &str, line: usize) -> Option<String> {
    let (lines, eol) = split(src);
    let i = line.checked_sub(1).filter(|&i| i < lines.len())?;
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    out.remove(i);
    Some(join(&out, eol))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECK: &str = "CE dipole\nSY len=.25\nGW 1 9 0 -len 0 0 len 0 .001  ' the dipole\nGE 0\nEX 0 1 5 0 1\nFR 0 1 0 0 300\nEN\n";
    const S: fn(&[&str]) -> Vec<String> = |v| v.iter().map(|s| (*s).to_string()).collect();

    #[test]
    fn lists_cards_with_text_and_values() {
        let gw = cards(DECK, &["GW"]).expect("parses");
        assert_eq!(gw.len(), 1);
        assert_eq!(gw[0].line, 3);
        assert_eq!(gw[0].fields[3], "-len");
        assert_eq!(gw[0].values[3], Some(-0.25));
        assert_eq!(gw[0].comment.as_deref(), Some(" the dipole"));
        assert_eq!(cards(DECK, &[]).expect("parses").len(), 5);
        assert_eq!(geometry_end(DECK), Some(4));
    }

    #[test]
    fn rewrites_one_card_keeping_its_comment() {
        let out = replace_card(
            DECK,
            3,
            "GW",
            &S(&["1", "11", "0", "-len", "0", "0", "len", "0", ".002"]),
        )
        .expect("line");
        assert_eq!(
            out,
            DECK.replace(
                "GW 1 9 0 -len 0 0 len 0 .001  ' the dipole",
                "GW 1 11 0 -len 0 0 len 0 .002  ' the dipole"
            )
        );
        assert!(replace_card(DECK, 99, "GW", &[]).is_none());
    }

    #[test]
    fn keeps_tabs_and_crlf() {
        let src = "CE\r\nGW\t1\t9\t0\t0\t0\t0\t0\t1\t.001\r\nGE\t0\r\nEN\r\n";
        let out = replace_card(
            src,
            2,
            "GW",
            &S(&["1", "9", "0", "0", "0", "0", "0", "2", ".001"]),
        )
        .expect("line");
        assert_eq!(
            out,
            "CE\r\nGW\t1\t9\t0\t0\t0\t0\t0\t2\t.001\r\nGE\t0\r\nEN\r\n"
        );
        let out = insert_card(
            &out,
            3,
            "GW",
            &S(&["2", "3", "1", "0", "0", "1", "0", "1", ".001"]),
        );
        assert_eq!(out.lines().nth(2), Some("GW 2 3 1 0 0 1 0 1 .001"));
        assert!(out.ends_with("GE\t0\r\nEN\r\n"));
    }

    #[test]
    fn inserts_and_deletes_lines() {
        let out = insert_card(
            DECK,
            4,
            "GW",
            &S(&["2", "3", "0", "0", "0", "0", "0", "1", ".001"]),
        );
        assert_eq!(geometry_end(&out), Some(5));
        assert_eq!(delete_line(&out, 4).as_deref(), Some(DECK));
        assert!(delete_line(DECK, 0).is_none());
        assert_eq!(insert_card("CE\n", 99, "EN", &[]), "CE\nEN\n");
    }
}
