//! Reading and writing NEC-2 card decks, including 4nec2's `SY` symbolic-variable extension.
//!
//! This crate is Phase 0 of the project: it exists so the whole 467-model corpus in
//! `C:\4nec2\models` can be loaded, resolved and re-emitted as engine-ready input *before*
//! any electromagnetics is written. That ordering is deliberate — the reference engine is the
//! oracle for every later phase, and feeding it requires this crate.
//!
//! # Example
//!
//! ```
//! use nec_cards::{parse_deck, to_engine_input};
//!
//! let src = "SY Len=0.2418\n\
//!            GW 1 9 0 -Len 0 0 Len 0 .0001\n\
//!            GE 0\n\
//!            EX 0 1 5 0 1 0\n\
//!            FR 0 1 0 0 300 0\n\
//!            EN\n";
//!
//! let mut deck = parse_deck(src)?;
//! assert!(deck.uses_sy());
//!
//! deck.resolve()?;
//! let inp = to_engine_input(&deck)?;
//!
//! // Symbols are gone and an execute card has been supplied.
//! assert!(inp.contains("GW 1 9 0 -0.2418 0 0 0.2418 0 0.0001"));
//! assert!(inp.contains("XQ"));
//! assert!(!inp.contains("SY"));
//! # Ok::<(), nec_cards::CardError>(())
//! ```
//!
//! # Scope
//!
//! The representation is syntactic. A card is a mnemonic plus fields; this crate does not know
//! that `GW` means "wire" or what `GN 2` implies. Interpreting cards belongs to `nec-geom` and
//! the solver crates. Keeping the split here means rare cards round-trip losslessly without
//! needing semantics first.

pub mod deck;
pub mod edit;
pub mod error;
pub mod expr;
pub mod lower;
pub mod out;
pub mod params;
pub mod parse;
pub mod write;

pub use deck::{Card, Deck, Field, FieldKind, Item, awg_radius_m, is_text_field, ngf_file_name};
pub use error::{CardError, Result};
pub use expr::SymbolTable;
pub use lower::{lower_4nec2, uses_4nec2_extensions};
pub use params::{Variable, format_value, set_variables, variables};
pub use parse::parse_deck;
pub use write::{to_authoring_text, to_engine_input, to_geometry_input};

/// Read a deck from a file, resolving `SY` symbols.
///
/// Decks in the wild are not UTF-8: the corpus contains Latin-1 bytes in comment text. Rather
/// than fail on those, bytes are decoded lossily, since comment text never feeds the numerics.
pub fn read_resolved(path: impl AsRef<std::path::Path>) -> Result<Deck> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut deck = parse_deck(&text)?;
    deck.resolve()?;
    Ok(deck)
}
