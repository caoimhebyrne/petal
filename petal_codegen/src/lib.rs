use petal_ir::function::Function;
use std::path::PathBuf;

pub mod aarch64;
pub mod error;
pub(crate) mod visitor;
pub mod x86_64;
pub use aarch64::*;
pub use x86_64::*;

use crate::error::DriverResult;

/// Represents a code-generation driver.
/// A driver is typically for a specific platform, for example: [X86_64LinuxDriver].
pub trait Driver {
    /// Initializes a new [Driver] for code generation.
    fn new() -> Self
    where
        Self: Sized;

    /// Generates platform-specific code from the intermediate representation.
    fn generate(&mut self, functions: Vec<Function>, output_path: &PathBuf) -> DriverResult<()>;
}
