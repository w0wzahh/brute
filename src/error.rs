use colored::*;

/// Represents a location in source code
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub filename: Option<String>,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column, filename: None }
    }

    pub fn with_file(line: usize, column: usize, filename: String) -> Self {
        Self { line, column, filename: Some(filename) }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(file) = &self.filename {
            write!(f, "{}:{}:{}", file, self.line, self.column)
        } else {
            write!(f, "line {}, column {}", self.line, self.column)
        }
    }
}

/// Errors that can occur in the Brute compiler and interpreter
#[derive(Debug, thiserror::Error)]
pub enum BruteError {
    // ── Compile-time errors ────────────────────────────────────────────────
    #[error("Syntax error at {location}: {message}")]
    SyntaxError {
        location: SourceLocation,
        message: String,
    },

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Undefined variable: '{0}'")]
    UndefinedVariable(String),

    #[error("Undefined function: '{0}'")]
    UndefinedFunction(String),

    #[error("Undefined type: '{0}'")]
    UndefinedType(String),

    // ── Runtime errors ────────────────────────────────────────────────────
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Value error: {0}")]
    ValueError(String),

    #[error("Index out of bounds: index {index} on length {length}")]
    IndexOutOfBounds { index: i64, length: usize },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Stack overflow: maximum call depth exceeded")]
    StackOverflow,

    // ── Control-flow signals (not really errors) ──────────────────────────
    /// Used internally to unwind a `break` statement
    #[error("break outside loop")]
    Break,

    /// Used internally to unwind a `continue` statement
    #[error("continue outside loop")]
    Continue,

    /// Used internally to carry a return value up the call stack
    #[error("return outside function")]
    Return(Option<Box<crate::interpreter::Value>>),

    // ── Module system ─────────────────────────────────────────────────────
    #[error("Module error: {0}")]
    ModuleError(String),

    #[error("Module not found: '{0}'")]
    ModuleNotFound(String),

    // ── Trait system ──────────────────────────────────────────────────────
    #[error("Trait error: {0}")]
    TraitError(String),

    // ── Pattern matching ──────────────────────────────────────────────────
    #[error("Non-exhaustive patterns: no arm matched {0}")]
    PatternMatchError(String),

    // ── Async / concurrency ───────────────────────────────────────────────
    #[error("Async error: {0}")]
    AsyncError(String),

    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    // ── I/O ───────────────────────────────────────────────────────────────
    #[error("IO error: {0}")]
    IOException(String),

    // ── Misc ─────────────────────────────────────────────────────────────
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Optional access on None value")]
    NullAccess,

    #[error("Timeout: {0}")]
    TimeoutError(String),

    #[error("Argument error: {0}")]
    ArgumentError(String),
}

impl BruteError {
    /// Render the error with coloured output for the terminal.
    pub fn format_error(&self) -> String {
        match self {
            BruteError::SyntaxError { location, message } => format!(
                "{} at {}: {}",
                "Syntax Error".red().bold(),
                location.to_string().yellow(),
                message
            ),
            BruteError::TypeError(msg) => {
                format!("{}: {}", "Type Error".red().bold(), msg)
            }
            BruteError::UndefinedVariable(name) => {
                format!("{}: undefined variable '{}'", "Error".red().bold(), name.cyan())
            }
            BruteError::RuntimeError(msg) => {
                format!("{}: {}", "Runtime Error".red().bold(), msg)
            }
            BruteError::PatternMatchError(val) => {
                format!("{}: no arm matched value '{}'", "Match Error".red().bold(), val)
            }
            _ => format!("{}: {}", "Error".red().bold(), self),
        }
    }

    /// Convenience: build a SyntaxError from parts without a filename.
    pub fn syntax(line: usize, column: usize, message: impl Into<String>) -> Self {
        BruteError::SyntaxError {
            location: SourceLocation::new(line, column),
            message: message.into(),
        }
    }
}

/// The canonical Result type used throughout the codebase.
pub type Result<T> = std::result::Result<T, BruteError>;
