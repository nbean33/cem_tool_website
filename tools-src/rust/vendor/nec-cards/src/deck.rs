//! Lossless in-memory representation of a NEC card deck.
//!
//! The representation is deliberately *syntactic*, not semantic: a card is a mnemonic plus a
//! list of fields. Giving `GW` or `GN` their own typed structs belongs in `nec-geom` and the
//! solver crates, which know what the fields mean. Keeping this layer dumb means an unknown
//! or rare card still round-trips without loss, which is what lets the whole 467-model corpus
//! load before any physics exists.
//!
//! Each field keeps both its original text and, once resolved, its numeric value. That single
//! choice lets one structure serve two jobs: re-emitting the authoring form faithfully
//! (tabs and symbols intact) and emitting the purely numeric deck the reference engine needs.

use crate::error::{CardError, Result};
use crate::expr::{self, SymbolTable};

/// What a field is expected to hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// A number, possibly written as a symbolic expression.
    Numeric,
    /// Free text that is never evaluated — currently only the NGF filename on a `GF` card,
    /// as in `GF 0 RADIAL8.NGF`.
    Text,
    /// A position along a wire as a percentage of its length, 4nec2's `EX 0 2 50% 0 1 0`. Its
    /// value is the percentage until [`Deck::resolve_percentages`] turns it into a segment
    /// number; until then it cannot be read as a number.
    Percent,
}

/// One field of a card.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The field exactly as it appeared in the source, e.g. `"-Dril"` or `"1.33173e-3"`.
    pub text: String,
    /// The numeric value, present when the field was a plain number or has been resolved.
    /// Always `None` for [`FieldKind::Text`].
    pub value: Option<f64>,
    /// Whether this field is numeric or free text.
    pub kind: FieldKind,
}

/// Whether field `idx` (zero-based) of `mnemonic` holds free text rather than a number.
///
/// Both Numerical Green's Function cards carry a filename, at different positions:
///
/// ```text
/// GF 0 RADIAL8.NGF      ' read an NGF file  -> field 1
/// WG RADIAL4.NGF        ' write an NGF file -> field 0
/// ```
#[must_use]
pub fn is_text_field(mnemonic: &str, idx: usize) -> bool {
    match mnemonic {
        "GF" => idx == 1,
        "WG" => idx == 0,
        _ => false,
    }
}

/// The numerical Green's function file a `WG` or `GF` card names: its first field that does
/// not start with `0` or `1` (as NEC's `PARSIT` decides), or NEC's default `NGF2D.NEC`.
///
/// ```text
/// WG RADIAL4.NGF        -> RADIAL4.NGF
/// GF 0 RADIAL8.NGF      -> RADIAL8.NGF
/// GF                    -> NGF2D.NEC
/// ```
#[must_use]
pub fn ngf_file_name(card: &Card) -> String {
    card.fields
        .iter()
        .find(|f| f.kind == FieldKind::Text)
        .map_or_else(|| "NGF2D.NEC".to_string(), |f| f.text.clone())
}

/// Wire radius in metres for an AWG gauge number.
///
/// 4nec2 decks abbreviate the `GW` radius field as `#12`, meaning AWG 12 wire. This is data,
/// not a comment: across the corpus, 242 of the `GW` lines containing `#n` have exactly nine
/// fields with `#n` in the radius position.
///
/// Diameter follows the standard definition `d = 0.127 mm * 92^((36-n)/39)`; AWG 12 gives
/// 2.05232 mm, which matches a corpus model that writes that exact figure as a millimetre
/// diameter for #12 wire.
///
/// # Units
///
/// The radius is returned in **metres**. `HFsimple\Isotrop.nec` settles this: it writes
/// `SY D = #12/in` and uses `D` as the radius of a model built in inches, which yields AWG 12's
/// radius in inches only if `#12` is itself in metres.
///
/// Note the geometry gate cannot confirm this — the engine reads the number we emit, so both
/// sides agree whatever the unit. The evidence is the corpus usage above.
#[must_use]
pub fn awg_radius_m(gauge: f64) -> f64 {
    let diameter_mm = 0.127 * 92.0_f64.powf((36.0 - gauge) / 39.0);
    0.5 * diameter_mm * 1.0e-3
}

