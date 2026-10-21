//! Evaluator for 4nec2's `SY` symbolic-variable expressions.
//!
//! This is not part of NEC-2. 4nec2 adds `SY` cards that define named variables, and lets
//! those names (and arithmetic over them) appear in any numeric field:
//!
//! ```text
//! SY Fr=14.05
//! SY D1=LOG(D)/log(10), D4=D1^4
//! GW  1  11  0  -Dril  Hgh  0  Dril  Hgh  Wr
//! GW  1  int(hgh)  0  0  0  0  0  hgh  1e-3
//! ```
//!
//! Half the model corpus (234 of 467 files) uses it, so resolving it is mandatory rather
//! than optional. The reference engine cannot read these files at all — 4nec2 resolves the
//! symbols and emits a purely numeric deck — which is why this evaluator must exist before
//! parametric models can be differential-tested.
//!
//! # Semantics, as determined from the corpus
//!
//! Operators `+ - * / ^` with parentheses and unary sign. `^` is right-associative and binds
//! tighter than unary minus, so `-D1^2` is `-(D1^2)`.
//!
//! Five functions appear across the corpus, and three of their meanings had to be settled by
//! reading usage rather than documentation:
//!
//! | function | meaning | how it was established |
//! |---|---|---|
//! | `sin`, `cos` | **degrees** | `HFbeamswa2wvl.nec` notes `W1 = L1 * SIN(ANG)  ' w1 = 46.397'` with `L1 = 64.5`, `ANG = 46`: 64.5·sin(46°) = 46.397, while radians give 58.165. Every corpus argument is an angle like `ANG = 45` or `ang/2` with `ang = 110` |
//! | `log` | **natural** log | `SY D1=LOG(D)/log(10)` divides by `log(10)` to get base 10 |
//! | `sqr` | **square root** | `SY xylen = sqr(len*len-dz*dz)` is a Pythagorean length |
//! | `int` | truncate toward zero | `SY segV=int(hgh)` is a segment count |
//!
//! Unknown functions are an error rather than silently zero, so that any corpus construct
//! this evaluator does not cover shows up immediately.
//!
//! Variable names are case-insensitive: models define `SY Fr=14.05` and then reference both
//! `FR` and `Fr`.

use crate::error::{CardError, Result};
use std::collections::HashMap;

/// A set of `SY` variable bindings. Names are stored lowercased.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    vars: HashMap<String, f64>,
}

impl SymbolTable {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Define or redefine a variable. Later definitions win, matching 4nec2's sequential
    /// evaluation of `SY` cards.
    pub fn set(&mut self, name: &str, value: f64) {
        self.vars.insert(name.to_ascii_lowercase(), value);
    }

    /// Look up a variable, case-insensitively, falling back to the built-in unit constants.
    ///
    /// User `SY` definitions are checked first, so a model that defines its own `in` or `m`
    /// shadows the built-in rather than fighting it.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<f64> {
        let key = name.to_ascii_lowercase();
        self.vars
            .get(&key)
            .copied()
            .or_else(|| builtin_unit(&key))
    }

    /// Number of bound variables.
    #[must_use]
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// Whether no variables are bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// Iterate over bindings, for diagnostics and GUI display.
    pub fn iter(&self) -> impl Iterator<Item = (&str, f64)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), *v))
    }
}

/// Value of a built-in length-unit constant, in metres, or `None` if `name_lower` is not one.
///
/// 4nec2 predefines unit names so a model can write dimensions in its author's units:
///
/// ```text
/// SY X=135 ft           ' 135 feet, via implicit multiplication
/// SY Inp=mm             ' select millimetres, used later as (WD*Inp*FR)/299.7925
/// ```
///
/// The scale factors were cross-checked against a model that writes `SY WD=2.05232` with the
/// comment "Enter Wire Diameter in mm" for AWG #12 wire, whose diameter is 2.05232 mm — which
/// only works out if `mm` is 1e-3 and the base unit is metres.
/// The electrical units cover `LD` loading cards, where models write values such as
/// `SY RCOIL = 2uH` (no space — implicit multiplication handles the juxtaposition).
///
/// This list is complete for the corpus: scanning every `SY` right-hand side for identifiers
/// never defined in the same file yields exactly `pF`, `uH`, `ft`, `mm`, `in` and `cm`.
#[must_use]
pub fn builtin_unit(name_lower: &str) -> Option<f64> {
    Some(match name_lower {
        // Length, in metres.
        "m" => 1.0,
        "cm" => 0.01,
        "mm" => 1.0e-3,
        "in" => 0.0254,
        "ft" => 0.3048,
        // Inductance, in henries.
        "mh" => 1.0e-3,
        "uh" => 1.0e-6,
        "nh" => 1.0e-9,
        // Capacitance, in farads.
        "mf" => 1.0e-3,
        "uf" => 1.0e-6,
        "nf" => 1.0e-9,
        "pf" => 1.0e-12,
        _ => return None,
    })
}

