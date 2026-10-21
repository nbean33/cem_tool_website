//! Wire geometry and segmentation for `nec-rs`.
//!
//! Takes a resolved card deck and produces the segments the method of moments solves on, plus
//! the endpoint connectivity that constrains the current expansion.
//!
//! Covers every NEC-2 geometry card: `GW` wires (tapered with `GC`), `GA` arcs, `GH` helices,
//! `SP`/`SM`/`SC` surface patches, `GS` scaling, the `GM`/`GX`/`GR` structure transforms and the
//! `GE` ground flag. Anything else in the geometry section is refused rather than skipped, so a
//! model cannot quietly produce wrong geometry.
//!
//! # Validating against the reference engine
//!
//! NEC prints its own segmentation — centres, lengths, orientation angles, radii, connectivity
//! and tags — in the `SEGMENTATION DATA` table of its output. That makes geometry checkable
//! before any electromagnetics exists, which is what `cargo xtask geometry` does.
//!
//! ```
//! use nec_cards::parse_deck;
//! use nec_geom::{Geometry, GroundFlag};
//!
//! let mut deck = parse_deck("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n")?;
//! deck.resolve()?;
//! let g = Geometry::build(&deck)?;
//!
//! assert_eq!(g.len(), 9);
//! assert_eq!(g.ground, GroundFlag::FreeSpace);
//! // Segment 5 straddles the feed point, which is why Example1.nec excites segment 5.
//! assert!(g.segment(5).unwrap().center().y.abs() < 1e-12);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod connect;
pub mod geometry;
pub mod lint;
pub mod patch;
pub mod segment;
pub mod transform;

pub use connect::{Connectivity, EndRef, Junction, WireEnd};
pub use geometry::{GeomError, Geometry, GroundFlag, MAX_SEGMENTS};
pub use lint::{AUXILIARY_TAG, Finding, LintContext, LintGround, Severity, lint, lint_with};
pub use patch::{Patch, PatchError};
pub use segment::Segment;