/// Parse an AWG shorthand field such as `#12`, returning the radius in metres.
fn parse_awg(text: &str) -> Option<f64> {
    let rest = text.strip_prefix('#')?;
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let gauge: f64 = rest.parse().ok()?;
    Some(awg_radius_m(gauge))
}

impl Field {
    /// A field from source text, evaluating it immediately if it is a plain number or an
    /// AWG shorthand.
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        Self::from_text_kind(text, FieldKind::Numeric)
    }

    /// A free-text field, kept verbatim and never evaluated.
    #[must_use]
    pub fn text_field(text: &str) -> Self {
        Self {
            text: text.trim().to_string(),
            value: None,
            kind: FieldKind::Text,
        }
    }

    /// A field of a given kind.
    #[must_use]
    pub fn from_text_kind(text: &str, kind: FieldKind) -> Self {
        if kind == FieldKind::Text {
            return Self::text_field(text);
        }
        if let Some(radius) = parse_awg(text.trim()) {
            return Self {
                text: text.trim().to_string(),
                value: Some(radius),
                kind,
            };
        }
        let value = if expr::is_plain_number(text) {
            text.trim().parse::<f64>().ok().or_else(|| {
                // Fortran `D` exponent form.
                let swapped: String = text
                    .trim()
                    .chars()
                    .map(|c| if c == 'd' || c == 'D' { 'e' } else { c })
                    .collect();
                swapped.parse::<f64>().ok()
            })
        } else {
            None
        };
        Self {
            text: text.trim().to_string(),
            value,
            kind,
        }
    }

    /// Whether this field still needs symbol resolution.
    ///
    /// Free-text fields never do, so they are not reported as unresolved.
    #[must_use]
    pub fn is_symbolic(&self) -> bool {
        self.kind == FieldKind::Numeric && self.value.is_none()
    }

    /// The resolved value, or an error naming the card and field.
    pub fn require(&self, mnemonic: &str, line: usize) -> Result<f64> {
        if self.kind == FieldKind::Percent {
            return Err(CardError::UnsupportedFieldSyntax {
                line,
                mnemonic: mnemonic.to_string(),
                field: self.text.clone(),
                reason: "a percentage position becomes a segment number only once the geometry is built",
            });
        }
        self.value.ok_or_else(|| CardError::BadField {
            line,
            mnemonic: mnemonic.to_string(),
            field: self.text.clone(),
        })
    }
}

/// A data card: two-letter mnemonic plus fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    /// Uppercased two-letter mnemonic, e.g. `"GW"`.
    pub mnemonic: String,
    /// The card's fields, in source order.
    pub fields: Vec<Field>,
    /// Trailing `'` comment, text only, without the quote.
    pub comment: Option<String>,
    /// 1-based source line.
    pub line: usize,
}

impl Card {
    /// Integer field by zero-based index, truncating the stored f64.
    pub fn int(&self, idx: usize) -> Result<i32> {
        let f = self.real(idx)?;
        #[allow(clippy::cast_possible_truncation)]
        Ok(f.trunc() as i32)
    }

    /// Real field by zero-based index.
    pub fn real(&self, idx: usize) -> Result<f64> {
        let f = self.fields.get(idx).ok_or_else(|| CardError::TooFewFields {
            line: self.line,
            mnemonic: self.mnemonic.clone(),
            needed: idx + 1,
            found: self.fields.len(),
        })?;
        f.require(&self.mnemonic, self.line)
    }

    /// Real field by index, or `default` when the field is absent. NEC decks routinely omit
    /// trailing zero fields.
    pub fn real_or(&self, idx: usize, default: f64) -> Result<f64> {
        match self.fields.get(idx) {
            None => Ok(default),
            Some(f) => f.require(&self.mnemonic, self.line),
        }
    }
}