/// Whether `name` is a bare unit name. Used by the card tokenizer to merge a unit that was
/// written as its own whitespace-separated field, as in `GW 1 11 0 -68 ft X Y -68 ft Z #12`.
#[must_use]
pub fn is_unit_name(name: &str) -> bool {
    builtin_unit(&name.to_ascii_lowercase()).is_some()
}

/// Evaluate `text` against `syms`. `line` is used only for error messages.
pub fn eval(text: &str, syms: &SymbolTable, line: usize) -> Result<f64> {
    let tokens = tokenize(text, line)?;
    let mut p = Parser {
        tokens: &tokens,
        pos: 0,
        syms,
        line,
        src: text,
    };
    let v = p.expr()?;
    if p.pos != p.tokens.len() {
        return Err(CardError::BadExpression {
            line,
            expr: text.to_string(),
            reason: format!("unexpected trailing input at token {}", p.pos),
        });
    }
    Ok(v)
}

/// Whether `text` is a plain number needing no symbol resolution. Used to keep purely
/// numeric decks on a fast path and to detect which fields are symbolic.
#[must_use]
pub fn is_plain_number(text: &str) -> bool {
    parse_number(text.trim()).is_some()
}

/// Parse a NEC-style numeric literal. Accepts forms the corpus uses that Rust's `f64`
/// parser also accepts (`1e-3`, `-.2418`, `58000000`), plus Fortran `D` exponents
/// (`1.5D-3`) which appear in older hand-written decks.
fn parse_number(s: &str) -> Option<f64> {
    if s.is_empty() {
        return None;
    }
    if let Ok(v) = s.parse::<f64>() {
        return Some(v);
    }
    // Fortran double-precision exponent marker.
    if s.contains(['d', 'D']) {
        let swapped: String = s
            .chars()
            .map(|c| if c == 'd' || c == 'D' { 'e' } else { c })
            .collect();
        if let Ok(v) = swapped.parse::<f64>() {
            return Some(v);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
}

fn tokenize(text: &str, line: usize) -> Result<Vec<Tok>> {
    let bytes: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            ' ' | '\t' => i += 1,
            '+' => {
                out.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                out.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                out.push(Tok::Star);
                i += 1;
            }
            '/' => {
                out.push(Tok::Slash);
                i += 1;
            }
            '^' => {
                out.push(Tok::Caret);
                i += 1;
            }
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            // AWG wire-gauge literal inside an expression, e.g. `SY D = #12/in`, which
            // divides the metre value by `in` to express it in inches. Resolves to the wire
            // radius in metres, the same as a bare `#12` radius field.
            '#' => {
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == '.') {
                    j += 1;
                }
                let lit: String = bytes[start..j].iter().collect();
                let gauge = parse_number(&lit).ok_or_else(|| CardError::BadExpression {
                    line,
                    expr: text.to_string(),
                    reason: format!("bad AWG gauge {lit:?} after '#'"),
                })?;
                out.push(Tok::Num(crate::deck::awg_radius_m(gauge)));
                i = j;
            }
            _ if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == '.') {
                    i += 1;
                }
                // Exponent, taking care not to swallow the `e` of an identifier that merely
                // follows a number, and not to treat `1-2` as an exponent.
                if i < bytes.len() && matches!(bytes[i], 'e' | 'E' | 'd' | 'D') {
                    let mut j = i + 1;
                    if j < bytes.len() && matches!(bytes[j], '+' | '-') {
                        j += 1;
                    }
                    if j < bytes.len() && bytes[j].is_ascii_digit() {
                        i = j;
                        while i < bytes.len() && bytes[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                }
                let lit: String = bytes[start..i].iter().collect();
                let v = parse_number(&lit).ok_or_else(|| CardError::BadExpression {
                    line,
                    expr: text.to_string(),
                    reason: format!("bad numeric literal {lit:?}"),
                })?;
                out.push(Tok::Num(v));
            }
            _ if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == '_') {
                    i += 1;
                }
                out.push(Tok::Ident(bytes[start..i].iter().collect()));
            }
            _ => {
                return Err(CardError::BadExpression {
                    line,
                    expr: text.to_string(),
                    reason: format!("unexpected character {c:?}"),
                });
            }
        }
    }
    Ok(out)
}

