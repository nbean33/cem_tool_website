//! Parse and evaluation errors, always carrying the source line so a bad model in a
//! 467-file corpus can be found without bisecting.

/// A failure while reading a deck or evaluating a symbolic expression.
#[derive(Debug, thiserror::Error)]
pub enum CardError {
    /// A card mnemonic was not two characters.
    #[error("line {line}: malformed card mnemonic {mnemonic:?}")]
    BadMnemonic {
        /// 1-based source line.
        line: usize,
        /// What was found.
        mnemonic: String,
    },

    /// A field was neither a number nor a valid expression.
    #[error("line {line}: cannot parse field {field:?} on {mnemonic} card")]
    BadField {
        /// 1-based source line.
        line: usize,
        /// The card this field belongs to.
        mnemonic: String,
        /// The offending text.
        field: String,
    },

    /// An `SY` card was not of the form `name=expression`.
    #[error("line {line}: malformed SY assignment {text:?}")]
    BadAssignment {
        /// 1-based source line.
        line: usize,
        /// The offending text.
        text: String,
    },

    /// An expression referenced a variable that was never defined.
    #[error("line {line}: undefined symbol {name:?}")]
    UndefinedSymbol {
        /// 1-based source line.
        line: usize,
        /// The unknown identifier.
        name: String,
    },

    /// An expression called a function this evaluator does not implement.
    #[error("line {line}: unknown function {name:?}")]
    UnknownFunction {
        /// 1-based source line.
        line: usize,
        /// The unknown function name.
        name: String,
    },

    /// A recognized 4nec2 field syntax that this implementation does not yet resolve.
    ///
    /// Distinct from [`CardError::BadField`] so the corpus gate can separate "deck we cannot
    /// read" from "deck using a feature scheduled for a later phase".
    #[error("line {line}: {mnemonic} field {field:?} uses unsupported syntax: {reason}")]
    UnsupportedFieldSyntax {
        /// 1-based source line.
        line: usize,
        /// The card.
        mnemonic: String,
        /// The offending field text.
        field: String,
        /// What is unsupported, and why.
        reason: &'static str,
    },

    /// An expression was syntactically invalid.
    #[error("line {line}: bad expression {expr:?}: {reason}")]
    BadExpression {
        /// 1-based source line.
        line: usize,
        /// The offending expression.
        expr: String,
        /// Why it failed.
        reason: String,
    },

    /// A required field was missing.
    #[error("line {line}: {mnemonic} card needs at least {needed} fields, found {found}")]
    TooFewFields {
        /// 1-based source line.
        line: usize,
        /// The card.
        mnemonic: String,
        /// How many are required.
        needed: usize,
        /// How many were present.
        found: usize,
    },

    /// A card cannot be fitted into the engine's 80-column record even at reduced precision.
    #[error("line {line}: {mnemonic} card does not fit in 80 columns even at 7 significant digits")]
    CardTooWide {
        /// 1-based source line.
        line: usize,
        /// The card.
        mnemonic: String,
    },

    /// A 4nec2 extension card that cannot be translated to standard NEC-2.
    #[error("line {line}: cannot translate {mnemonic} to NEC-2: {reason}")]
    Lowering {
        /// 1-based source line.
        line: usize,
        /// The card.
        mnemonic: String,
        /// Why.
        reason: String,
    },

    /// Underlying I/O failure.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result alias for this crate.
pub type Result<T> = core::result::Result<T, CardError>;
