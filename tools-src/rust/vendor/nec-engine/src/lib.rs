//! Orchestration over the nec-rs solver: what a user asks of an antenna model, rather than what
//! NEC's cards ask of it.
//!
//! * [`Model`] turns deck text (with optional `SY` overrides) into something solvable.
//! * [`Metrics`] is what is read off one solution: impedance, SWR, gain, front-to-back,
//!   efficiency.
//! * [`sweep`] solves many frequencies or parameter values in parallel.
//! * [`optimize()`] tunes `SY` variables against [`Goal`]s with Nelder–Mead or differential
//!   evolution, built on the problem-independent minimizers in [`optim`].
//! * [`Project`] is the native project file: a deck plus the settings a deck cannot hold.
//!
//! Every long-running entry point takes a cancellation flag and reports progress through a
//! callback, so a GUI can show work as it happens and stop it.

mod error;
pub mod fields;
mod goal;
pub mod checks;
pub mod matching;
pub mod segmentation;
pub mod steprad;
mod metrics;
mod model;
pub mod optim;
mod optimize;
pub mod pattern;
pub mod project;
mod report;
pub mod sweep;

pub use error::EngineError;
pub use goal::{Goal, GoalKind, cost};
pub use metrics::{Capture, Direction, EvalOptions, Frame, Metrics, swr};
pub use model::{Model, ModelOptions, NgfStore};
pub use optimize::{OptimizeOutcome, OptimizeProgress, OptimizeSpec, Parameter, optimize};
pub use fields::{Slice, SlicePlane, SliceRequest};
pub use report::ground_label;
pub use project::{Project, ProjectError, Units};

/// A `Result` whose error is an [`EngineError`].
pub type Result<T, E = EngineError> = std::result::Result<T, E>;

#[cfg(test)]
pub(crate) mod testdata {
    /// A half-wave dipole for 300 MHz along y, its half-length the `SY` variable `len`.
    pub const DIPOLE: &str = "CE\nSY len=.2418\nGW 1 9 0 -len 0 0 len 0 .0001\nGE 0\nEX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\nEN\n";
}