struct Parser<'a> {
    tokens: &'a [Tok],
    pos: usize,
    syms: &'a SymbolTable,
    line: usize,
    src: &'a str,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos)
    }

    fn bad(&self, reason: impl Into<String>) -> CardError {
        CardError::BadExpression {
            line: self.line,
            expr: self.src.to_string(),
            reason: reason.into(),
        }
    }

    /// `expr := term (('+'|'-') term)*`
    fn expr(&mut self) -> Result<f64> {
        let mut acc = self.term()?;
        while let Some(op) = self.peek().cloned() {
            match op {
                Tok::Plus => {
                    self.pos += 1;
                    acc += self.term()?;
                }
                Tok::Minus => {
                    self.pos += 1;
                    acc -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(acc)
    }

    /// `term := unary (('*'|'/') unary)*`
    fn term(&mut self) -> Result<f64> {
        let mut acc = self.unary()?;
        while let Some(op) = self.peek().cloned() {
            match op {
                Tok::Star => {
                    self.pos += 1;
                    acc *= self.unary()?;
                }
                Tok::Slash => {
                    self.pos += 1;
                    let d = self.unary()?;
                    // Division by zero yields inf/NaN rather than an error: 4nec2 models are
                    // sometimes written so a branch divides by zero but the result is unused.
                    acc /= d;
                }
                // Implicit multiplication, which is how unit suffixes work: `135 ft` is
                // `135 * ft`. Only a token that can begin an atom counts, so `a - b` is
                // still a subtraction rather than `a * (-b)`.
                Tok::Num(_) | Tok::Ident(_) | Tok::LParen => {
                    acc *= self.unary()?;
                }
                _ => break,
            }
        }
        Ok(acc)
    }

    /// `unary := ('-'|'+') unary | power`
    ///
    /// Unary sign sits *outside* exponentiation so that `-3^2` is `-(3^2) = -9`, matching
    /// VB6's precedence (4nec2 is a VB6 program) and ordinary mathematical convention.
    /// Note this differs from Excel, which would give `+9`.
    fn unary(&mut self) -> Result<f64> {
        match self.peek() {
            Some(Tok::Minus) => {
                self.pos += 1;
                Ok(-self.unary()?)
            }
            Some(Tok::Plus) => {
                self.pos += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    /// `power := atom ('^' unary)?` — right-associative, and the exponent may carry a sign
    /// so `2^-1` works.
    fn power(&mut self) -> Result<f64> {
        let base = self.atom()?;
        if matches!(self.peek(), Some(Tok::Caret)) {
            self.pos += 1;
            let exp = self.unary()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    /// `atom := number | ident '(' expr ')' | ident | '(' expr ')'`
    fn atom(&mut self) -> Result<f64> {
        let tok = self
            .peek()
            .cloned()
            .ok_or_else(|| self.bad("unexpected end of expression"))?;
        match tok {
            Tok::Num(v) => {
                self.pos += 1;
                Ok(v)
            }
            Tok::LParen => {
                self.pos += 1;
                let v = self.expr()?;
                if !matches!(self.peek(), Some(Tok::RParen)) {
                    return Err(self.bad("missing closing parenthesis"));
                }
                self.pos += 1;
                Ok(v)
            }
            Tok::Ident(name) => {
                self.pos += 1;
                if matches!(self.peek(), Some(Tok::LParen)) {
                    self.pos += 1;
                    let arg = self.expr()?;
                    if !matches!(self.peek(), Some(Tok::RParen)) {
                        return Err(self.bad("missing closing parenthesis after function argument"));
                    }
                    self.pos += 1;
                    return apply_fn(&name, arg, self.line);
                }
                self.syms
                    .get(&name)
                    .ok_or(CardError::UndefinedSymbol { line: self.line, name })
            }
            other => Err(self.bad(format!("unexpected token {other:?}"))),
        }
    }
}

fn apply_fn(name: &str, arg: f64, line: usize) -> Result<f64> {
    let v = match name.to_ascii_lowercase().as_str() {
        // Degrees, as 4nec2 evaluates them; see the table in the module documentation.
        "sin" => arg.to_radians().sin(),
        "cos" => arg.to_radians().cos(),
        // Natural log: the corpus writes LOG(D)/log(10) to obtain base-10.
        "log" => arg.ln(),
        // BASIC-style SQR is square root, confirmed by Pythagorean usage in the corpus.
        "sqr" => arg.sqrt(),
        // Truncation toward zero; used for segment counts.
        "int" => arg.trunc(),
        _ => {
            return Err(CardError::UnknownFunction {
                line,
                name: name.to_string(),
            });
        }
    };
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syms(pairs: &[(&str, f64)]) -> SymbolTable {
        let mut t = SymbolTable::new();
        for (k, v) in pairs {
            t.set(k, *v);
        }
        t
    }

    fn e(text: &str, t: &SymbolTable) -> f64 {
        eval(text, t, 1).unwrap_or_else(|err| panic!("eval {text:?} failed: {err}"))
    }

    #[test]
    fn arithmetic_and_precedence() {
        let t = SymbolTable::new();
        assert!((e("1+2*3", &t) - 7.0).abs() < 1e-15);
        assert!((e("(1+2)*3", &t) - 9.0).abs() < 1e-15);
        assert!((e("8/2/2", &t) - 2.0).abs() < 1e-15);
    }

    #[test]
    fn power_is_right_associative_and_binds_tighter_than_unary_minus() {
        let t = SymbolTable::new();
        // 2^(3^2) = 512, not (2^3)^2 = 64
        assert!((e("2^3^2", &t) - 512.0).abs() < 1e-9);
        // -(3^2) = -9, not (-3)^2 = 9 — VB6 precedence, unlike Excel.
        assert!((e("-3^2", &t) + 9.0).abs() < 1e-12);
        // A signed exponent still parses.
        assert!((e("2^-1", &t) - 0.5).abs() < 1e-15);
        // Parentheses override, giving the Excel reading explicitly.
        assert!((e("(-3)^2", &t) - 9.0).abs() < 1e-12);
    }

    #[test]
    fn trig_takes_degrees() {
        let t = SymbolTable::new();
        // HFbeams\wa2wvl.nec: W1 = L1 * SIN(ANG), L1 = 64.5, ANG = 46, noted as 46.397.
        assert!((e("64.5*SIN(46)", &t) - 46.397).abs() < 5e-4);
        assert!((e("cos(60)", &t) - 0.5).abs() < 1e-15);
        assert!((e("sin(90)", &t) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn variables_are_case_insensitive() {
        // Real corpus pattern: defined as `Fr`, referenced as both `FR` and `Fr`.
        let t = syms(&[("Fr", 14.05)]);
        assert!((e("FR", &t) - 14.05).abs() < 1e-15);
        assert!((e("fr*2", &t) - 28.1).abs() < 1e-12);
    }

    #[test]
    fn log_is_natural_so_base10_idiom_works() {
        let t = syms(&[("D", 1000.0)]);
        // The corpus idiom LOG(D)/log(10) must yield log10(1000) = 3.
        assert!((e("LOG(D)/log(10)", &t) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn sqr_is_square_root() {
        // From the corpus: SY xylen = sqr(len*len-dz*dz) — a Pythagorean length.
        let t = syms(&[("len", 5.0), ("dz", 3.0)]);
        assert!((e("sqr(len*len-dz*dz)", &t) - 4.0).abs() < 1e-12);
    }

    #[test]
    fn int_truncates_toward_zero() {
        let t = syms(&[("hgh", 7.9), ("neg", -7.9)]);
        assert!((e("int(hgh)", &t) - 7.0).abs() < 1e-15);
        assert!((e("int(neg)", &t) + 7.0).abs() < 1e-15);
    }

    #[test]
    fn real_corpus_polynomial() {
        // Equations/3elYagiGain.nec
        let t = syms(&[("D1", -2.0), ("D2", 4.0), ("D3", -8.0), ("D4", 16.0)]);
        let got = e(
            "0.001832727273*D4+0.02234666667*D3+0.0992369697*D2+0.1852866667*D1+0.3655207792",
            &t,
        );
        let want = 0.001_832_727_273 * 16.0
            + 0.022_346_666_67 * -8.0
            + 0.099_236_969_7 * 4.0
            + 0.185_286_666_7 * -2.0
            + 0.365_520_779_2;
        assert!((got - want).abs() < 1e-12, "got {got}, want {want}");
    }

    #[test]
    fn nested_parens_from_corpus() {
        // SY TTP=(ATT*(D1^4))+(BTT*(D1^3))+(CTT*(D1^2))+(DTT*D1)+ETT
        let t = syms(&[
            ("ATT", 1.0),
            ("BTT", 2.0),
            ("CTT", 3.0),
            ("DTT", 4.0),
            ("ETT", 5.0),
            ("D1", 2.0),
        ]);
        let got = e("(ATT*(D1^4))+(BTT*(D1^3))+(CTT*(D1^2))+(DTT*D1)+ETT", &t);
        assert!((got - (16.0 + 16.0 + 12.0 + 8.0 + 5.0)).abs() < 1e-12);
    }

    #[test]
    fn numeric_literal_forms() {
        assert_eq!(parse_number("-.2418"), Some(-0.2418));
        assert_eq!(parse_number("1e-3"), Some(1e-3));
        assert_eq!(parse_number("1.33173e-3"), Some(1.33173e-3));
        assert_eq!(parse_number("58000000"), Some(58_000_000.0));
        assert_eq!(parse_number("1.5D-3"), Some(1.5e-3));
        assert_eq!(parse_number("Dril"), None);
    }

    #[test]
    fn negated_variable_field() {
        // GW fields like `-Dril` are expressions, not numbers.
        let t = syms(&[("Dril", 0.25)]);
        assert!(!is_plain_number("-Dril"));
        assert!((e("-Dril", &t) + 0.25).abs() < 1e-15);
    }

    #[test]
    fn undefined_symbol_is_an_error() {
        let t = SymbolTable::new();
        assert!(matches!(
            eval("NoSuchVar+1", &t, 42),
            Err(CardError::UndefinedSymbol { line: 42, .. })
        ));
    }

    #[test]
    fn unknown_function_is_an_error_not_zero() {
        let t = SymbolTable::new();
        assert!(matches!(
            eval("tan(1.0)", &t, 7),
            Err(CardError::UnknownFunction { line: 7, .. })
        ));
    }

    #[test]
    fn unit_suffixes_multiply_implicitly() {
        let t = SymbolTable::new();
        // Corpus: `SY X=135 ft`
        assert!((e("135 ft", &t) - 135.0 * 0.3048).abs() < 1e-12);
        assert!((e("-68 ft", &t) + 68.0 * 0.3048).abs() < 1e-12);
        assert!((e("24 in", &t) - 24.0 * 0.0254).abs() < 1e-12);
        // Corpus: `SY Inp=mm` then `(WD*Inp*FR)/299.7925`
        assert!((e("mm", &t) - 1e-3).abs() < 1e-18);
    }

    #[test]
    fn awg12_diameter_cross_checks_the_mm_constant() {
        // A corpus model writes `SY WD=2.05232` commented "wire diameter in mm" for AWG #12,
        // whose diameter is 2.05232 mm. That is only consistent if mm == 1e-3 in metres.
        let t = syms(&[("WD", 2.052_32)]);
        let metres = e("WD*mm", &t);
        assert!((metres - 2.052_32e-3).abs() < 1e-15, "got {metres}");
    }

    #[test]
    fn user_definitions_shadow_builtin_units() {
        let t = syms(&[("in", 7.0)]);
        assert!((e("in", &t) - 7.0).abs() < 1e-15);
    }

    #[test]
    fn subtraction_is_not_mistaken_for_implicit_multiplication() {
        let t = syms(&[("a", 10.0), ("b", 4.0)]);
        assert!((e("a - b", &t) - 6.0).abs() < 1e-12);
        assert!((e("a-b", &t) - 6.0).abs() < 1e-12);
    }
}
