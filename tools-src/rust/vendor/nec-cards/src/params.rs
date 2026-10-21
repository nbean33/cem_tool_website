//! A deck's `SY` variables as parameters: listing them, and writing new values back.
//!
//! Sweeps and the optimizer vary a deck through its `SY` variables, never through card
//! fields directly: a 4nec2 model is written in terms of names like `Len` and `Spc`, and the
//! wires follow when the names change (see [`Deck::resolve_with`]). Writing the result back
//! edits only the right-hand sides of the assignments, so the author's layout, comments and
//! every other line survive byte for byte.

use crate::deck::{Deck, Item, split_assignments};
use crate::expr::is_plain_number;

/// One `SY` variable, as first assigned in the deck.
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    /// The name as written at its first assignment.
    pub name: String,
    /// The value after resolution (the last assignment wins, as in [`Deck::resolve`]).
    pub value: f64,
    /// The right-hand side of the first assignment, as written.
    pub expression: String,
    /// Whether every assignment is a plain number. Only such variables are free to tune; one
    /// defined by an expression follows the variables it is computed from.
    pub tunable: bool,
    /// 1-based line of the first assignment.
    pub line: usize,
    /// The trailing comment of that line, trimmed, if any: a label such as
    /// `Overall length, m`, shared by every variable the line assigns.
    pub comment: Option<String>,
}

/// The variables a resolved deck defines, in order of first assignment.
///
/// Values come from [`Deck::symbols`], so call [`Deck::resolve`] (or
/// [`Deck::resolve_with`]) first; on an unresolved deck every value is NaN.
#[must_use]
pub fn variables(deck: &Deck) -> Vec<Variable> {
    let mut out: Vec<Variable> = Vec::new();
    for item in &deck.items {
        let Item::Sy { raw, line, comment } = item else {
            continue;
        };
        let Ok(assignments) = split_assignments(raw, *line) else {
            continue;
        };
        for (name, rhs) in assignments {
            let literal = is_plain_number(&rhs);
            match out.iter_mut().find(|v| v.name.eq_ignore_ascii_case(&name)) {
                Some(v) => v.tunable &= literal,
                None => out.push(Variable {
                    value: deck.symbols.get(&name).unwrap_or(f64::NAN),
                    name,
                    expression: rhs,
                    tunable: literal,
                    line: *line,
                    comment: comment.as_deref().map(str::trim).filter(|c| !c.is_empty()).map(str::to_string),
                }),
            }
        }
    }
    out
}

/// Rewrite the `SY` assignments of `src` that set the given variables, leaving everything else
/// exactly as written.
///
/// Every assignment to a named variable (matched case-insensitively) gets the new value as its
/// right-hand side, keeping the spacing around it and any trailing comment. Names not assigned
/// in `src` are ignored.
#[must_use]
pub fn set_variables(src: &str, values: &[(&str, f64)]) -> String {
    let mut out = String::with_capacity(src.len() + 16 * values.len());
    for piece in src.split_inclusive('\n') {
        out.push_str(&rewrite_line(piece, values));
    }
    out
}