/// One line of a deck.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// An empty or whitespace-only line.
    Blank,
    /// A whole-line comment introduced by `'`, text without the quote.
    LineComment(String),
    /// A `CM` or `CE` comment card, whose remainder is free text rather than fields.
    CommentCard {
        /// `"CM"` or `"CE"`.
        mnemonic: String,
        /// The free text following the mnemonic.
        text: String,
    },
    /// An `SY` symbolic-variable card (a 4nec2 extension).
    Sy {
        /// The assignment text, e.g. `"D1=LOG(D)/log(10), D4=D1^4"`.
        raw: String,
        /// Trailing `'` comment.
        comment: Option<String>,
        /// 1-based source line.
        line: usize,
    },
    /// Any other card.
    Card(Card),
}

/// A parsed deck, in source order.
#[derive(Debug, Clone, Default)]
pub struct Deck {
    /// Every line, including blanks and comments, so the authoring form can be rebuilt.
    pub items: Vec<Item>,
    /// Variables bound by `SY` cards. Empty until [`Deck::resolve`] runs.
    pub symbols: SymbolTable,
}

impl Deck {
    /// Whether any field anywhere still needs symbol resolution.
    ///
    /// Only card fields count. The presence of an `SY` card does not by itself mean anything
    /// is unresolved — after [`Deck::resolve`] the `SY` cards remain in the deck (so the
    /// authoring form can be rebuilt) while every field has a value.
    #[must_use]
    pub fn has_symbolic_fields(&self) -> bool {
        self.items.iter().any(|it| match it {
            Item::Card(c) => c.fields.iter().any(Field::is_symbolic),
            _ => false,
        })
    }

    /// Whether this deck uses the `SY` extension at all.
    #[must_use]
    pub fn uses_sy(&self) -> bool {
        self.items.iter().any(|it| matches!(it, Item::Sy { .. }))
    }

    /// The deck's runs. `NX` ends one structure and starts the next, which begins again with
    /// comment cards and geometry; each run is solved on its own, except that a later run may
    /// read the numerical Green's function an earlier one wrote. A deck without `NX` is one run.
    #[must_use]
    pub fn runs(&self) -> Vec<Deck> {
        let mut out = vec![Deck {
            items: Vec::new(),
            symbols: self.symbols.clone(),
        }];
        for item in &self.items {
            if matches!(item, Item::Card(c) if c.mnemonic == "NX") {
                out.push(Deck {
                    items: Vec::new(),
                    symbols: self.symbols.clone(),
                });
            } else if let Some(run) = out.last_mut() {
                run.items.push(item.clone());
            }
        }
        // A trailing NX with nothing after it starts no run.
        if out.len() > 1 && out.last().is_some_and(|r| !r.items.iter().any(|i| matches!(i, Item::Card(_)))) {
            out.pop();
        }
        out
    }

    /// Cards with the given mnemonic.
    pub fn cards(&self, mnemonic: &str) -> impl Iterator<Item = &Card> {
        let want = mnemonic.to_ascii_uppercase();
        self.items.iter().filter_map(move |it| match it {
            Item::Card(c) if c.mnemonic == want => Some(c),
            _ => None,
        })
    }

    /// Evaluate all `SY` cards in order and substitute every symbolic field with its value.
    ///
    /// `SY` cards are evaluated sequentially, so a later card may redefine a variable and may
    /// reference variables defined earlier — which is how the corpus writes derived
    /// quantities such as `SY D1=LOG(D)/log(10), D4=D1^4`.
    ///
    /// Cards are resolved against the symbol table *as it stands at that point in the deck*,
    /// matching 4nec2's top-to-bottom expansion.
    pub fn resolve(&mut self) -> Result<()> {
        self.resolve_with(&SymbolTable::new())
    }

