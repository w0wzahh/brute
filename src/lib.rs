// Core language modules
pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod formatter;
pub mod compiler;
pub mod type_system;
pub mod traits;
pub mod module_system;

// LLVM-based native code generator — only built with the `llvm` feature
#[cfg(feature = "llvm")]
pub mod code_gen;

// Standard library sub-modules
pub mod stdlib;

// Re-exports for convenience
pub use error::{BruteError, Result};
pub use ast::{Program, Stmt, Expr, Literal, Type};
pub use lexer::Lexer;
pub use parser::Parser;
pub use interpreter::Interpreter;
pub use formatter::Formatter;
pub use compiler::Compiler;
pub use type_system::TypeChecker;
