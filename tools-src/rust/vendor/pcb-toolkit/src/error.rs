/// Reject non-finite inputs before any range check runs.
///
/// Every public calculation function calls this first, over all of its `f64` inputs. It is
/// deliberately a slice of `(name, value)` rather than one call per argument: a single line
/// per entry point is hard to half-apply, and the name travels with the value so the error
/// says which input was wrong.
///
/// Returns the **first** offending input, in the order given, so the message is deterministic
/// for a caller that passed several.
pub(crate) fn all_finite(inputs: &[(&'static str, f64)]) -> Result<(), CalcError> {
    for (name, value) in inputs {
        if !value.is_finite() {
            return Err(CalcError::NotFinite {
                name,
                value: *value,
            });
        }
    }
    Ok(())
}

/// Errors returned by pcb-toolkit calculation functions.
#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    #[error("W/H ratio {ratio:.3} outside valid range [{min}, {max}]")]
    InvalidRatio { ratio: f64, min: f64, max: f64 },

    #[error("negative dimension: {name} = {value}")]
    NegativeDimension { name: &'static str, value: f64 },

    #[error("value out of range: {name} = {value} (expected {expected})")]
    OutOfRange {
        name: &'static str,
        value: f64,
        expected: &'static str,
    },

    #[error("unknown material: {0}")]
    UnknownMaterial(String),

    /// A string did not name a variant of an enumerated input.
    ///
    /// One variant serves every `FromStr` in the crate, and `expected` carries the valid
    /// values so the message is actionable without the caller re-listing them. Before this
    /// existed, each enum had a hand-written parser in both the CLI and the Tauri bridge —
    /// eleven copies in total, with the two `Awg` parsers disagreeing about which aliases
    /// they accepted.
    #[error("unknown {kind} '{value}' — valid values: {expected}")]
    UnknownVariant {
        kind: &'static str,
        value: String,
        expected: &'static str,
    },

    /// An input was NaN or infinite.
    ///
    /// This has to be its own check rather than falling out of the range validations,
    /// because every one of those is written as `x <= 0.0` — and **every comparison against
    /// NaN is false**, so NaN passed all of them. It then propagated to the output, where it
    /// printed as `NaN`, serialised to JSON as `null`, and in one case produced the confident
    /// falsehood "Ret loss = infinite (perfect match)". A CLI sweep found 313 argument
    /// spellings that accepted a non-finite number; see
    /// `non_finite_numbers_are_rejected_everywhere` in the CLI's integration tests.
    ///
    /// Arguments carrying a unit were never affected: `Length`, `Freq`, `Capacitance` and
    /// `Inductance` parse a number plus a known suffix and reject anything else. The gap was
    /// every bare `f64`, which `f64::from_str` happily fills from "nan", "inf", "Infinity"
    /// and their signed and mixed-case variants.
    #[error("{name} must be a finite number (got {value})")]
    NotFinite { name: &'static str, value: f64 },

    #[error("insufficient inputs: {0}")]
    InsufficientInputs(&'static str),

    /// The inputs were individually valid, but the model produced a result that
    /// cannot be physical — typically because the geometry falls outside the
    /// formula's range of validity.
    #[error("unphysical result: {name} = {value} ({reason})")]
    UnphysicalResult {
        name: &'static str,
        value: f64,
        reason: &'static str,
    },
}