    /// [`Deck::resolve`], with some variables pinned to given values.
    ///
    /// Every `SY` assignment to an overridden name takes the override instead of its own
    /// expression, and everything derived from it is recomputed — which is what a parameter
    /// sweep or an optimizer needs: change `Len` and every wire written in terms of `Len`
    /// follows. Overrides of names the deck never assigns have no effect; see
    /// [`crate::variables`] for the names a deck defines.
    pub fn resolve_with(&mut self, overrides: &SymbolTable) -> Result<()> {
        let mut syms = SymbolTable::new();
        let pinned = |name: &str| {
            overrides
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v)
        };

        for item in &mut self.items {
            match item {
                Item::Sy { raw, line, .. } => {
                    for (name, rhs) in split_assignments(raw, *line)? {
                        let v = match pinned(&name) {
                            Some(v) => v,
                            None => expr::eval(&rhs, &syms, *line)?,
                        };
                        syms.set(&name, v);
                    }
                }
                Item::Card(card) => {
                    let line = card.line;
                    let mnemonic = card.mnemonic.clone();
                    for (i, f) in card.fields.iter_mut().enumerate() {
                        if !f.is_symbolic() {
                            continue;
                        }
                        // `EX 0 2 50% 0 1 0` places the source at 50% of the wire's length.
                        // Converting that to a segment number needs the segmentation; see
                        // `resolve_percentages`. 4nec2 (5.7.0 on) accepts it for sources, spot
                        // loads and both ends of lines and networks.
                        if let Some(pct) = f.text.strip_suffix('%') {
                            if percent_tag_field(&mnemonic, i).is_none() {
                                return Err(CardError::UnsupportedFieldSyntax {
                                    line,
                                    mnemonic: mnemonic.clone(),
                                    field: f.text.clone(),
                                    reason: "a percentage position is understood only for the segment fields of EX, LD, TL and NT",
                                });
                            }
                            f.value = Some(expr::eval(pct, &syms, line)?);
                            f.kind = FieldKind::Percent;
                            continue;
                        }
                        f.value = Some(expr::eval(&f.text, &syms, line)?);
                    }
                }
                Item::Blank | Item::LineComment(_) | Item::CommentCard { .. } => {}
            }
        }

        self.symbols = syms;
        Ok(())
    }
}

impl Deck {
    /// Turn percentage positions — on `EX`, `LD`, `TL` and `NT` segment fields — into segment
    /// numbers, given how many segments carry each tag. Call after [`Deck::resolve`], once the
    /// geometry is built.
    ///
    /// The rule is 4nec2's, measured from what it hands the engine: `p`% of a wire of `N`
    /// segments is segment `p·N/100 + ½` rounded half to even (Visual Basic's `CInt`), kept
    /// within `1..=N`. So 50% of a 10-segment wire is segment 6 and 20% is segment 2, while 25% of
    /// a 10-segment wire, 30% and 33.33% of a 9-segment one give 3. (4nec2 itself mangles a deck
    /// with more than 30 such sources; there is no such limit here.)
    pub fn resolve_percentages(&mut self, segments_with_tag: impl Fn(i32) -> usize) -> Result<()> {
        for item in &mut self.items {
            let Item::Card(card) = item else { continue };
            for i in 0..card.fields.len() {
                if card.fields[i].kind != FieldKind::Percent {
                    continue;
                }
                let Some(tag_field) = percent_tag_field(&card.mnemonic, i) else { continue };
                let tag = card.int(tag_field)?;
                let n = segments_with_tag(tag);
                if n == 0 {
                    return Err(CardError::BadField {
                        line: card.line,
                        mnemonic: card.mnemonic.clone(),
                        field: format!("{} (no segment carries tag {tag})", card.fields[i].text),
                    });
                }
                let pct = card.fields[i].value.unwrap_or(0.0);
                #[allow(clippy::cast_precision_loss)]
                let x = (pct * n as f64 / 100.0 + 0.5).round_ties_even();
                #[allow(clippy::cast_precision_loss)]
                let seg = x.clamp(1.0, n as f64);
                card.fields[i] = Field {
                    text: format!("{seg}"),
                    value: Some(seg),
                    kind: FieldKind::Numeric,
                };
            }
        }
        Ok(())
    }
}