/// A number written compactly to seven significant figures.
#[must_use]
pub fn format_value(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return if v.is_finite() {
            "0".into()
        } else {
            format!("{v}")
        };
    }
    let magnitude = v.abs().log10().floor();
    if !(-4.0..7.0).contains(&magnitude) {
        let s = format!("{v:.6e}");
        let (mantissa, exponent) = s.split_once('e').unwrap_or((&s, "0"));
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        return format!("{mantissa}e{exponent}");
    }
    // Seven significant figures: the digits after the point that leaves.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let decimals = (6.0 - magnitude).max(0.0) as usize;
    let s = format!("{v:.decimals$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

fn rewrite_line(line: &str, values: &[(&str, f64)]) -> String {
    let body = line.trim_start();
    let indent = line.len() - body.len();
    let is_sy = body.len() > 2
        && body[..2].eq_ignore_ascii_case("SY")
        && !body.as_bytes()[2].is_ascii_alphabetic();
    if !is_sy {
        return line.to_string();
    }
    // The payload runs from after the mnemonic to the first comment delimiter.
    let start = indent + 2;
    let end = line[start..]
        .find(['\'', '!', '\r', '\n'])
        .map_or(line.len(), |i| start + i);

    // Byte spans of the right-hand sides to replace, found by splitting on top-level commas.
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let mut depth = 0i32;
    let mut part_start = start;
    let payload = &line[start..end];
    for (i, c) in payload
        .char_indices()
        .chain(std::iter::once((payload.len(), ',')))
    {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let part = &line[part_start..start + i];
                if let Some(eq) = part.find('=') {
                    let name = part[..eq].trim();
                    if let Some((_, v)) = values.iter().find(|(n, _)| n.eq_ignore_ascii_case(name))
                    {
                        let rhs = &part[eq + 1..];
                        let lead = rhs.len() - rhs.trim_start().len();
                        let trail = rhs.len() - rhs.trim_end().len();
                        let a = part_start + eq + 1 + lead;
                        let b = part_start + part.len() - trail;
                        edits.push((a, b.max(a), format_value(*v)));
                    }
                }
                part_start = start + i + 1;
            }
            _ => {}
        }
    }
    let mut s = line.to_string();
    for (a, b, text) in edits.into_iter().rev() {
        s.replace_range(a..b, &text);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::SymbolTable;
    use crate::parse_deck;

    const SRC: &str = "CM test\r\nSY len=0.25, h = 10 ' height\r\nSY d=len*2\r\n  sy H=12\r\n\
                       GW 1 9 0 -len h 0 len h 0.001\r\nEN\r\n";

    #[test]
    fn lists_variables_in_order() {
        let mut deck = parse_deck(SRC).expect("parses");
        deck.resolve().expect("resolves");
        let v = variables(&deck);
        let names: Vec<_> = v.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names, ["len", "h", "d"]);
        assert!(v[0].tunable && v[1].tunable && !v[2].tunable);
        // The last assignment wins.
        assert!((v[1].value - 12.0).abs() < 1e-12);
        assert!((v[2].value - 0.5).abs() < 1e-12);
        // A line's comment labels the variables it assigns.
        assert_eq!(v[1].comment.as_deref(), Some("height"));
        assert_eq!(v[2].comment, None);
    }

    #[test]
    fn overrides_flow_through_expressions() {
        let mut deck = parse_deck(SRC).expect("parses");
        let mut pins = SymbolTable::new();
        pins.set("LEN", 0.3);
        deck.resolve_with(&pins).expect("resolves");
        assert!((deck.symbols.get("d").expect("d") - 0.6).abs() < 1e-12);
        let gw = deck.cards("GW").next().expect("GW");
        assert_eq!(gw.fields[3].value, Some(-0.3));
    }

    #[test]
    fn rewrites_only_the_assignments() {
        let out = set_variables(SRC, &[("LEN", 0.3125), ("h", 7.5)]);
        assert_eq!(
            out,
            "CM test\r\nSY len=0.3125, h = 7.5 ' height\r\nSY d=len*2\r\n  sy H=7.5\r\n\
             GW 1 9 0 -len h 0 len h 0.001\r\nEN\r\n"
        );
        assert_eq!(set_variables(SRC, &[]), SRC);
    }

    #[test]
    fn formats_compactly() {
        assert_eq!(format_value(0.25), "0.25");
        assert_eq!(format_value(10.0), "10");
        assert_eq!(format_value(1.0 / 3.0), "0.3333333");
        assert_eq!(format_value(-123.456_789_1), "-123.4568");
        assert_eq!(format_value(1.5e-6), "1.5e-6");
        assert_eq!(format_value(0.0), "0");
    }
}
