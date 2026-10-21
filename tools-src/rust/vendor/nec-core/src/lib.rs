//! Shared units, physical constants and error types for `nec-rs`.
//!
//! This crate deliberately holds no physics. It exists so that every other crate agrees on
//! the same constants and the same notion of "wavelength", because in a method-of-moments
//! solver a disagreement about the speed of light shows up as a systematic error in every
//! computed impedance.

pub mod constants;
pub mod geom;
pub mod units;

pub use constants::{CVEL_MHZ_M, ETA0, EPSILON0, MU0, SPEED_OF_LIGHT};
pub use geom::Point3;
pub use units::{Frequency, Wavelengths};

/// Errors that can arise before any numerics run: bad geometry, unsupported input, or a
/// model outside this implementation's documented limits.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// A model exceeded a documented capacity limit.
    #[error("model too large: {what} is {actual}, limit is {limit}")]
    TooLarge {
        /// What was being counted (e.g. `"segment count"`).
        what: &'static str,
        /// The value the model asked for.
        actual: usize,
        /// The limit this implementation enforces.
        limit: usize,
    },

    /// A geometric quantity was non-physical.
    #[error("invalid geometry: {0}")]
    InvalidGeometry(String),

    /// A frequency was zero or negative.
    #[error("invalid frequency: {0} MHz")]
    InvalidFrequency(f64),
}

/// Result alias for this crate.
pub type Result<T> = core::result::Result<T, CoreError>;