/// For a segment field that may hold a percentage position, the field holding its wire's tag:
/// `EX`'s segment (tag in field 1), `LD`'s first and last segments (tag in field 1), and the
/// two ends of `TL` and `NT` (tags in fields 0 and 2).
#[must_use]
pub fn percent_tag_field(mnemonic: &str, field: usize) -> Option<usize> {
    match (mnemonic, field) {
        ("EX", 2) | ("LD", 2 | 3) => Some(1),
        ("TL" | "NT", 1) => Some(0),
        ("TL" | "NT", 3) => Some(2),
        _ => None,
    }
}

/// Split an `SY` payload into `(name, expression)` pairs.
///
/// Assignments are comma-separated, but commas also appear inside expressions is not a case
/// the corpus exhibits — functions here are all single-argument. Even so, the split tracks
/// parenthesis depth so `f(a,b)` would not be mis-split if it ever appeared.
pub(crate) fn split_assignments(raw: &str, line: usize) -> Result<Vec<(String, String)>> {
    let mut parts: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for c in raw.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }

    let mut out = Vec::new();
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        let (name, rhs) = p.split_once('=').ok_or_else(|| CardError::BadAssignment {
            line,
            text: p.to_string(),
        })?;
        let name = name.trim();
        if name.is_empty() {
            return Err(CardError::BadAssignment {
                line,
                text: p.to_string(),
            });
        }
        out.push((name.to_string(), rhs.trim().to_string()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nx_splits_a_deck_into_runs_and_ngf_files_are_named() {
        let mut d = crate::parse_deck("CE\nGW 1 3 0 0 0 1 0 0 1e-3\nGE 0\nWG\nNX\nCE\nGF 0 R8.NGF\nGE 0\nEN\n")
            .expect("parses");
        d.resolve().expect("resolves");
        let runs = d.runs();
        assert_eq!(runs.len(), 2);
        let wg = runs[0].cards("WG").next().expect("WG");
        assert_eq!(ngf_file_name(wg), "NGF2D.NEC");
        let gf = runs[1].cards("GF").next().expect("GF");
        assert_eq!(ngf_file_name(gf), "R8.NGF");
        assert_eq!(gf.fields[0].value, Some(0.0));
        let named = crate::parse_deck("WG RADIAL4.NGF\n").expect("parses");
        assert_eq!(ngf_file_name(named.cards("WG").next().expect("WG")), "RADIAL4.NGF");
    }

    #[test]
    fn percentage_positions_follow_4nec2s_rounding() {
        // Measured from 4nec2's own output: (segments, percent) -> segment.
        let cases = [
            (10, 0.0, 1),
            (10, 9.0, 1),
            (10, 10.0, 2),
            (10, 20.0, 2),
            (10, 30.0, 4),
            (10, 40.0, 4),
            (10, 50.0, 6),
            (10, 95.0, 10),
            (10, 100.0, 10),
            (9, 33.33, 3),
            (9, 94.0, 9),
            (4, 50.0, 2),
            (3, 67.0, 3),
            (7, 50.0, 4),
            (8, 62.5, 6),
            (1, 50.0, 1),
        ];
        for (n, pct, want) in cases {
            let mut d = crate::parse_deck(&format!("GW 1 {n} 0 0 0 1 0 0 1e-3\nGE 0\nEX 0 1 {pct}% 0 1 0\nEN\n"))
                .expect("parses");
            d.resolve().expect("resolves");
            assert!(d.cards("EX").next().expect("EX").int(2).is_err(), "unconverted percent must not read as a number");
            d.resolve_percentages(|_| n).expect("converts");
            assert_eq!(d.cards("EX").next().expect("EX").int(2).expect("segment"), want, "{pct}% of {n}");
        }
    }

    #[test]
    fn percentages_work_on_loads_and_lines_too() {
        let src = "GW 1 10 0 0 0 1 0 0 1e-3
GW 2 7 0 1 0 1 1 0 1e-3
GE 0
LD 0 1 20% 50% 50 0 0
TL 1 50% 2 100% 50 0
NT 2 0% 1 30% 0 0 0 1 0 0
EN
";
        let mut d = crate::parse_deck(src).expect("parses");
        d.resolve().expect("resolves");
        d.resolve_percentages(|t| if t == 1 { 10 } else { 7 }).expect("converts");
        let ld = d.cards("LD").next().expect("LD");
        assert_eq!((ld.int(2).expect("from"), ld.int(3).expect("to")), (2, 6));
        let tl = d.cards("TL").next().expect("TL");
        assert_eq!((tl.int(1).expect("1"), tl.int(3).expect("2")), (6, 7));
        let nt = d.cards("NT").next().expect("NT");
        assert_eq!((nt.int(1).expect("1"), nt.int(3).expect("2")), (1, 4)); // CInt(3.5) = 4
        // A percentage anywhere else is still refused.
        let mut bad = crate::parse_deck("GW 1 10 0 0 0 1 0 0 50%
GE 0
EN
").expect("parses");
        assert!(bad.resolve().is_err());
    }

    #[test]
    fn field_detects_plain_numbers_and_symbols() {
        assert_eq!(Field::from_text("-.2418").value, Some(-0.2418));
        assert_eq!(Field::from_text("1.33173e-3").value, Some(1.33173e-3));
        assert!(Field::from_text("-Dril").is_symbolic());
        assert!(Field::from_text("int(hgh)").is_symbolic());
    }

    #[test]
    fn awg_gauges_match_standard_diameters() {
        // Published AWG diameters in mm: #12 = 2.05232, #14 = 1.62814, #0 = 8.25246.
        // Those table figures are rounded; the defining formula agrees to about 1e-3 mm,
        // which is the tolerance asserted here. Tighter would be testing the rounding of
        // the table rather than the formula.
        let d = |n: f64| awg_radius_m(n) * 2.0 * 1000.0;
        assert!((d(12.0) - 2.052_32).abs() < 2e-3, "#12 -> {} mm", d(12.0));
        assert!((d(14.0) - 1.628_14).abs() < 2e-3, "#14 -> {} mm", d(14.0));
        assert!((d(0.0) - 8.252_46).abs() < 2e-3, "#0 -> {} mm", d(0.0));

        // Gauge ordering: a larger AWG number is a thinner wire.
        assert!(d(18.0) < d(14.0) && d(14.0) < d(12.0) && d(12.0) < d(0.0));
    }

    #[test]
    fn awg_field_resolves_without_symbols() {
        let f = Field::from_text("#12");
        assert!(!f.is_symbolic(), "#12 must resolve to a radius");
        let r = f.value.expect("resolved");
        // 1.026263e-3 m from the formula.
        assert!((r - 1.026_262_69e-3).abs() < 1e-9, "got {r}");
    }

    #[test]
    fn hash_without_digits_is_not_an_awg_field() {
        assert!(parse_awg("#").is_none());
        assert!(parse_awg("#abc").is_none());
        assert!(parse_awg("12").is_none());
    }

    #[test]
    fn splits_multiple_assignments() {
        let got = split_assignments("D1=LOG(D)/log(10), D4=D1^4, D3=D1^3", 1).expect("ok");
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].0, "D1");
        assert_eq!(got[1].0, "D4");
        assert_eq!(got[2].1, "D1^3");
    }

    #[test]
    fn split_respects_parenthesis_depth() {
        let got = split_assignments("a=f(1), b=2", 1).expect("ok");
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].1, "f(1)");
    }

    #[test]
    fn assignment_without_equals_is_rejected() {
        assert!(matches!(
            split_assignments("justaname", 9),
            Err(CardError::BadAssignment { line: 9, .. })
        ));
    }
}
