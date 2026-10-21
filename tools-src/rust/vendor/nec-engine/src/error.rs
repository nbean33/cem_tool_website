//! Why a model could not be evaluated.

/// Why a model could not be evaluated, by stage. The messages are the underlying errors'
/// own, which are written for people.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    /// The deck could not be read or its `SY` variables could not be resolved.
    #[error("{0}")]
    Deck(String),
    /// The structure could not be built.
    #[error("{0}")]
    Geometry(String),
    /// The program cards (sources, loads, ground, ...) are not usable.
    #[error("{0}")]
    Program(String),
    /// The frequency is not usable.
    #[error("{0}")]
    Frequency(String),
    /// The solver failed.
    #[error("{0}")]
    Solve(String),
    /// A numerical Green's function could not be made, found or read.
    #[error("{0}")]
    Ngf(String),
    /// The deck has no voltage source, so there is no input impedance to report.
    #[error("the deck has no voltage source")]
    NoSource,
    /// A name given as a parameter is not an `SY` variable of the deck.
    #[error("`{0}` is not a variable of this deck")]
    UnknownVariable(String),
    /// The work was cancelled.
    #[error("cancelled")]
    Cancelled,
}
