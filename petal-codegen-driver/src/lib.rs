use std::path::PathBuf;

use petal_ast::visitor::ASTVisitor;
use petal_core::string_intern::StringInternPool;

use crate::options::DriverOptions;

pub mod options;

/// A generic driver trait for all codegen drivers to implement.
///
/// All [Driver] implementations must also derive [ASTVisitor].
pub trait Driver<'s>: ASTVisitor {
    /// Creates a new instance of this [Driver] using a [StringInternPool] implementation.
    fn new(options: DriverOptions, string_intern_pool: &'s dyn StringInternPool) -> Self;

    /// Compiles the code to an object file, returning the path to the object file if successful.
    /// If an error occurs, a [Err] may be returned with a human-readable error message.
    fn compile_to_object(&self) -> Result<PathBuf, String>;
}
